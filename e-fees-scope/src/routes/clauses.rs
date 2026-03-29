//! Clause library CRUD route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use e_fees_core::models::{dbvalue_to_json, json_to_dbvalue, record_key_string};

use crate::error::ApiError;
use crate::models::{Clause, NewClause, UpdateClause};
use crate::AppState;

/// Query parameters for listing clauses.
#[derive(Debug, Deserialize)]
pub struct ClauseListParams {
    /// Filter by category (exact match).
    pub category: Option<String>,
    /// Filter by status (default: "active").
    pub status: Option<String>,
    /// Filter by tag (clauses containing this tag).
    pub tag: Option<String>,
}

/// Convert a Clause (SurrealDB native types) to a plain JSON Value.
fn clause_to_json(c: &Clause) -> Value {
    let mut obj = json!({
        "id": record_key_string(&c.id.key),
        "category": c.category,
        "title": c.title,
        "body": c.body,
        "sort_order": c.sort_order,
        "is_default": c.is_default,
        "status": c.status,
        "version": c.version,
        "created_at": c.created_at.to_string(),
        "updated_at": c.updated_at.to_string(),
    });

    if let Some(ref sub) = c.subcategory {
        obj["subcategory"] = json!(sub);
    }

    if let Some(ref conds) = c.conditions {
        obj["conditions"] = dbvalue_to_json(conds);
    }

    if let Some(ref tags) = c.tags {
        obj["tags"] = json!(tags);
    }

    obj
}

/// List clauses with optional filters.
#[utoipa::path(
    get,
    path = "/clauses",
    tag = "Clauses",
    params(
        ("category" = Option<String>, Query, description = "Filter by category"),
        ("status" = Option<String>, Query, description = "Filter by status (default: active)"),
        ("tag" = Option<String>, Query, description = "Filter by tag"),
    ),
    responses(
        (status = 200, description = "List of clauses"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_clauses(
    State(state): State<Arc<AppState>>,
    params: Query<ClauseListParams>,
) -> Result<Json<Value>, ApiError> {
    let status = params
        .status
        .clone()
        .unwrap_or_else(|| "active".to_string());

    let mut where_clauses = vec!["status = $status".to_string()];
    let mut binds: Vec<(&str, Value)> = vec![("status", json!(status))];

    if let Some(ref cat) = params.category {
        where_clauses.push("category = $category".to_string());
        binds.push(("category", json!(cat)));
    }

    if let Some(ref tag) = params.tag {
        where_clauses.push("tags CONTAINS $tag".to_string());
        binds.push(("tag", json!(tag)));
    }

    let where_str = where_clauses.join(" AND ");
    let query = format!(
        "SELECT * FROM clause WHERE {} ORDER BY sort_order ASC, category ASC",
        where_str
    );

    let mut q = state.db.query(&query);
    for (k, v) in &binds {
        q = q.bind((*k, v.clone()));
    }

    let mut response = q.await?;
    let clauses: Vec<Clause> = response.take(0)?;
    let total = clauses.len();

    let data: Vec<Value> = clauses.iter().map(clause_to_json).collect();

    Ok(Json(json!({ "data": data, "total": total })))
}

/// Get a single clause by ID.
#[utoipa::path(
    get,
    path = "/clauses/{id}",
    tag = "Clauses",
    params(("id" = String, Path, description = "Clause record key")),
    responses(
        (status = 200, description = "Clause found"),
        (status = 404, description = "Clause not found"),
    ),
    security(("api_key" = []))
)]
pub async fn get_clause(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("clause:").unwrap_or(&id);

    let clause: Option<Clause> = state.db.select(("clause", key)).await?;

    match clause {
        Some(c) => Ok(Json(json!({ "data": clause_to_json(&c) }))),
        None => Err(ApiError::not_found("Clause", &id)),
    }
}

/// Create a new clause.
#[utoipa::path(
    post,
    path = "/clauses",
    tag = "Clauses",
    request_body = Value,
    responses(
        (status = 201, description = "Clause created"),
        (status = 400, description = "Validation error"),
    ),
    security(("api_key" = []))
)]
pub async fn create_clause(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NewClause>,
) -> Result<Json<Value>, ApiError> {
    // Validate required fields
    if body.category.trim().is_empty() {
        return Err(ApiError::bad_request("category must not be empty"));
    }
    if body.title.trim().is_empty() {
        return Err(ApiError::bad_request("title must not be empty"));
    }
    if body.body.trim().is_empty() {
        return Err(ApiError::bad_request("body must not be empty"));
    }

    // Build optional SET fragments — never bind NULL for option<T> fields
    let mut optional_sets = String::new();
    if body.subcategory.is_some() {
        optional_sets.push_str(", subcategory = $subcategory");
    }
    if body.conditions.is_some() {
        optional_sets.push_str(", conditions = $conditions");
    }
    if body.tags.is_some() {
        optional_sets.push_str(", tags = $tags");
    }

    let query = format!(
        "CREATE clause SET \
         category = $category, \
         title = $title, \
         body = $body_text, \
         sort_order = $sort_order, \
         is_default = $is_default, \
         status = 'active', \
         version = 1, \
         created_at = time::now(), \
         updated_at = time::now(){optional_sets};"
    );

    let mut q = state
        .db
        .query(&query)
        .bind(("category", body.category.clone()))
        .bind(("title", body.title.clone()))
        .bind(("body_text", body.body.clone()))
        .bind(("sort_order", body.sort_order))
        .bind(("is_default", body.is_default));

    if let Some(ref sub) = body.subcategory {
        q = q.bind(("subcategory", sub.clone()));
    }
    if let Some(ref conds) = body.conditions {
        q = q.bind(("conditions", json_to_dbvalue(conds)));
    }
    if let Some(ref tags) = body.tags {
        q = q.bind(("tags", tags.clone()));
    }

    let mut response = q.await?;
    let clauses: Vec<Clause> = response.take(0)?;

    match clauses.into_iter().next() {
        Some(c) => Ok(Json(json!({ "data": clause_to_json(&c) }))),
        None => Err(ApiError::internal(
            "Failed to create clause — no record returned",
        )),
    }
}

/// Update an existing clause (partial update).
#[utoipa::path(
    put,
    path = "/clauses/{id}",
    tag = "Clauses",
    params(("id" = String, Path, description = "Clause record key")),
    request_body = Value,
    responses(
        (status = 200, description = "Clause updated"),
        (status = 404, description = "Clause not found"),
    ),
    security(("api_key" = []))
)]
pub async fn update_clause(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateClause>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("clause:").unwrap_or(&id);

    // Build dynamic SET clauses for provided fields
    let mut sets: Vec<String> = Vec::new();

    if let Some(ref cat) = body.category {
        if cat.trim().is_empty() {
            return Err(ApiError::bad_request("category must not be empty"));
        }
        sets.push("category = $category".into());
    }
    if body.subcategory.is_some() {
        sets.push("subcategory = $subcategory".into());
    }
    if let Some(ref title) = body.title {
        if title.trim().is_empty() {
            return Err(ApiError::bad_request("title must not be empty"));
        }
        sets.push("title = $title".into());
    }
    if let Some(ref body_text) = body.body {
        if body_text.trim().is_empty() {
            return Err(ApiError::bad_request("body must not be empty"));
        }
        sets.push("body = $body_text".into());
    }
    if body.sort_order.is_some() {
        sets.push("sort_order = $sort_order".into());
    }
    if body.is_default.is_some() {
        sets.push("is_default = $is_default".into());
    }
    if body.tags.is_some() {
        sets.push("tags = $tags".into());
    }
    if body.conditions.is_some() {
        sets.push("conditions = $conditions".into());
    }

    // Always bump version and updated_at
    sets.push("version = version + 1".into());
    sets.push("updated_at = time::now()".into());

    let set_str = sets.join(", ");
    let query = format!(
        "UPDATE clause:{key} SET {set_str}; SELECT * FROM clause:{key};",
        key = key,
        set_str = set_str,
    );

    // Bind each field individually — bind() requires owned 'static values
    let mut q = state.db.query(&query);
    if let Some(ref cat) = body.category {
        q = q.bind(("category", cat.clone()));
    }
    if let Some(ref sub) = body.subcategory {
        q = q.bind(("subcategory", sub.clone()));
    }
    if let Some(ref title) = body.title {
        q = q.bind(("title", title.clone()));
    }
    if let Some(ref body_text) = body.body {
        q = q.bind(("body_text", body_text.clone()));
    }
    if let Some(sort) = body.sort_order {
        q = q.bind(("sort_order", sort));
    }
    if let Some(def) = body.is_default {
        q = q.bind(("is_default", def));
    }
    if let Some(ref tags) = body.tags {
        q = q.bind(("tags", tags.clone()));
    }
    if let Some(ref conds) = body.conditions {
        q = q.bind(("conditions", json_to_dbvalue(conds)));
    }

    let mut response = q.await?;
    // Statement 0 is UPDATE (returns updated records), statement 1 is SELECT
    let clauses: Vec<Clause> = response.take(1)?;

    match clauses.into_iter().next() {
        Some(c) => Ok(Json(json!({ "data": clause_to_json(&c) }))),
        None => Err(ApiError::not_found("Clause", &id)),
    }
}

/// Soft-delete (archive) a clause.
#[utoipa::path(
    delete,
    path = "/clauses/{id}",
    tag = "Clauses",
    params(("id" = String, Path, description = "Clause record key")),
    responses(
        (status = 200, description = "Clause archived"),
        (status = 404, description = "Clause not found"),
    ),
    security(("api_key" = []))
)]
pub async fn delete_clause(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("clause:").unwrap_or(&id);

    let query = format!(
        "UPDATE clause:{key} SET status = 'archived', updated_at = time::now(); \
         SELECT * FROM clause:{key};",
        key = key,
    );

    let mut response = state.db.query(&query).await?;
    let clauses: Vec<Clause> = response.take(1)?;

    match clauses.into_iter().next() {
        Some(c) => Ok(Json(json!({ "data": clause_to_json(&c) }))),
        None => Err(ApiError::not_found("Clause", &id)),
    }
}

/// List distinct categories with clause counts.
#[utoipa::path(
    get,
    path = "/clauses/categories",
    tag = "Clauses",
    responses(
        (status = 200, description = "List of categories with counts"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_categories(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let query = "SELECT category, count() AS count FROM clause \
                 WHERE status = 'active' GROUP BY category ORDER BY category ASC";

    let mut response = state.db.query(query).await?;
    let rows: Vec<Value> = response.take(0)?;

    Ok(Json(json!({ "data": rows })))
}
