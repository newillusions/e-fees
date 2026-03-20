use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use tracing::error;
use utoipa::OpenApi;

use crate::AppState;

const PUBLIC_PATHS: &[&str] = &["/health", "/api/health", "/help", "/openapi.json", "/docs"];

/// Health check endpoint (no auth required).
///
/// Returns service status with uptime, timestamp, and dependency health.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service healthy or degraded", body = crate::schemas::HealthResponse),
        (status = 503, description = "Service unhealthy"),
    )
)]
pub async fn health(State(state): State<Arc<AppState>>) -> (axum::http::StatusCode, Json<Value>) {
    let start = Instant::now();
    let db_result = tokio::time::timeout(
        Duration::from_secs(3),
        state.db.health(),
    )
    .await;
    let db_ok = matches!(&db_result, Ok(Ok(_)));
    let db_latency = start.elapsed().as_millis() as f64;

    if !db_ok {
        error!("Health check: SurrealDB unreachable (latency: {db_latency}ms)");
    }

    let status = if db_ok { "ok" } else { "error" };
    let http_status = if db_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    let body = json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": state.started_at.elapsed().as_secs_f64(),
        "checked_at": chrono::Utc::now().to_rfc3339(),
        "dependencies": {
            "surrealdb": {
                "status": if db_ok { "ok" } else { "error" },
                "latency_ms": db_latency,
            }
        }
    });

    (http_status, Json(body))
}

/// Self-documentation endpoint (no auth required).
#[utoipa::path(
    get,
    path = "/help",
    tag = "Health",
    responses(
        (status = 200, description = "Service documentation", body = crate::schemas::HelpResponse),
    )
)]
pub async fn help() -> Json<Value> {
    let spec = crate::ApiDoc::openapi();

    let endpoints: Vec<Value> = spec
        .paths
        .paths
        .iter()
        .flat_map(|(path, item)| {
            let path = path.clone();
            let pairs: Vec<(&str, Option<&utoipa::openapi::path::Operation>)> = vec![
                ("GET", item.get.as_ref()),
                ("POST", item.post.as_ref()),
                ("PUT", item.put.as_ref()),
                ("DELETE", item.delete.as_ref()),
                ("PATCH", item.patch.as_ref()),
            ];
            pairs
                .into_iter()
                .filter_map(|(method, op)| {
                    op.map(|op| {
                        let is_public = PUBLIC_PATHS.contains(&path.as_str());
                        json!({
                            "method": method,
                            "path": path,
                            "description": op.description.as_deref()
                                .or(op.summary.as_deref())
                                .unwrap_or(""),
                            "auth": !is_public,
                        })
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    Json(json!({
        "service": "e-fees-api",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "REST API for managing fee proposals, projects, companies, and contacts.",
        "config_source": "environment variables (.env)",
        "endpoints": endpoints,
    }))
}
