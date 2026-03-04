//! Pagination helpers for list endpoints.

use axum::extract::Query;
use serde::Deserialize;
use serde_json::{json, Value};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::error::ApiError;

/// Default and maximum page sizes.
const DEFAULT_PAGE_SIZE: u64 = 50;
const MAX_PAGE_SIZE: u64 = 100;

/// Query parameters for paginated list endpoints.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PaginationParams {
    /// Page number (1-indexed, default: 1).
    pub page: Option<u64>,
    /// Items per page (default: 50, max: 100).
    pub page_size: Option<u64>,
}

impl PaginationParams {
    /// Validated page number (minimum 1).
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    /// Validated page size (clamped to 1..=MAX_PAGE_SIZE).
    pub fn page_size(&self) -> u64 {
        self.page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE)
    }
}

/// Optional WHERE clause with parameterized bindings for filtered queries.
pub struct FilterClause {
    /// SurrealQL WHERE clause fragment, e.g. "status = $filter_status"
    pub clause: String,
    /// Parameterized bind values (name, value) pairs.
    pub binds: Vec<(String, serde_json::Value)>,
}

/// Execute a paginated query against a SurrealDB table.
///
/// Runs two statements in a single query:
/// 1. `SELECT count() FROM {table} GROUP ALL` — total count
/// 2. `SELECT * FROM {table} ORDER BY time.created_at DESC LIMIT {page_size} START {offset}` — page data
///
/// Returns `(items, total)`.
pub async fn db_paginate<T>(
    db: &Surreal<Client>,
    table: &str,
    params: &Query<PaginationParams>,
) -> Result<(Vec<T>, u64), ApiError>
where
    T: serde::de::DeserializeOwned + surrealdb::types::SurrealValue,
{
    db_paginate_filtered(db, table, params.page(), params.page_size(), None).await
}

/// Execute a paginated + optionally filtered query against a SurrealDB table.
///
/// When `filter` is provided, injects a WHERE clause into both the count and data queries.
/// All user values go through `$param` bindings (never string interpolation).
pub async fn db_paginate_filtered<T>(
    db: &Surreal<Client>,
    table: &str,
    page: u64,
    page_size: u64,
    filter: Option<FilterClause>,
) -> Result<(Vec<T>, u64), ApiError>
where
    T: serde::de::DeserializeOwned + surrealdb::types::SurrealValue,
{
    let page = page.max(1);
    let page_size = page_size.clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    let where_clause = filter
        .as_ref()
        .map(|f| format!(" WHERE {}", f.clause))
        .unwrap_or_default();

    let query = format!(
        "SELECT count() FROM {table}{where_clause} GROUP ALL; \
         SELECT * FROM {table}{where_clause} ORDER BY time.created_at DESC LIMIT {page_size} START {offset}",
    );

    let mut builder = db.query(&query);

    if let Some(ref f) = filter {
        for (name, value) in &f.binds {
            builder = builder.bind((name.clone(), value.clone()));
        }
    }

    let mut response = builder.await?;

    // Statement 0: count
    let count_result: Option<serde_json::Value> = response.take(0)?;
    let total = count_result
        .and_then(|v| v.get("count").and_then(|c| c.as_u64()))
        .unwrap_or(0);

    // Statement 1: paginated items
    let items: Vec<T> = response.take(1)?;

    Ok((items, total))
}

/// Build a paginated JSON response envelope from raw page/page_size values.
pub fn paginated_json_raw(data: Vec<Value>, total: u64, page: u64, page_size: u64) -> Value {
    let page = page.max(1);
    let page_size = page_size.clamp(1, MAX_PAGE_SIZE);
    let total_pages = if total == 0 {
        0
    } else {
        (total + page_size - 1) / page_size
    };

    json!({
        "data": data,
        "total": total,
        "page": page,
        "page_size": page_size,
        "total_pages": total_pages
    })
}

/// Build a paginated JSON response envelope.
pub fn paginated_json(data: Vec<Value>, total: u64, params: &Query<PaginationParams>) -> Value {
    paginated_json_raw(data, total, params.page(), params.page_size())
}
