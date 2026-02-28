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
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let query = format!(
        "SELECT count() FROM {} GROUP ALL; SELECT * FROM {} ORDER BY time.created_at DESC LIMIT {} START {}",
        table, table, page_size, offset
    );

    let mut response = db.query(&query).await?;

    // Statement 0: count
    let count_result: Option<serde_json::Value> = response.take(0)?;
    let total = count_result
        .and_then(|v| v.get("count").and_then(|c| c.as_u64()))
        .unwrap_or(0);

    // Statement 1: paginated items
    let items: Vec<T> = response.take(1)?;

    Ok((items, total))
}

/// Build a paginated JSON response envelope.
pub fn paginated_json(data: Vec<Value>, total: u64, params: &Query<PaginationParams>) -> Value {
    let page = params.page();
    let page_size = params.page_size();
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
