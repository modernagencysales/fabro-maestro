use std::collections::HashSet;
use std::path::Path as StdPath;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use fabro_store::CachedRunProjection;
use serde_json::json;

use super::super::{ApiError, AppState, MAX_PAGE_OFFSET, PaginationParams};
use crate::principal_middleware::RequiredUser;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/workflows", get(list_workflows))
        .route("/workflows/{name}", get(get_workflow))
        .route("/workflows/{name}/runs", get(list_workflow_runs))
}

fn paginate_items<T>(items: Vec<T>, pagination: &PaginationParams) -> (Vec<T>, bool) {
    let limit = pagination.limit.clamp(1, 100) as usize;
    let offset = pagination.offset.min(MAX_PAGE_OFFSET) as usize;
    let mut data: Vec<_> = items.into_iter().skip(offset).take(limit + 1).collect();
    let has_more = data.len() > limit;
    data.truncate(limit);
    (data, has_more)
}

fn slugify_name(name: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('-');
            last_was_separator = true;
        }
    }

    if last_was_separator {
        slug.pop();
    }

    if slug.is_empty() {
        "workflow".to_string()
    } else {
        slug
    }
}

fn workflow_slug(entry: &CachedRunProjection) -> String {
    entry
        .summary
        .workflow
        .slug
        .as_deref()
        .filter(|slug| !slug.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| slugify_name(&entry.summary.workflow.name))
}

fn workflow_filename(entry: &CachedRunProjection, slug: &str) -> String {
    let configured_graph = entry.projection.spec.settings.workflow.graph.trim();
    if !configured_graph.is_empty() {
        if let Some(filename) = StdPath::new(configured_graph)
            .file_name()
            .and_then(|filename| filename.to_str())
        {
            if !filename.trim().is_empty() && (filename != "workflow.fabro" || slug == "workflow") {
                return filename.to_string();
            }
        }
    }

    format!("{slug}.fabro")
}

fn workflow_description(entry: &CachedRunProjection) -> String {
    entry
        .projection
        .spec
        .settings
        .workflow
        .description
        .as_deref()
        .filter(|description| !description.trim().is_empty())
        .or_else(|| {
            let goal = entry.projection.spec.graph.goal();
            (!goal.trim().is_empty()).then_some(goal)
        })
        .unwrap_or(&entry.summary.goal)
        .to_string()
}

fn workflow_matches(entry: &CachedRunProjection, requested: &str) -> bool {
    workflow_slug(entry) == requested || entry.summary.workflow.name == requested
}

fn latest_workflow_entries(entries: Vec<CachedRunProjection>) -> Vec<CachedRunProjection> {
    let mut seen = HashSet::new();
    let mut workflows = Vec::new();

    for entry in entries {
        if entry.summary.lifecycle.archived {
            continue;
        }

        let slug = workflow_slug(&entry);
        if seen.insert(slug) {
            workflows.push(entry);
        }
    }

    workflows
}

async fn list_workflows(
    _auth: RequiredUser,
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    let entries = match state
        .store
        .list_cached_runs(&fabro_store::ListRunsQuery::default())
        .await
    {
        Ok(entries) => entries,
        Err(err) => {
            return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                .into_response();
        }
    };

    let items = latest_workflow_entries(entries)
        .into_iter()
        .map(|entry| {
            let slug = workflow_slug(&entry);
            json!({
                "name": entry.summary.workflow.name,
                "slug": slug,
                "filename": workflow_filename(&entry, &slug),
                "last_run": {
                    "ran_at": entry.summary.timestamps.started_at
                        .unwrap_or(entry.summary.timestamps.created_at)
                        .to_rfc3339(),
                },
                "schedule": null,
            })
        })
        .collect::<Vec<_>>();
    let (data, has_more) = paginate_items(items, &pagination);

    (
        StatusCode::OK,
        Json(json!({
            "data": data,
            "pagination": { "has_more": has_more },
        })),
    )
        .into_response()
}

async fn get_workflow(
    _auth: RequiredUser,
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Response {
    let entries = match state
        .store
        .list_cached_runs(&fabro_store::ListRunsQuery::default())
        .await
    {
        Ok(entries) => entries,
        Err(err) => {
            return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                .into_response();
        }
    };

    let Some(entry) = latest_workflow_entries(entries)
        .into_iter()
        .find(|entry| workflow_matches(entry, &name))
    else {
        return ApiError::not_found("Workflow not found.").into_response();
    };

    let slug = workflow_slug(&entry);
    (
        StatusCode::OK,
        Json(json!({
            "name": entry.summary.workflow.name,
            "slug": slug,
            "description": workflow_description(&entry),
            "filename": workflow_filename(&entry, &slug),
            "settings": entry.projection.spec.settings,
            "graph": entry.projection.spec.graph_source.as_deref().unwrap_or(""),
        })),
    )
        .into_response()
}

async fn list_workflow_runs(
    _auth: RequiredUser,
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> Response {
    let entries = match state
        .store
        .list_cached_runs(&fabro_store::ListRunsQuery::default())
        .await
    {
        Ok(entries) => entries,
        Err(err) => {
            return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                .into_response();
        }
    };

    if !entries.iter().any(|entry| workflow_matches(entry, &name)) {
        return ApiError::not_found("Workflow not found.").into_response();
    }

    let items = entries
        .into_iter()
        .filter(|entry| !entry.summary.lifecycle.archived && workflow_matches(entry, &name))
        .map(|entry| entry.summary)
        .collect::<Vec<_>>();
    let (data, has_more) = paginate_items(items, &pagination);

    (
        StatusCode::OK,
        Json(json!({
            "data": data,
            "meta": { "has_more": has_more },
        })),
    )
        .into_response()
}
