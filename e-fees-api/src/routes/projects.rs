//! Project route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use chrono::Datelike;
use e_fees_core::models::{
    record_id_string, record_key_string, Company, Fee, NewProjectApi, Project, ProjectNumber,
};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::error::ApiError;
use crate::pagination::{db_paginate_filtered, paginated_json_raw, FilterClause};
use crate::schemas;
use crate::validation::{require_non_empty, validate_id, validate_status, PROJECT_STATUSES};
use crate::AppState;

/// Combined query parameters for project listing (pagination + filters).
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ProjectListParams {
    /// Page number (1-indexed, default: 1).
    pub page: Option<u64>,
    /// Items per page (default: 50, max: 100).
    pub page_size: Option<u64>,
    /// Filter by exact project status (e.g. "Lead", "RFP", "Awarded").
    pub status: Option<String>,
    /// Case-insensitive substring search across name, name_short, and number.id.
    pub search: Option<String>,
}

/// Query parameters for `GET /projects/{id}`.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ProjectDetailParams {
    /// Comma-separated expansion list. Currently supported: `client` — embeds
    /// the latest non-superseded fee's company as a nested
    /// `client: { id, name } | null` object on the response.
    pub include: Option<String>,
}

/// List projects with pagination and optional status filter.
#[utoipa::path(
    get,
    path = "/projects",
    tag = "Projects",
    params(ProjectListParams),
    responses(
        (status = 200, description = "Paginated list of projects", body = schemas::PaginatedResponse<schemas::ProjectResponse>),
        (status = 400, description = "Invalid status filter", body = schemas::ErrorResponse),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_projects(
    State(state): State<Arc<AppState>>,
    params: Query<ProjectListParams>,
) -> Result<Json<Value>, ApiError> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50);

    let mut clauses = Vec::new();
    let mut binds: Vec<(String, serde_json::Value)> = Vec::new();

    if let Some(ref status) = params.status {
        validate_status(status, PROJECT_STATUSES, "project")?;
        clauses.push("status = $filter_status".to_string());
        binds.push(("filter_status".into(), json!(status)));
    }

    if let Some(ref search) = params.search {
        clauses.push(
            "(string::lowercase(name) CONTAINS string::lowercase($search) \
             OR string::lowercase(name_short) CONTAINS string::lowercase($search) \
             OR number.id CONTAINS $search)"
                .to_string(),
        );
        binds.push(("search".into(), json!(search)));
    }

    let filter = if clauses.is_empty() {
        None
    } else {
        Some(FilterClause {
            clause: clauses.join(" AND "),
            binds,
        })
    };

    let (projects, total): (Vec<Project>, u64) =
        db_paginate_filtered(&state.db, "projects", page, page_size, filter).await?;

    let data: Vec<Value> = projects.iter().map(project_to_json).collect();

    Ok(Json(paginated_json_raw(data, total, page, page_size)))
}

/// Get a single project by ID.
#[utoipa::path(
    get,
    path = "/projects/{id}",
    tag = "Projects",
    params(
        ("id" = String, Path, description = "Project record key (e.g. 25_97105)"),
        ProjectDetailParams,
    ),
    responses(
        (status = 200, description = "Project found", body = schemas::SingleResponse<schemas::ProjectResponse>),
        (status = 404, description = "Project not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<ProjectDetailParams>,
) -> Result<Json<Value>, ApiError> {
    let project: Option<Project> = state.db.select(("projects", &*id)).await?;

    let project = match project {
        Some(p) => p,
        None => return Err(ApiError::not_found("Project", &id)),
    };

    let mut data = project_to_json(&project);

    if has_include(&params.include, "client") {
        data["client"] = resolve_client_for_project(&state.db, &id).await?;
    }

    Ok(Json(json!({ "data": data })))
}

/// Parse a comma-separated include list and check if `needle` is present.
fn has_include(include: &Option<String>, needle: &str) -> bool {
    include
        .as_deref()
        .map(|s| s.split(',').any(|t| t.trim() == needle))
        .unwrap_or(false)
}

/// Resolve the "client" of a project as the company of the latest
/// non-superseded fee. Returns `{ id, name }` JSON object, or `null` if the
/// project has no qualifying fees (or the linked company is missing).
async fn resolve_client_for_project(
    db: &Surreal<Client>,
    project_key: &str,
) -> Result<Value, ApiError> {
    let mut result = db
        .query(
            "SELECT * FROM fee \
             WHERE project_id = type::record('projects', $pkey) \
               AND status != 'Superseded' \
             ORDER BY time.created DESC LIMIT 1",
        )
        .bind(("pkey", project_key.to_string()))
        .await?;
    let fees: Vec<Fee> = result.take(0)?;

    let Some(fee) = fees.into_iter().next() else {
        return Ok(Value::Null);
    };

    let company_key = record_key_string(&fee.company_id.key);
    let company: Option<Company> = db.select(("company", &*company_key)).await?;

    Ok(match company {
        Some(c) => json!({
            "id": record_id_string(&fee.company_id),
            "name": c.name,
        }),
        None => Value::Null,
    })
}

/// Query parameters for the next-number preview endpoint.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct NextNumberParams {
    /// Country name (e.g. "UAE", "Saudi Arabia").
    pub country: String,
    /// Two-digit year (e.g. 26). Defaults to current year if omitted.
    pub year: Option<u8>,
}

/// Preview the next available project number for a country/year.
#[utoipa::path(
    get,
    path = "/projects/next-number",
    tag = "Projects",
    params(NextNumberParams),
    responses(
        (status = 200, description = "Next available project number", body = schemas::NextNumberResponse),
        (status = 400, description = "Country not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn get_next_number(
    State(state): State<Arc<AppState>>,
    params: Query<NextNumberParams>,
) -> Result<Json<Value>, ApiError> {
    let year = params
        .year
        .unwrap_or_else(|| (chrono::Utc::now().year() % 100) as u8);

    let (_canonical_name, country_code) = resolve_country(&state.db, &params.country).await?;
    let seq = next_sequence(&state.db, country_code, year).await?;
    let number = format!("{:02}-{}{:02}", year, country_code, seq);

    Ok(Json(json!({
        "number": number,
        "year": year,
        "country_code": country_code,
        "seq": seq,
    })))
}

/// Create a new project. If `number` is omitted, auto-assigns based on country + year.
#[utoipa::path(
    post,
    path = "/projects",
    tag = "Projects",
    request_body = NewProjectApi,
    responses(
        (status = 200, description = "Project created", body = schemas::SingleResponse<schemas::ProjectResponse>),
        (status = 400, description = "Validation error", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NewProjectApi>,
) -> Result<Json<Value>, ApiError> {
    require_non_empty(&body.name, "name")?;
    validate_status(&body.status, PROJECT_STATUSES, "project")?;

    // Normalize the country name via the DB stored function before any path.
    // dial_code is also returned here for use in auto-numbering (None branch below).
    let (canonical_country, dial_code) = resolve_country(&state.db, &body.country).await?;

    let number = match body.number {
        Some(n) => n,
        None => {
            let year = (chrono::Utc::now().year() % 100) as u8;
            let seq = next_sequence(&state.db, dial_code, year).await?;
            let id = format!("{:02}-{}{:02}", year, dial_code, seq);
            ProjectNumber {
                year: year as i64,
                country: dial_code as i64,
                seq: seq as i64,
                id,
            }
        }
    };

    let record_key = number.id.replace('-', "_");

    let query = format!(
        "CREATE projects:{key} SET name = $name, name_short = $name_short, \
         status = $status, area = $area, city = $city, country = $country, \
         folder = $folder, number = $number, \
         time = {{ created_at: time::now(), updated_at: time::now() }}",
        key = record_key
    );
    let mut response = state
        .db
        .query(&query)
        .bind(("name", body.name))
        .bind(("name_short", body.name_short))
        .bind(("status", body.status))
        .bind(("area", body.area))
        .bind(("city", body.city))
        .bind(("country", canonical_country))
        .bind(("folder", body.folder))
        .bind((
            "number",
            json!({
                "year": number.year,
                "country": number.country,
                "seq": number.seq,
                "id": number.id,
            }),
        ))
        .await?;
    let created: Option<Project> = response.take(0)?;

    match created {
        Some(p) => Ok(Json(json!({ "data": project_to_json(&p) }))),
        None => Err(ApiError::bad_request("Failed to create project")),
    }
}

/*
 * lookup_dial_code — replaced by resolve_country (uses fn::resolve_country stored function).
 * Kept for reference only; do not use.
 *
 * async fn lookup_dial_code(db: &Surreal<Client>, country_name: &str) -> Result<u16, ApiError> {
 *     if country_name.is_empty() || country_name.len() > 100 {
 *         return Err(ApiError::bad_request("Invalid country name"));
 *     }
 *     let mut response = db
 *         .query(
 *             "SELECT dial_code FROM country \
 *              WHERE name = $name \
 *              OR string::lowercase(string::replace(name, '.', '')) \
 *                 = string::lowercase(string::replace($name, '.', '')) \
 *              LIMIT 1",
 *         )
 *         .bind(("name", country_name.to_string()))
 *         .await?;
 *     let results: Vec<serde_json::Value> = response.take(0)?;
 *     let dial_code = results
 *         .first()
 *         .and_then(|r| r.get("dial_code").and_then(|v| v.as_u64()))
 *         .ok_or_else(|| ApiError::bad_request(format!("Country not found: '{}'", country_name)))?;
 *     u16::try_from(dial_code).map_err(|_| {
 *         ApiError::bad_request(format!("Dial code {} out of range for '{}'", dial_code, country_name))
 *     })
 * }
 */

/// Resolve a country input string via the DB stored function `fn::resolve_country`.
///
/// Returns `(canonical_name, dial_code)`. Returns a 400 error if the country is
/// not found in the reference table.
pub(crate) async fn resolve_country(
    db: &Surreal<Client>,
    input: &str,
) -> Result<(String, u16), ApiError> {
    if input.is_empty() || input.len() > 100 {
        return Err(ApiError::bad_request("Invalid country name"));
    }

    let mut response = db
        .query("RETURN fn::resolve_country($input);")
        .bind(("input", input.to_string()))
        .await?;

    let result: Option<serde_json::Value> = response.take(0)?;

    match result {
        None => Err(ApiError::bad_request(format!(
            "Country not found: '{}'. Please provide a valid country name.",
            input
        ))),
        Some(v) if v.is_null() => Err(ApiError::bad_request(format!(
            "Country not found: '{}'. Please provide a valid country name.",
            input
        ))),
        Some(v) => {
            let name = v
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| ApiError::bad_request("resolve_country returned invalid data"))?
                .to_string();

            let dial_code_raw = v.get("dial_code").and_then(|d| d.as_u64()).ok_or_else(|| {
                ApiError::bad_request("resolve_country returned invalid dial_code")
            })?;

            let dial_code = u16::try_from(dial_code_raw).map_err(|_| {
                ApiError::bad_request(format!(
                    "Dial code {} out of range for '{}'",
                    dial_code_raw, input
                ))
            })?;

            Ok((name, dial_code))
        }
    }
}

/// Find the next available sequence number for a country/year combo.
async fn next_sequence(db: &Surreal<Client>, country_code: u16, year: u8) -> Result<u8, ApiError> {
    let mut response = db
        .query(
            "SELECT number.seq FROM projects \
             WHERE number.year = $year AND number.country = $country \
             AND number.seq >= 1 AND number.seq <= 99 \
             ORDER BY number.seq DESC LIMIT 1",
        )
        .bind(("year", year as i64))
        .bind(("country", country_code as i64))
        .await?;
    let results: Vec<serde_json::Value> = response.take(0)?;

    let next = results
        .first()
        .and_then(|r| {
            r.get("number")
                .and_then(|n| n.get("seq"))
                .and_then(|s| s.as_u64())
        })
        .map(|seq| seq + 1)
        .unwrap_or(1);

    if next > 99 {
        return Err(ApiError::bad_request(format!(
            "Maximum 99 projects per year/country reached (year={}, country={})",
            year, country_code
        )));
    }

    Ok(next as u8)
}

/// Update an existing project (merge).
#[utoipa::path(
    put,
    path = "/projects/{id}",
    tag = "Projects",
    params(("id" = String, Path, description = "Project record key")),
    request_body = Value,
    responses(
        (status = 200, description = "Project updated", body = schemas::SingleResponse<schemas::ProjectResponse>),
        (status = 404, description = "Project not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    validate_id(&id)?;

    if let Some(status) = body.get("status").and_then(|s| s.as_str()) {
        validate_status(status, PROJECT_STATUSES, "project")?;
    }

    let query = format!(
        "UPDATE projects:{id} MERGE $data RETURN NONE; \
         UPDATE projects:{id} SET time.updated_at = time::now() RETURN NONE; \
         SELECT * FROM projects:{id};",
        id = id
    );
    let mut response = state.db.query(&query).bind(("data", body)).await?;
    let updated: Option<Project> = response.take(2)?;

    match updated {
        Some(p) => Ok(Json(json!({ "data": project_to_json(&p) }))),
        None => Err(ApiError::not_found("Project", &id)),
    }
}

/// Delete a project by ID.
#[utoipa::path(
    delete,
    path = "/projects/{id}",
    tag = "Projects",
    params(("id" = String, Path, description = "Project record key")),
    responses(
        (status = 200, description = "Project deleted", body = schemas::DeleteResponse),
        (status = 404, description = "Project not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_id(&id)?;

    let deleted: Option<Project> = state.db.delete(("projects", &*id)).await?;

    match deleted {
        Some(_) => Ok(Json(
            json!({ "deleted": true, "id": format!("projects:{}", id) }),
        )),
        None => Err(ApiError::not_found("Project", &id)),
    }
}

/// Query parameters for `GET /projects/typeahead`.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TypeaheadParams {
    /// Case-insensitive substring matched against project number, name, and short name.
    pub q: Option<String>,
    /// Maximum results to return (default: 20, max: 100).
    pub limit: Option<u64>,
}

/// Type-ahead search returning a lightweight project array for dropdowns.
///
/// Returns `[{ number, name, client }]` where `client` is `null` or
/// `{ id, name }` (the company of the project's latest non-superseded fee).
/// Matches `q` against project number, name, and short name (case-insensitive
/// substring). Default limit 20, max 100.
#[utoipa::path(
    get,
    path = "/projects/typeahead",
    tag = "Projects",
    params(TypeaheadParams),
    responses(
        (status = 200, description = "Matching projects for type-ahead"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn typeahead_projects(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TypeaheadParams>,
) -> Result<Json<Value>, ApiError> {
    let q = params.q.as_deref().unwrap_or("").to_string();
    let limit = params.limit.unwrap_or(20).min(100);

    let query = format!(
        "SELECT * FROM projects \
         WHERE (string::lowercase(name) CONTAINS string::lowercase($q) \
                OR string::lowercase(name_short) CONTAINS string::lowercase($q) \
                OR number.id CONTAINS $q) \
         LIMIT {limit}",
        limit = limit
    );

    let mut response = state.db.query(&query).bind(("q", q)).await?;
    let projects: Vec<Project> = response.take(0)?;

    let mut items = Vec::with_capacity(projects.len());
    for p in &projects {
        let key = p
            .id
            .as_ref()
            .map(|id| record_key_string(&id.key))
            .unwrap_or_default();
        let client = resolve_client_for_project(&state.db, &key).await?;
        items.push(json!({
            "number": p.number.id,
            "name": p.name,
            "client": client,
        }));
    }

    Ok(Json(Value::Array(items)))
}

/// Convert a Project to a JSON value for API response.
fn project_to_json(p: &Project) -> Value {
    json!({
        "id": p.id.as_ref().map(|id| record_id_string(id)).unwrap_or_default(),
        "name": p.name,
        "name_short": p.name_short,
        "status": p.status,
        "country": p.country,
        "city": p.city,
        "area": p.area,
        "folder": p.folder,
        "number": p.number.id,
    })
}
