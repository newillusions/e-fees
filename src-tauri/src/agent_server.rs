//! Agent API HTTP Server
//!
//! Provides a local REST API for AI agents and external tools to interact
//! with E-Fees data programmatically. Runs on a configurable port (default 3100)
//! alongside the Tauri desktop application.
//!
//! ## Endpoints
//!
//! - `GET  /api/health`          - Health check with version and DB status
//! - `GET  /api/stats`           - Dashboard statistics (entity counts)
//! - `GET  /api/projects`        - List all projects
//! - `GET  /api/projects/:id`    - Get project by ID
//! - `POST /api/projects`        - Create a new project
//! - `GET  /api/fees`            - List all fees (optional ?project_id= filter)
//! - `GET  /api/fees/:id`        - Get fee by ID
//! - `POST /api/fees`            - Create a new fee
//! - `GET  /api/companies`       - List all companies
//! - `POST /api/companies`       - Create a new company
//! - `GET  /api/contacts`        - List all contacts (optional ?company_id= filter)
//! - `GET  /api/contacts/:id`    - Get contact by ID
//! - `POST /api/contacts`        - Create a new contact
//! - `POST /api/fees/:id/export` - Export fee as .xlsx file
//! - `GET  /api/logs/stream`     - SSE real-time log stream (public)
//! - `GET  /api/logs/level`      - Get current log level
//! - `POST /api/logs/level`      - Set log level at runtime

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json, Response,
    },
    routing::{get, post},
    Router,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio::sync::RwLock;

use crate::db::{DatabaseManager, CompanyCreate, ContactCreate, FeeCreate};
use crate::db::types::{record_key_string, record_id_string};
use crate::commands::AppState;

// ============================================================================
// SERVER STATE
// ============================================================================

/// Shared state for the agent API server.
#[derive(Clone)]
pub struct AgentState {
    pub db: AppState,
    pub start_time: Instant,
    pub api_key: Option<String>,
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub db_connected: bool,
    pub uptime_seconds: u64,
    pub log_file: String,
    pub log_level: String,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_projects: usize,
    pub total_companies: usize,
    pub total_contacts: usize,
    pub total_fees: usize,
    pub active_fees: usize,
}

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub country: String,
    pub city: String,
    pub area: String,
    pub number: String,
    pub folder: String,
}

#[derive(Serialize)]
pub struct FeeResponse {
    pub id: String,
    pub name: String,
    pub number: String,
    pub rev: i64,
    pub status: String,
    pub project_id: String,
    pub company_id: String,
    pub total_fee: f64,
    pub currency: String,
}

#[derive(Serialize)]
pub struct CompanyResponse {
    pub id: String,
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
}

#[derive(Serialize)]
pub struct ContactResponse {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company_id: String,
}

#[derive(Serialize)]
pub struct ListResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct ExportResponse {
    pub path: String,
    pub filename: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ============================================================================
// REQUEST TYPES
// ============================================================================

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub country: String,
    pub city: String,
    pub area: String,
    pub number: String,
    pub folder: String,
}

#[derive(Deserialize)]
pub struct CreateCompanyRequest {
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
    #[serde(default)]
    pub reg_no: Option<String>,
    #[serde(default)]
    pub tax_no: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateFeeRequest {
    pub name: String,
    pub number: String,
    pub project_id: String,
    pub company_id: String,
    pub contact_id: String,
    #[serde(default = "default_fee_status")]
    pub status: String,
    #[serde(default)]
    pub issue_date: Option<String>,
    #[serde(default)]
    pub activity: Option<String>,
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub staff_name: Option<String>,
    #[serde(default)]
    pub staff_email: Option<String>,
    #[serde(default)]
    pub staff_phone: Option<String>,
    #[serde(default)]
    pub staff_position: Option<String>,
    #[serde(default)]
    pub strap_line: Option<String>,
}

fn default_fee_status() -> String {
    "Draft".to_string()
}

#[derive(Deserialize)]
pub struct CreateContactRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct ListContactsQuery {
    pub company_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ListFeesQuery {
    pub project_id: Option<String>,
}

// ============================================================================
// ROUTE HANDLERS
// ============================================================================

async fn health_handler(State(state): State<AgentState>) -> Json<HealthResponse> {
    let db_connected = {
        let manager = state.db.read().await;
        manager.check_connection().await
    };
    let uptime = state.start_time.elapsed().as_secs();

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        db_connected,
        uptime_seconds: uptime,
        log_file: get_log_file_path(),
        log_level: log::max_level().to_string().to_lowercase(),
    })
}

async fn stats_handler(
    State(state): State<AgentState>,
) -> Result<Json<StatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;
    let counts = manager.get_entity_counts().await.map_err(|e| {
        error!("Agent API: Failed to get stats: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get stats: {}", e),
            }),
        )
    })?;

    Ok(Json(StatsResponse {
        total_projects: counts.total_projects,
        total_companies: counts.total_companies,
        total_contacts: counts.total_contacts,
        total_fees: counts.total_fees,
        active_fees: counts.active_fees,
    }))
}

async fn list_projects_handler(
    State(state): State<AgentState>,
) -> Result<Json<ListResponse<ProjectResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;
    let projects = manager.get_projects().await.map_err(|e| {
        error!("Agent API: Failed to list projects: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to list projects: {}", e),
            }),
        )
    })?;

    let items: Vec<ProjectResponse> = projects
        .into_iter()
        .map(|p| ProjectResponse {
            id: p.id.map(|t| record_id_string(&t)).unwrap_or_default(),
            name: p.name,
            name_short: p.name_short,
            status: p.status,
            country: p.country,
            city: p.city,
            area: p.area,
            number: p.number.id,
            folder: p.folder,
        })
        .collect();

    let total = items.len();
    Ok(Json(ListResponse { items, total }))
}

async fn get_project_handler(
    State(state): State<AgentState>,
    Path(id): Path<String>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;
    let project = manager.get_project_by_id(&id).await.map_err(|e| {
        error!("Agent API: Failed to get project: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get project: {}", e),
            }),
        )
    })?;

    match project {
        Some(p) => Ok(Json(ProjectResponse {
            id: p.id.map(|t| record_id_string(&t)).unwrap_or_default(),
            name: p.name,
            name_short: p.name_short,
            status: p.status,
            country: p.country,
            city: p.city,
            area: p.area,
            number: p.number.id,
            folder: p.folder,
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
            }),
        )),
    }
}

async fn create_project_handler(
    State(state): State<AgentState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), (StatusCode, Json<ErrorResponse>)> {
    use crate::db::Project;
    use crate::db::types::{ProjectNumber, TimeStamps};

    // Parse number string (e.g., "26-97101") into ProjectNumber
    let number = parse_project_number(&req.number).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )
    })?;

    let project = Project {
        id: None,
        name: req.name,
        name_short: req.name_short,
        status: req.status,
        area: req.area,
        city: req.city,
        country: req.country,
        folder: req.folder,
        number,
        time: TimeStamps {
            created_at: surrealdb_types::Datetime::now(),
            updated_at: surrealdb_types::Datetime::now(),
        },
    };

    let manager = state.db.read().await;
    let created = manager.create_project(project).await.map_err(|e| {
        error!("Agent API: Failed to create project: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create project: {}", e),
            }),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(ProjectResponse {
            id: created.id.map(|t| record_id_string(&t)).unwrap_or_default(),
            name: created.name,
            name_short: created.name_short,
            status: created.status,
            country: created.country,
            city: created.city,
            area: created.area,
            number: created.number.id,
            folder: created.folder,
        }),
    ))
}

async fn list_fees_handler(
    State(state): State<AgentState>,
    Query(params): Query<ListFeesQuery>,
) -> Result<Json<ListResponse<FeeResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;
    let fees = manager.get_fees().await.map_err(|e| {
        error!("Agent API: Failed to list fees: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to list fees: {}", e),
            }),
        )
    })?;

    let items: Vec<FeeResponse> = fees
        .into_iter()
        .filter(|f| {
            if let Some(ref filter_project) = params.project_id {
                let fee_project_id = record_id_string(&f.project_id);
                &fee_project_id == filter_project
            } else {
                true
            }
        })
        .map(|f| {
            let pricing = f.pricing_typed();
            let total_fee = pricing.as_ref().map(|p| p.config.quoted_fee).unwrap_or(0.0);
            let currency = pricing.as_ref().map(|p| p.config.currency.clone()).unwrap_or_else(|| "AED".to_string());
            FeeResponse {
                id: f.id.map(|t| record_id_string(&t)).unwrap_or_default(),
                name: f.name,
                number: f.number,
                rev: f.rev,
                status: f.status,
                project_id: record_id_string(&f.project_id),
                company_id: record_id_string(&f.company_id),
                total_fee,
                currency,
            }
        })
        .collect();

    let total = items.len();
    Ok(Json(ListResponse { items, total }))
}

async fn get_fee_handler(
    State(state): State<AgentState>,
    Path(id): Path<String>,
) -> Result<Json<FeeResponse>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;

    // Parse the id — if it contains "fee:", strip the table prefix
    let record_id = id.strip_prefix("fee:").unwrap_or(&id).to_string();

    let fee = manager.get_fee_by_id(&record_id).await.map_err(|e| {
        error!("Agent API: Failed to get fee: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get fee: {}", e),
            }),
        )
    })?;

    match fee {
        Some(f) => {
            let pricing = f.pricing_typed();
            let total_fee = pricing.as_ref().map(|p| p.config.quoted_fee).unwrap_or(0.0);
            let currency = pricing.as_ref().map(|p| p.config.currency.clone()).unwrap_or_else(|| "AED".to_string());
            Ok(Json(FeeResponse {
                id: f.id.map(|t| record_id_string(&t)).unwrap_or_default(),
                name: f.name,
                number: f.number,
                rev: f.rev,
                status: f.status,
                project_id: record_id_string(&f.project_id),
                company_id: record_id_string(&f.company_id),
                total_fee,
                currency,
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Fee not found".to_string(),
            }),
        )),
    }
}

async fn create_fee_handler(
    State(state): State<AgentState>,
    Json(req): Json<CreateFeeRequest>,
) -> Result<(StatusCode, Json<FeeResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Strip table prefixes from IDs if present
    let project_id = req.project_id.strip_prefix("projects:").unwrap_or(&req.project_id).to_string();
    let company_id = req.company_id.strip_prefix("company:").unwrap_or(&req.company_id).to_string();
    let contact_id = req.contact_id.strip_prefix("contacts:").unwrap_or(&req.contact_id).to_string();

    let issue_date = req.issue_date.unwrap_or_else(|| {
        chrono::Utc::now().format("%y%m%d").to_string()
    });

    let fee_create = FeeCreate {
        name: req.name,
        number: req.number,
        rev: 1,
        status: req.status,
        issue_date,
        activity: req.activity.unwrap_or_else(|| "Design and Consultancy".to_string()),
        package: req.package.unwrap_or_else(|| "Lighting".to_string()),
        project_id,
        company_id,
        contact_id,
        staff_name: req.staff_name.unwrap_or_default(),
        staff_email: req.staff_email.unwrap_or_default(),
        staff_phone: req.staff_phone.unwrap_or_default(),
        staff_position: req.staff_position.unwrap_or_default(),
        strap_line: req.strap_line.unwrap_or_default(),
        revisions: vec![],
        pricing: None,
        post_contract_items: None,
        reimbursable_costs: None,
        payment_schedule: None,
        pricing_revisions: None,
        current_revision_number: None,
        current_release_number: None,
        import_source: None,
    };

    let manager = state.db.read().await;
    let created = manager.create_fee(fee_create).await.map_err(|e| {
        error!("Agent API: Failed to create fee: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create fee: {}", e),
            }),
        )
    })?;

    let pricing = created.pricing_typed();
    let total_fee = pricing.as_ref().map(|p| p.config.quoted_fee).unwrap_or(0.0);
    let currency = pricing.as_ref().map(|p| p.config.currency.clone()).unwrap_or_else(|| "AED".to_string());

    Ok((
        StatusCode::CREATED,
        Json(FeeResponse {
            id: created.id.map(|t| record_id_string(&t)).unwrap_or_default(),
            name: created.name,
            number: created.number,
            rev: created.rev,
            status: created.status,
            project_id: record_id_string(&created.project_id),
            company_id: record_id_string(&created.company_id),
            total_fee,
            currency,
        }),
    ))
}

async fn list_companies_handler(
    State(state): State<AgentState>,
) -> Result<Json<ListResponse<CompanyResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;
    let companies = manager.get_companies().await.map_err(|e| {
        error!("Agent API: Failed to list companies: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to list companies: {}", e),
            }),
        )
    })?;

    let items: Vec<CompanyResponse> = companies
        .into_iter()
        .map(|c| CompanyResponse {
            id: c.id.map(|t| record_id_string(&t)).unwrap_or_default(),
            name: c.name,
            name_short: c.name_short,
            abbreviation: c.abbreviation,
            city: c.city,
            country: c.country,
        })
        .collect();

    let total = items.len();
    Ok(Json(ListResponse { items, total }))
}

async fn create_company_handler(
    State(state): State<AgentState>,
    Json(req): Json<CreateCompanyRequest>,
) -> Result<(StatusCode, Json<CompanyResponse>), (StatusCode, Json<ErrorResponse>)> {
    let company = CompanyCreate {
        name: req.name,
        name_short: req.name_short,
        abbreviation: req.abbreviation,
        city: req.city,
        country: req.country,
        reg_no: req.reg_no,
        tax_no: req.tax_no,
    };

    let manager = state.db.read().await;
    let created = manager.create_company(company).await.map_err(|e| {
        error!("Agent API: Failed to create company: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create company: {}", e),
            }),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(CompanyResponse {
            id: created.id.map(|t| record_id_string(&t)).unwrap_or_default(),
            name: created.name,
            name_short: created.name_short,
            abbreviation: created.abbreviation,
            city: created.city,
            country: created.country,
        }),
    ))
}

async fn list_contacts_handler(
    State(state): State<AgentState>,
    Query(params): Query<ListContactsQuery>,
) -> Result<Json<ListResponse<ContactResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;
    let contacts = manager.get_contacts().await.map_err(|e| {
        error!("Agent API: Failed to list contacts: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to list contacts: {}", e),
            }),
        )
    })?;

    let items: Vec<ContactResponse> = contacts
        .into_iter()
        .filter(|c| {
            if let Some(ref filter_company) = params.company_id {
                if let Some(ref company) = c.company {
                    let company_id = record_id_string(&company);
                    &company_id == filter_company
                } else {
                    false
                }
            } else {
                true
            }
        })
        .map(|c| {
            let company_id = c.company.map(|t| record_id_string(&t)).unwrap_or_default();
            ContactResponse {
                id: c.id.map(|t| record_id_string(&t)).unwrap_or_default(),
                first_name: c.first_name.unwrap_or_default(),
                last_name: c.last_name.unwrap_or_default(),
                full_name: c.full_name.unwrap_or_default(),
                email: c.email.unwrap_or_default(),
                phone: c.phone.unwrap_or_default(),
                position: c.position.unwrap_or_default(),
                company_id,
            }
        })
        .collect();

    let total = items.len();
    Ok(Json(ListResponse { items, total }))
}

async fn get_contact_handler(
    State(state): State<AgentState>,
    Path(id): Path<String>,
) -> Result<Json<ContactResponse>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;

    let record_id = id.strip_prefix("contacts:").unwrap_or(&id).to_string();

    let contact = manager.get_contact_by_id(&record_id).await.map_err(|e| {
        error!("Agent API: Failed to get contact: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get contact: {}", e),
            }),
        )
    })?;

    match contact {
        Some(c) => {
            let company_id = c.company.map(|t| record_id_string(&t)).unwrap_or_default();
            Ok(Json(ContactResponse {
                id: c.id.map(|t| record_id_string(&t)).unwrap_or_default(),
                first_name: c.first_name.unwrap_or_default(),
                last_name: c.last_name.unwrap_or_default(),
                full_name: c.full_name.unwrap_or_default(),
                email: c.email.unwrap_or_default(),
                phone: c.phone.unwrap_or_default(),
                position: c.position.unwrap_or_default(),
                company_id,
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Contact not found".to_string(),
            }),
        )),
    }
}

async fn create_contact_handler(
    State(state): State<AgentState>,
    Json(req): Json<CreateContactRequest>,
) -> Result<(StatusCode, Json<ContactResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Strip table prefix from company_id if present
    let company = req.company_id.strip_prefix("company:").unwrap_or(&req.company_id).to_string();

    let contact = ContactCreate {
        first_name: req.first_name,
        last_name: req.last_name,
        email: req.email,
        phone: req.phone,
        position: req.position,
        company,
    };

    let manager = state.db.read().await;
    let created = manager.create_contact(contact).await.map_err(|e| {
        error!("Agent API: Failed to create contact: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create contact: {}", e),
            }),
        )
    })?;

    let company_id = created.company.map(|t| record_id_string(&t)).unwrap_or_default();

    Ok((
        StatusCode::CREATED,
        Json(ContactResponse {
            id: created.id.map(|t| record_id_string(&t)).unwrap_or_default(),
            first_name: created.first_name.unwrap_or_default(),
            last_name: created.last_name.unwrap_or_default(),
            full_name: created.full_name.unwrap_or_default(),
            email: created.email.unwrap_or_default(),
            phone: created.phone.unwrap_or_default(),
            position: created.position.unwrap_or_default(),
            company_id,
        }),
    ))
}

async fn export_fee_handler(
    State(state): State<AgentState>,
    Path(id): Path<String>,
) -> Result<Json<ExportResponse>, (StatusCode, Json<ErrorResponse>)> {
    let manager = state.db.read().await;

    let record_id = id.strip_prefix("fee:").unwrap_or(&id).to_string();

    let fee = manager.get_fee_by_id(&record_id).await.map_err(|e| {
        error!("Agent API: Failed to get fee for export: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get fee: {}", e),
            }),
        )
    })?;

    let fee = fee.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Fee not found".to_string(),
            }),
        )
    })?;

    let safe_number = fee.number.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");
    let filename = format!("fee-{}-rev{}.xlsx", safe_number, fee.rev);
    let output_path = std::env::temp_dir().join(&filename);

    let path = crate::excel_export::generate_fee_excel(&fee, &output_path).map_err(|e| {
        error!("Agent API: Excel generation failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Excel generation failed: {}", e),
            }),
        )
    })?;

    Ok(Json(ExportResponse { path, filename }))
}

// ============================================================================
// LOGGING ENDPOINTS
// ============================================================================

/// Resolve the log file path for the current platform.
fn get_log_file_path() -> String {
    if let Some(home) = dirs::home_dir() {
        if cfg!(target_os = "macos") {
            home.join("Library/Logs/com.emittiv.e-fees/E-Fees.log")
                .to_string_lossy()
                .to_string()
        } else if cfg!(target_os = "windows") {
            home.join("AppData/Local/com.emittiv.e-fees/logs/E-Fees.log")
                .to_string_lossy()
                .to_string()
        } else {
            home.join(".local/share/com.emittiv.e-fees/logs/E-Fees.log")
                .to_string_lossy()
                .to_string()
        }
    } else {
        "unknown".to_string()
    }
}

/// SSE endpoint that tails the log file in real-time.
///
/// Seeks to end of file and streams new lines as SSE events.
/// Public endpoint (no auth) for easy `curl -N` access.
async fn log_stream_handler() -> Result<
    Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let log_path = get_log_file_path();
    let path = std::path::PathBuf::from(&log_path);

    let file = tokio::fs::File::open(&path).await.map_err(|e| {
        error!("Failed to open log file {}: {}", log_path, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to open log file: {}", e),
            }),
        )
    })?;

    let stream = async_stream::stream! {
        let path_clone = path.clone();
        let mut pos = {
            let meta = tokio::fs::metadata(&path_clone).await;
            meta.map(|m| m.len()).unwrap_or(0)
        };
        let mut line = String::new();
        loop {
            // Check if file has grown since last read
            let current_len = tokio::fs::metadata(&path_clone).await
                .map(|m| m.len()).unwrap_or(0);
            if current_len > pos {
                // New data available — open, seek to last position, read lines
                if let Ok(mut file) = tokio::fs::File::open(&path_clone).await {
                    if file.seek(std::io::SeekFrom::Start(pos)).await.is_ok() {
                        let mut reader = BufReader::new(&mut file);
                        loop {
                            line.clear();
                            match reader.read_line(&mut line).await {
                                Ok(0) => break, // caught up
                                Ok(n) => {
                                    pos += n as u64;
                                    let trimmed = line.trim().to_string();
                                    if !trimmed.is_empty() {
                                        yield Ok(Event::default().data(trimmed));
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[derive(Deserialize)]
struct SetLogLevelRequest {
    level: String,
}

#[derive(Serialize)]
struct LogLevelResponse {
    level: String,
}

/// GET /api/logs/level — return current log level.
async fn get_log_level_handler() -> Json<LogLevelResponse> {
    Json(LogLevelResponse {
        level: log::max_level().to_string().to_lowercase(),
    })
}

/// POST /api/logs/level — set log level at runtime.
async fn set_log_level_handler(
    Json(req): Json<SetLogLevelRequest>,
) -> Result<Json<LogLevelResponse>, (StatusCode, Json<ErrorResponse>)> {
    let filter = match req.level.to_lowercase().as_str() {
        "off" => log::LevelFilter::Off,
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown log level: {}", req.level),
                }),
            ))
        }
    };
    log::set_max_level(filter);
    info!("Log level changed via API to: {}", req.level);
    Ok(Json(LogLevelResponse {
        level: req.level.to_lowercase(),
    }))
}

// ============================================================================
// HELPERS
// ============================================================================

/// Parse a project number string like "26-97101" into a ProjectNumber struct.
fn parse_project_number(number_str: &str) -> Result<crate::db::types::ProjectNumber, String> {
    let parts: Vec<&str> = number_str.split('-').collect();
    if parts.len() != 2 || parts[0].len() != 2 || parts[1].len() != 5 {
        return Err(format!("Invalid project number format: '{}'. Expected YY-CCCNN (e.g., 26-97101)", number_str));
    }

    let year: i64 = parts[0].parse().map_err(|_| format!("Invalid year in project number: '{}'", parts[0]))?;
    let country: i64 = parts[1][..3].parse().map_err(|_| format!("Invalid country code in project number: '{}'", &parts[1][..3]))?;
    let seq: i64 = parts[1][3..].parse().map_err(|_| format!("Invalid sequence in project number: '{}'", &parts[1][3..]))?;

    Ok(crate::db::types::ProjectNumber {
        year,
        country,
        seq,
        id: number_str.to_string(),
    })
}

// ============================================================================
// AUTH MIDDLEWARE
// ============================================================================

/// Middleware that validates the X-API-Key header against EFEES_AGENT_API_KEY.
///
/// If no API key is configured (api_key is None), all requests are allowed.
/// The /api/health endpoint is exempt (handled on a separate router layer).
async fn api_key_auth(
    State(state): State<AgentState>,
    request: Request,
    next: Next,
) -> Response {
    let api_key = match &state.api_key {
        Some(key) => key,
        None => return next.run(request).await,
    };

    let provided = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match provided {
        Some(value) if value == api_key => next.run(request).await,
        _ => {
            let body = Json(ErrorResponse {
                error: "Unauthorized: missing or invalid X-API-Key".to_string(),
            });
            (StatusCode::UNAUTHORIZED, body).into_response()
        }
    }
}

// ============================================================================
// SERVER STARTUP
// ============================================================================

/// Build the axum router with all agent API routes.
pub fn build_router(state: AgentState) -> Router {
    // Public endpoints — exempt from auth
    let public = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/logs/stream", get(log_stream_handler))
        .with_state(state.clone());

    // All other endpoints — protected by API key middleware
    let protected = Router::new()
        .route("/api/stats", get(stats_handler))
        .route("/api/projects", get(list_projects_handler).post(create_project_handler))
        .route("/api/projects/{id}", get(get_project_handler))
        .route("/api/fees", get(list_fees_handler).post(create_fee_handler))
        .route("/api/fees/{id}", get(get_fee_handler))
        .route("/api/companies", get(list_companies_handler).post(create_company_handler))
        .route("/api/contacts", get(list_contacts_handler).post(create_contact_handler))
        .route("/api/contacts/{id}", get(get_contact_handler))
        .route("/api/fees/{id}/export", post(export_fee_handler))
        .route("/api/logs/level", get(get_log_level_handler).post(set_log_level_handler))
        .route_layer(middleware::from_fn_with_state(state.clone(), api_key_auth))
        .with_state(state);

    public.merge(protected)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_state(api_key: Option<&str>) -> AgentState {
        let db = Arc::new(RwLock::new(crate::db::DatabaseManager::new_unconfigured()));
        AgentState {
            db,
            start_time: Instant::now(),
            api_key: api_key.map(|s| s.to_string()),
        }
    }

    fn request(method: &str, uri: &str, api_key: Option<&str>) -> axum::http::Request<Body> {
        let mut builder = axum::http::Request::builder()
            .method(method)
            .uri(uri);
        if let Some(key) = api_key {
            builder = builder.header("X-API-Key", key);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn auth_rejects_missing_key() {
        let app = build_router(test_state(Some("secret-key")));
        let resp = app.oneshot(request("GET", "/api/stats", None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_rejects_wrong_key() {
        let app = build_router(test_state(Some("secret-key")));
        let resp = app.oneshot(request("GET", "/api/stats", Some("wrong-key"))).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_allows_correct_key() {
        let app = build_router(test_state(Some("secret-key")));
        let resp = app.oneshot(request("GET", "/api/stats", Some("secret-key"))).await.unwrap();
        // Should pass auth — will get 500 (no DB) rather than 401
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn health_exempt_from_auth() {
        let app = build_router(test_state(Some("secret-key")));
        let resp = app.oneshot(request("GET", "/api/health", None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn no_key_configured_allows_all() {
        let app = build_router(test_state(None));
        let resp = app.oneshot(request("GET", "/api/stats", None)).await.unwrap();
        // No API key configured — should pass auth (will get 500 from DB, not 401)
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_401_body_is_json_error() {
        let app = build_router(test_state(Some("secret-key")));
        let resp = app.oneshot(request("GET", "/api/stats", None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(parsed.get("error").is_some());
        assert!(parsed["error"].as_str().unwrap().contains("Unauthorized"));
    }

    fn json_request(method: &str, uri: &str, body: &str) -> axum::http::Request<Body> {
        axum::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    // ==================== Contact endpoint tests ====================

    #[tokio::test]
    async fn contacts_list_returns_500_without_db() {
        let app = build_router(test_state(None));
        let resp = app.oneshot(request("GET", "/api/contacts", None)).await.unwrap();
        // No DB connected — should get 500, not a panic
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(parsed.get("error").is_some());
    }

    #[tokio::test]
    async fn contacts_list_requires_auth() {
        let app = build_router(test_state(Some("secret-key")));
        let resp = app.oneshot(request("GET", "/api/contacts", None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn contacts_get_returns_500_without_db() {
        let app = build_router(test_state(None));
        let resp = app.oneshot(request("GET", "/api/contacts/abc123", None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn contacts_get_requires_auth() {
        let app = build_router(test_state(Some("secret-key")));
        let resp = app.oneshot(request("GET", "/api/contacts/abc123", None)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn contacts_create_rejects_missing_fields() {
        let app = build_router(test_state(None));
        // Missing required fields — should get 422 (Unprocessable Entity)
        let resp = app.oneshot(json_request("POST", "/api/contacts", r#"{"first_name":"John"}"#)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn contacts_create_returns_500_without_db() {
        let app = build_router(test_state(None));
        let body = r#"{
            "first_name": "Ahmed",
            "last_name": "Al Rashid",
            "email": "ahmed@example.com",
            "phone": "+971501234567",
            "position": "Project Manager",
            "company_id": "company:xyz789"
        }"#;
        let resp = app.oneshot(json_request("POST", "/api/contacts", body)).await.unwrap();
        // Valid payload but no DB — should get 500
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ==================== Fee export endpoint tests ====================

    #[tokio::test]
    async fn export_fee_returns_500_without_db() {
        let app = build_router(test_state(None));
        let resp = app.oneshot(json_request("POST", "/api/fees/test123/export", "")).await.unwrap();
        // No DB connected — should get 500
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(parsed.get("error").is_some());
    }

    #[tokio::test]
    async fn export_fee_requires_auth() {
        let app = build_router(test_state(Some("secret-key")));
        let resp = app.oneshot(json_request("POST", "/api/fees/test123/export", "")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn export_fee_strips_prefix() {
        // Both "fee:abc" and "abc" should be accepted (both hit DB, get 500 without DB)
        let app = build_router(test_state(None));
        let resp = app.oneshot(json_request("POST", "/api/fees/fee:abc/export", "")).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

/// Start the agent API HTTP server.
///
/// Spawns a tokio task that listens on the specified port.
/// Returns immediately — the server runs in the background.
///
/// # Environment Variables
/// - `EFEES_AGENT_BIND`: Bind address (default: `0.0.0.0:<port>`)
/// - `EFEES_AGENT_API_KEY`: If set, all endpoints (except /api/health) require
///   this value in the `X-API-Key` header. If unset, no auth is enforced.
pub async fn start_agent_server(db_state: AppState, port: u16) {
    let api_key = std::env::var("EFEES_AGENT_API_KEY").ok();
    if api_key.is_none() {
        warn!("Agent API: EFEES_AGENT_API_KEY is not set — all requests will be allowed without authentication");
    }

    let state = AgentState {
        db: db_state,
        start_time: Instant::now(),
        api_key,
    };

    let app = build_router(state);

    let addr = std::env::var("EFEES_AGENT_BIND")
        .unwrap_or_else(|_| format!("0.0.0.0:{}", port));
    info!("Agent API server starting on http://{}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Agent API: Failed to bind to {}: {}", addr, e);
            warn!("Agent API server will not be available this session");
            return;
        }
    };

    info!("Agent API server listening on http://{}", addr);

    if let Err(e) = axum::serve(listener, app).await {
        error!("Agent API server error: {}", e);
    }
}
