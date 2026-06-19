use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{extract::State, http::StatusCode, response::Json};
use emittiv_container_utils::health::{
    compute_status, DependencyStatus, EndpointInfo, HealthResponse,
};
use serde::Serialize;
use tracing::{error, warn};
use utoipa::OpenApi;

use crate::AppState;

const PUBLIC_PATHS: &[&str] = &["/health", "/api/health", "/help", "/openapi.json", "/docs"];

/// Health check endpoint (no auth required).
///
/// Returns service status with uptime, timestamp, and dependency health.
/// Checks SurrealDB (critical) and Ollama (non-critical) concurrently.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service healthy or degraded", body = crate::schemas::HealthResponse),
        (status = 503, description = "Service unhealthy"),
    )
)]
pub async fn health(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthResponse>) {
    // Run both dependency checks concurrently
    let db_check = async {
        let start = Instant::now();
        let result = tokio::time::timeout(Duration::from_secs(3), state.db.health()).await;
        let ok = matches!(&result, Ok(Ok(_)));
        let latency = start.elapsed().as_millis() as f64;
        (ok, latency)
    };

    let ollama_check = async {
        let start = Instant::now();
        let ok = tokio::time::timeout(
            Duration::from_secs(2),
            state
                .http
                .get(format!("{}/api/tags", state.ollama_url))
                .send(),
        )
        .await
        .map(|r| r.map(|r| r.status().is_success()).unwrap_or(false))
        .unwrap_or(false);
        let latency = start.elapsed().as_millis() as f64;
        (ok, latency)
    };

    let ((db_ok, db_latency), (ollama_ok, ollama_latency)) = tokio::join!(db_check, ollama_check);

    if !db_ok {
        error!("Health check: SurrealDB unreachable (latency: {db_latency}ms)");
    }
    if !ollama_ok {
        warn!("Health check: Ollama unreachable (latency: {ollama_latency}ms)");
    }

    let mut dependencies = HashMap::new();
    dependencies.insert(
        "surrealdb".to_string(),
        DependencyStatus {
            status: if db_ok { "ok" } else { "error" }.to_string(),
            latency_ms: db_latency,
        },
    );
    dependencies.insert(
        "ollama".to_string(),
        DependencyStatus {
            status: if ollama_ok { "ok" } else { "error" }.to_string(),
            latency_ms: ollama_latency,
        },
    );

    // Only surrealdb is critical — ollama failure produces "degraded" not "error"
    let (status, is_error) = compute_status(&dependencies, &["surrealdb"]);
    let http_status = if is_error {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    let body = HealthResponse {
        status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: state.started_at.elapsed().as_secs_f64(),
        checked_at: chrono::Utc::now().to_rfc3339(),
        dependencies,
    };

    (http_status, Json(body))
}

/// /help response shape — extends the container-utils struct with a description field.
#[derive(Serialize)]
pub struct HelpResponseWithDesc {
    service: String,
    version: String,
    description: String,
    endpoints: Vec<EndpointInfo>,
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
pub async fn help() -> Json<HelpResponseWithDesc> {
    let spec = crate::ApiDoc::openapi();

    let endpoints: Vec<EndpointInfo> = spec
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
                        EndpointInfo {
                            method: method.to_string(),
                            path: path.clone(),
                            description: op
                                .description
                                .as_deref()
                                .or(op.summary.as_deref())
                                .unwrap_or("")
                                .to_string(),
                            auth_required: !is_public,
                        }
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    Json(HelpResponseWithDesc {
        service: "e-fees-scope".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Clause library, scope assembly, and corpus retrieval microservice for emittiv fee proposals.".to_string(),
        endpoints,
    })
}
