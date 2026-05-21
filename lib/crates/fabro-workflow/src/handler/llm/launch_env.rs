use std::collections::HashMap;
use std::sync::Arc;

use fabro_agent::{Sandbox, ToolEnvProvider};
use fabro_auth::{CliAgentKind, CredentialResolver, CredentialUsage, ResolvedCredential};
use fabro_model::{Catalog, CredentialRef, Provider};
use tokio_util::sync::CancellationToken;

use super::cli::{AgentCli, process_env_var};
use crate::error::Error;
use crate::event::{Emitter, RunNoticeCode, RunNoticeLevel};

const CODEX_AUTH_JSON_BASE64_ENV: &str = "CODEX_AUTH_JSON_BASE64";

pub(crate) struct AgentLaunchEnvRequest<'a> {
    pub provider: Provider,
    pub cli: AgentCli,
    pub catalog: &'a Catalog,
    pub resolver: Option<&'a CredentialResolver>,
    pub tool_env: Option<&'a Arc<dyn ToolEnvProvider>>,
    pub github_token_refresh_managed: bool,
    pub stage_label: &'static str,
    pub emitter: &'a Arc<Emitter>,
    pub sandbox: &'a Arc<dyn Sandbox>,
    pub cancel_token: &'a CancellationToken,
}

pub(crate) async fn resolve_agent_launch_env(
    request: AgentLaunchEnvRequest<'_>,
) -> Result<HashMap<String, String>, Error> {
    let mut tool_env = resolve_tool_env(&request).await?;
    let codex_auth_json_base64 = take_codex_auth_json_base64(request.cli, &mut tool_env);
    if let Some(auth_json_base64) = codex_auth_json_base64.as_deref() {
        materialize_codex_auth_json(&request, auth_json_base64).await?;
    }

    let cli_agent = match request.cli {
        AgentCli::Claude => CliAgentKind::Claude,
        AgentCli::Codex => CliAgentKind::Codex,
        AgentCli::Gemini => CliAgentKind::Gemini,
    };

    let mut launch_env = if codex_auth_json_base64.is_some() {
        HashMap::new()
    } else if let Some(resolver) = request.resolver {
        let resolved = resolver
            .resolve(
                request.provider,
                CredentialUsage::CliAgent(cli_agent),
                request.catalog,
            )
            .await
            .map_err(|err| {
                Error::handler_with_source(
                    format!("Failed to resolve {} credential", request.stage_label),
                    err,
                )
            })?;
        let ResolvedCredential::Cli(cli_credential) = resolved else {
            return Err(Error::handler("Expected CLI credential".to_string()));
        };
        if let Some(login_cmd) = &cli_credential.login_command {
            let login_result = request
                .sandbox
                .exec_command(
                    login_cmd,
                    30_000,
                    None,
                    None,
                    Some(request.cancel_token.child_token()),
                )
                .await
                .map_err(|err| {
                    Error::handler_with_source(
                        format!("{} credential login failed", request.stage_label),
                        err,
                    )
                })?;
            if !login_result.is_success() {
                tracing::warn!(
                    exit_code = login_result.display_exit_code(),
                    stage = request.stage_label,
                    "{} credential login failed: {}",
                    request.stage_label,
                    login_result.stderr
                );
            }
        }
        cli_credential.env_vars
    } else {
        let mut env = HashMap::new();
        let provider_id = request.provider.id();
        if let Some(provider) = request.catalog.provider(&provider_id) {
            for credential_ref in &provider.credentials {
                let CredentialRef::Env(name) = credential_ref else {
                    continue;
                };
                if let Some(value) = process_env_var(name) {
                    env.insert(name.clone(), value);
                }
            }
        } else {
            for name in request.provider.api_key_env_vars() {
                if let Some(value) = process_env_var(name) {
                    env.insert((*name).to_string(), value);
                }
            }
        }
        env
    };

    launch_env.extend(tool_env);

    Ok(launch_env)
}

async fn resolve_tool_env(
    request: &AgentLaunchEnvRequest<'_>,
) -> Result<HashMap<String, String>, Error> {
    if let Some(provider) = request.tool_env {
        if request.github_token_refresh_managed {
            request.emitter.notice(
                RunNoticeLevel::Info,
                RunNoticeCode::GithubTokenRefreshLimited,
                format!(
                    "{} agent stages receive GitHub tokens at process launch; stages running \
                     beyond token expiry may need to be retried.",
                    request.stage_label
                ),
            );
        }
        provider.resolve().await.map_err(|err| {
            Error::handler_with_anyhow(
                format!("Failed to resolve {} agent env", request.stage_label),
                err,
            )
        })
    } else {
        Ok(HashMap::new())
    }
}

fn take_codex_auth_json_base64(
    cli: AgentCli,
    env: &mut HashMap<String, String>,
) -> Option<String> {
    if cli != AgentCli::Codex {
        return None;
    }
    let auth_json = env.remove(CODEX_AUTH_JSON_BASE64_ENV)?;
    env.remove("OPENAI_API_KEY");
    Some(auth_json)
}

async fn materialize_codex_auth_json(
    request: &AgentLaunchEnvRequest<'_>,
    auth_json_base64: &str,
) -> Result<(), Error> {
    let env = HashMap::from([(
        CODEX_AUTH_JSON_BASE64_ENV.to_string(),
        auth_json_base64.to_string(),
    )]);
    let command = codex_auth_materialization_command();
    let result = request
        .sandbox
        .exec_command(
            &command,
            30_000,
            None,
            Some(&env),
            Some(request.cancel_token.child_token()),
        )
        .await
        .map_err(|err| {
            Error::handler_with_source(
                format!("Failed to materialize {} Codex auth", request.stage_label),
                err,
            )
        })?;
    if result.is_success() {
        return Ok(());
    }
    Err(Error::handler(format!(
        "{} Codex auth materialization failed with exit code {}",
        request.stage_label,
        result.display_exit_code()
    )))
}

fn codex_auth_materialization_command() -> String {
    format!(
        "mkdir -p \"$HOME/.codex\" && umask 077 && printf '%s' \"${{{}}}\" | base64 -d > \
         \"$HOME/.codex/auth.json\"",
        CODEX_AUTH_JSON_BASE64_ENV
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        CODEX_AUTH_JSON_BASE64_ENV, codex_auth_materialization_command,
        take_codex_auth_json_base64,
    };
    use crate::handler::llm::cli::AgentCli;

    #[test]
    fn codex_auth_json_env_removes_api_key_from_launch_env() {
        let mut env = HashMap::from([
            (
                CODEX_AUTH_JSON_BASE64_ENV.to_string(),
                "encoded-auth".to_string(),
            ),
            ("OPENAI_API_KEY".to_string(), "api-key".to_string()),
            ("OTHER".to_string(), "kept".to_string()),
        ]);

        let auth_json = take_codex_auth_json_base64(AgentCli::Codex, &mut env);

        assert_eq!(auth_json.as_deref(), Some("encoded-auth"));
        assert!(!env.contains_key(CODEX_AUTH_JSON_BASE64_ENV));
        assert!(!env.contains_key("OPENAI_API_KEY"));
        assert_eq!(env.get("OTHER").map(String::as_str), Some("kept"));
    }

    #[test]
    fn codex_auth_json_env_is_ignored_for_other_clis() {
        let mut env = HashMap::from([(
            CODEX_AUTH_JSON_BASE64_ENV.to_string(),
            "encoded-auth".to_string(),
        )]);

        let auth_json = take_codex_auth_json_base64(AgentCli::Claude, &mut env);

        assert!(auth_json.is_none());
        assert_eq!(
            env.get(CODEX_AUTH_JSON_BASE64_ENV).map(String::as_str),
            Some("encoded-auth")
        );
    }

    #[test]
    fn codex_auth_materialization_command_writes_private_auth_file_from_base64_env() {
        let command = codex_auth_materialization_command();

        assert!(command.contains("mkdir -p \"$HOME/.codex\""));
        assert!(command.contains("umask 077"));
        assert!(command.contains("printf '%s' \"${CODEX_AUTH_JSON_BASE64}\""));
        assert!(command.contains("base64 -d"));
        assert!(command.contains("> \"$HOME/.codex/auth.json\""));
    }
}
