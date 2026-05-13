use fabro_interview::Answer;
use serde_json::Value;

use crate::payload::{
    self, SlackActionPayload, SlackAnswerSubmission, SlackGateDecisionSubmission,
};

const MULTI_SELECT_BLOCK_ID: &str = "interview.checkboxes";
const MULTI_SELECT_ACTION_ID: &str = "interview.select";
const ANSWER_ACTION_ID: &str = "interview.answer";
const GATE_DECISION_ACTION_ID: &str = "gate.decision";
const MULTI_SELECT_SUBMIT_ACTION_ID: &str = "interview.submit";

fn action_id_matches(action_id: &str, base: &str) -> bool {
    action_id == base
        || action_id
            .strip_prefix(base)
            .is_some_and(|rest| rest.starts_with('.'))
}

/// Parses a Slack interaction payload and returns a server-routable answer
/// submission.
pub fn parse_interaction(payload: &Value) -> Option<SlackAnswerSubmission> {
    if payload["type"].as_str()? != "block_actions" {
        return None;
    }

    let action = payload["actions"].as_array()?.first()?;
    let action_id = action["action_id"].as_str()?;
    let value = action["value"].as_str()?;
    let routed: SlackActionPayload = serde_json::from_str(value).ok()?;
    let question_ref = routed.question_ref()?;
    let actor = payload::interaction_actor(payload)?;

    let action_type = action["type"].as_str().unwrap_or("button");

    let answer = match action_type {
        "button" if action_id_matches(action_id, ANSWER_ACTION_ID) => match routed {
            SlackActionPayload::Yes { .. } => Answer::yes(),
            SlackActionPayload::No { .. } => Answer::no(),
            SlackActionPayload::Selected { key, .. } => Answer {
                value: fabro_interview::AnswerValue::Selected(key),
                selected_option: None,
                text: None,
            },
            SlackActionPayload::SubmitMulti { .. } | SlackActionPayload::GateDecision { .. } => {
                return None;
            }
        },
        "button" if action_id_matches(action_id, MULTI_SELECT_SUBMIT_ACTION_ID) => {
            extract_checkbox_selections(payload)
        }
        "checkboxes" => {
            // Ignore checkbox toggle events — wait for Submit button
            return None;
        }
        _ => return None,
    };

    Some(SlackAnswerSubmission {
        run_id: question_ref.run_id,
        qid: question_ref.qid,
        answer,
        actor,
    })
}

/// Parses a Slack approval-gate interaction payload into a server-routable gate
/// decision. This is separate from interview answers because the server needs
/// the pending question record to map approve/reject/edit onto the gate's
/// concrete outgoing edges.
pub fn parse_gate_decision(payload: &Value) -> Option<SlackGateDecisionSubmission> {
    if payload["type"].as_str()? != "block_actions" {
        return None;
    }

    let action = payload["actions"].as_array()?.first()?;
    if action["type"].as_str().unwrap_or("button") != "button" {
        return None;
    }
    let action_id = action["action_id"].as_str()?;
    if !action_id_matches(action_id, GATE_DECISION_ACTION_ID) {
        return None;
    }

    let value = action["value"].as_str()?;
    let routed: SlackActionPayload = serde_json::from_str(value).ok()?;
    let SlackActionPayload::GateDecision {
        run_id,
        gate_id,
        decision,
    } = routed
    else {
        return None;
    };
    let actor = payload::interaction_actor(payload)?;

    Some(SlackGateDecisionSubmission {
        run_id,
        gate_id,
        decision,
        actor,
    })
}

/// Extract selected checkbox values from `payload.state.values`.
fn extract_checkbox_selections(payload: &Value) -> Answer {
    let selected =
        payload["state"]["values"][MULTI_SELECT_BLOCK_ID][MULTI_SELECT_ACTION_ID]["selected_options"]
            .as_array();

    match selected {
        Some(options) if !options.is_empty() => {
            let values: Vec<String> = options
                .iter()
                .filter_map(|opt| opt["value"].as_str().map(String::from))
                .collect();
            Answer::multi_selected(values)
        }
        _ => Answer::skipped(),
    }
}

#[cfg(test)]
mod tests {
    use fabro_interview::AnswerValue;

    use super::*;

    #[test]
    fn parse_yes_button_click() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "team": { "id": "T123" },
            "user": { "id": "U123", "name": "ada" },
            "actions": [{
                "action_id": "interview.answer.0",
                "type": "button",
                "value": "{\"kind\":\"yes\",\"run_id\":\"run-1\",\"qid\":\"q-1\"}"
            }]
        });
        let result = parse_interaction(&payload).unwrap();
        assert_eq!(result.run_id, "run-1");
        assert_eq!(result.qid, "q-1");
        assert_eq!(result.answer.value, AnswerValue::Yes);
        assert_eq!(
            result.actor,
            fabro_types::Principal::Slack {
                team_id: "T123".to_string(),
                user_id: "U123".to_string(),
                user_name: Some("ada".to_string()),
            }
        );
    }

    #[test]
    fn parse_no_button_click() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "team": { "id": "T123" },
            "user": { "id": "U123", "name": "ada" },
            "actions": [{
                "action_id": "interview.answer.1",
                "type": "button",
                "value": "{\"kind\":\"no\",\"run_id\":\"run-1\",\"qid\":\"q-2\"}"
            }]
        });
        let result = parse_interaction(&payload).unwrap();
        assert_eq!(result.run_id, "run-1");
        assert_eq!(result.qid, "q-2");
        assert_eq!(result.answer.value, AnswerValue::No);
    }

    #[test]
    fn parse_multiple_choice_button() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "team": { "id": "T123" },
            "user": { "id": "U123", "name": "ada" },
            "actions": [{
                "action_id": "interview.answer.0",
                "type": "button",
                "value": "{\"kind\":\"selected\",\"run_id\":\"run-1\",\"qid\":\"q-3\",\"key\":\"rs\"}"
            }]
        });
        let result = parse_interaction(&payload).unwrap();
        assert_eq!(result.qid, "q-3");
        assert_eq!(result.answer.value, AnswerValue::Selected("rs".to_string()));
    }

    #[test]
    fn parse_gate_decision_button() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "team": { "id": "T123" },
            "user": { "id": "U123", "name": "ada" },
            "actions": [{
                "action_id": "gate.decision.0",
                "type": "button",
                "value": "{\"kind\":\"gate_decision\",\"run_id\":\"run-1\",\"gate_id\":\"gate-1\",\"decision\":\"approve\"}"
            }]
        });
        let result = parse_gate_decision(&payload).unwrap();
        assert_eq!(result.run_id, "run-1");
        assert_eq!(result.gate_id, "gate-1");
        assert_eq!(result.decision, payload::SlackGateDecision::Approve);
        assert_eq!(
            result.actor,
            fabro_types::Principal::Slack {
                team_id: "T123".to_string(),
                user_id: "U123".to_string(),
                user_name: Some("ada".to_string()),
            }
        );
        assert!(parse_interaction(&payload).is_none());
    }

    #[test]
    fn checkbox_toggle_is_ignored() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "team": { "id": "T123" },
            "user": { "id": "U123", "name": "ada" },
            "actions": [{
                "action_id": "interview.select",
                "type": "checkboxes",
                "selected_options": [
                    { "value": "a" },
                    { "value": "b" }
                ]
            }]
        });
        assert!(parse_interaction(&payload).is_none());
    }

    #[test]
    fn submit_button_reads_checkbox_state() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "team": { "id": "T123" },
            "user": { "id": "U123", "name": "ada" },
            "actions": [{
                "action_id": "interview.submit.0",
                "type": "button",
                "value": "{\"kind\":\"submit_multi\",\"run_id\":\"run-1\",\"qid\":\"q-5\"}"
            }],
            "state": {
                "values": {
                    "interview.checkboxes": {
                        "interview.select": {
                            "type": "checkboxes",
                            "selected_options": [
                                { "value": "auth" },
                                { "value": "billing" }
                            ]
                        }
                    }
                }
            }
        });
        let result = parse_interaction(&payload).unwrap();
        assert_eq!(result.qid, "q-5");
        assert_eq!(
            result.answer.value,
            AnswerValue::MultiSelected(vec!["auth".to_string(), "billing".to_string()])
        );
    }

    #[test]
    fn submit_button_with_no_checkboxes_selected() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "team": { "id": "T123" },
            "user": { "id": "U123", "name": "ada" },
            "actions": [{
                "action_id": "interview.submit.0",
                "type": "button",
                "value": "{\"kind\":\"submit_multi\",\"run_id\":\"run-1\",\"qid\":\"q-5\"}"
            }],
            "state": {
                "values": {
                    "interview.checkboxes": {
                        "interview.select": {
                            "type": "checkboxes",
                            "selected_options": []
                        }
                    }
                }
            }
        });
        let result = parse_interaction(&payload).unwrap();
        assert_eq!(result.qid, "q-5");
        assert_eq!(result.answer.value, AnswerValue::Skipped);
    }

    #[test]
    fn parse_plain_text_input() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "actions": [{
                "action_id": "interview.answer",
                "type": "plain_text_input",
                "value": "{\"kind\":\"selected\",\"run_id\":\"run-1\",\"qid\":\"q-6\",\"key\":\"input\"}"
            }]
        });
        assert!(parse_interaction(&payload).is_none());
    }

    #[test]
    fn returns_none_for_empty_actions() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "actions": []
        });
        assert!(parse_interaction(&payload).is_none());
    }

    #[test]
    fn returns_none_for_unknown_type() {
        let payload = serde_json::json!({
            "type": "view_submission"
        });
        assert!(parse_interaction(&payload).is_none());
    }

    #[test]
    fn returns_none_for_malformed_action_id() {
        let payload = serde_json::json!({
            "type": "block_actions",
            "actions": [{
                "action_id": "no-colon",
                "type": "button",
                "value": "yes"
            }]
        });
        assert!(parse_interaction(&payload).is_none());
    }
}
