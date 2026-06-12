# SurrealDB Patterns for E-Fees Project

## Database Configuration

### Connection Setup
```rust
use surrealdb::{Surreal, engine::local::RocksDb};
use surrealdb::opt::auth::Root;

pub async fn initialize_db() -> Result<Surreal<RocksDb>, surrealdb::Error> {
    // Create embedded database
    let db = Surreal::new::<RocksDb>("./data/efees.db").await?;
    
    // Sign in as root
    db.signin(Root {
        username: "root",
        password: "root",
    }).await?;
    
    // Select namespace and database
    db.use_ns("efees").use_db("production").await?;
    
    Ok(db)
}
```

### Tauri Integration
```rust
use tauri::State;
use std::sync::Mutex;

pub struct DatabaseState {
    pub connection: Mutex<Option<Surreal<RocksDb>>>,
}

#[tauri::command]
async fn init_database(state: State<'_, DatabaseState>) -> Result<(), String> {
    let db = initialize_db().await.map_err(|e| e.to_string())?;
    let mut conn = state.connection.lock().unwrap();
    *conn = Some(db);
    Ok(())
}
```

## Schema Design

### Table Definitions
```sql
-- Fees table
DEFINE TABLE fees SCHEMAFULL;
DEFINE FIELD id ON fees TYPE record;
DEFINE FIELD amount ON fees TYPE decimal ASSERT $value > 0;
DEFINE FIELD description ON fees TYPE string ASSERT string::len($value) > 0;
DEFINE FIELD category ON fees TYPE string;
DEFINE FIELD date ON fees TYPE datetime DEFAULT time::now();
DEFINE FIELD status ON fees TYPE string ASSERT $value IN ['pending', 'paid', 'overdue'];
DEFINE FIELD created_at ON fees TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON fees TYPE datetime DEFAULT time::now();

-- Users table
DEFINE TABLE users SCHEMAFULL;
DEFINE FIELD username ON users TYPE string ASSERT string::len($value) >= 3;
DEFINE FIELD email ON users TYPE string ASSERT string::is::email($value);
DEFINE FIELD password_hash ON users TYPE string;
DEFINE FIELD role ON users TYPE string ASSERT $value IN ['admin', 'user'];
DEFINE FIELD created_at ON users TYPE datetime DEFAULT time::now();

-- Payments table
DEFINE TABLE payments SCHEMAFULL;
DEFINE FIELD fee_id ON payments TYPE record<fees>;
DEFINE FIELD amount ON payments TYPE decimal;
DEFINE FIELD method ON payments TYPE string ASSERT $value IN ['cash', 'card', 'bank_transfer'];
DEFINE FIELD transaction_id ON payments TYPE string;
DEFINE FIELD paid_at ON payments TYPE datetime DEFAULT time::now();

-- Indexes
DEFINE INDEX idx_fees_date ON fees FIELDS date;
DEFINE INDEX idx_fees_status ON fees FIELDS status;
DEFINE INDEX idx_users_email ON users FIELDS email UNIQUE;
DEFINE INDEX idx_payments_fee ON payments FIELDS fee_id;
```

### Relationships
```sql
-- Define relationships
DEFINE TABLE paid SCHEMAFULL;
DEFINE FIELD in ON paid TYPE record<fees>;
DEFINE FIELD out ON paid TYPE record<payments>;
DEFINE FIELD amount ON paid TYPE decimal;

-- Query fees with payments
SELECT *, ->paid->payments.* AS payments 
FROM fees
WHERE id = fees:abc123;
```

## CRUD Operations

### Create (Insert)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Fee {
    amount: f64,
    description: String,
    category: String,
    status: String,
}

#[tauri::command]
async fn create_fee(
    state: State<'_, DatabaseState>,
    fee: Fee,
) -> Result<String, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let created: Option<Fee> = db
        .create("fees")
        .content(fee)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(format!("Created fee: {:?}", created))
}
```

### Read (Select)
```rust
#[derive(Debug, Serialize, Deserialize)]
struct FeeRecord {
    id: Thing,
    amount: f64,
    description: String,
    category: String,
    status: String,
}

// Get all fees
#[tauri::command]
async fn get_fees(
    state: State<'_, DatabaseState>,
) -> Result<Vec<FeeRecord>, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let fees: Vec<FeeRecord> = db
        .select("fees")
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(fees)
}

// Get specific fee
#[tauri::command]
async fn get_fee_by_id(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Option<FeeRecord>, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let fee: Option<FeeRecord> = db
        .select(("fees", id))
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(fee)
}
```

### Update
```rust
#[tauri::command]
async fn update_fee(
    state: State<'_, DatabaseState>,
    id: String,
    updates: Fee,
) -> Result<Option<FeeRecord>, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let updated: Option<FeeRecord> = db
        .update(("fees", id))
        .content(updates)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(updated)
}

// Partial update
#[tauri::command]
async fn update_fee_status(
    state: State<'_, DatabaseState>,
    id: String,
    status: String,
) -> Result<Option<FeeRecord>, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let updated: Option<FeeRecord> = db
        .update(("fees", id))
        .merge(serde_json::json!({
            "status": status,
            "updated_at": time::now()
        }))
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(updated)
}
```

### Delete
```rust
#[tauri::command]
async fn delete_fee(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<(), String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let _: Option<FeeRecord> = db
        .delete(("fees", id))
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}
```

## Advanced Queries

### Filtering and Pagination
```rust
#[tauri::command]
async fn search_fees(
    state: State<'_, DatabaseState>,
    category: Option<String>,
    status: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<FeeRecord>, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let mut query = "SELECT * FROM fees WHERE 1=1".to_string();
    
    if let Some(cat) = category {
        query.push_str(&format!(" AND category = '{}'", cat));
    }
    
    if let Some(stat) = status {
        query.push_str(&format!(" AND status = '{}'", stat));
    }
    
    query.push_str(" ORDER BY date DESC");
    
    if let Some(lim) = limit {
        query.push_str(&format!(" LIMIT {}", lim));
    }
    
    if let Some(off) = offset {
        query.push_str(&format!(" START {}", off));
    }
    
    let mut result = db.query(query).await.map_err(|e| e.to_string())?;
    let fees: Vec<FeeRecord> = result.take(0).map_err(|e| e.to_string())?;
    
    Ok(fees)
}
```

### Aggregations
```rust
#[derive(Debug, Serialize, Deserialize)]
struct FeeStats {
    total: f64,
    count: u32,
    average: f64,
}

#[tauri::command]
async fn get_fee_stats(
    state: State<'_, DatabaseState>,
    category: Option<String>,
) -> Result<FeeStats, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let query = if let Some(cat) = category {
        format!(
            "SELECT 
                math::sum(amount) AS total,
                count() AS count,
                math::mean(amount) AS average 
            FROM fees 
            WHERE category = '{}'
            GROUP ALL",
            cat
        )
    } else {
        "SELECT 
            math::sum(amount) AS total,
            count() AS count,
            math::mean(amount) AS average 
        FROM fees 
        GROUP ALL".to_string()
    };
    
    let mut result = db.query(query).await.map_err(|e| e.to_string())?;
    let stats: Vec<FeeStats> = result.take(0).map_err(|e| e.to_string())?;
    
    stats.into_iter().next().ok_or("No stats found".to_string())
}
```

### Date Range Queries
```rust
#[tauri::command]
async fn get_fees_by_date_range(
    state: State<'_, DatabaseState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<FeeRecord>, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    let query = format!(
        "SELECT * FROM fees 
        WHERE date >= '{}' AND date <= '{}'
        ORDER BY date DESC",
        start_date, end_date
    );
    
    let mut result = db.query(query).await.map_err(|e| e.to_string())?;
    let fees: Vec<FeeRecord> = result.take(0).map_err(|e| e.to_string())?;
    
    Ok(fees)
}
```

## Transaction Management

### Basic Transactions
```rust
#[tauri::command]
async fn process_payment(
    state: State<'_, DatabaseState>,
    fee_id: String,
    payment_amount: f64,
    payment_method: String,
) -> Result<String, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    // Start transaction
    let mut result = db.query("
        BEGIN TRANSACTION;
        
        -- Update fee status
        UPDATE $fee_id SET status = 'paid', updated_at = time::now();
        
        -- Create payment record
        CREATE payments CONTENT {
            fee_id: $fee_id,
            amount: $amount,
            method: $method,
            paid_at: time::now()
        };
        
        COMMIT TRANSACTION;
    ")
    .bind(("fee_id", fee_id))
    .bind(("amount", payment_amount))
    .bind(("method", payment_method))
    .await
    .map_err(|e| e.to_string())?;
    
    Ok("Payment processed successfully".to_string())
}
```

## Performance Optimization

### Batch Operations
```rust
#[tauri::command]
async fn create_multiple_fees(
    state: State<'_, DatabaseState>,
    fees: Vec<Fee>,
) -> Result<u32, String> {
    let db = state.connection.lock().unwrap();
    let db = db.as_ref().ok_or("Database not connected")?;
    
    // Batch insert
    let count = fees.len() as u32;
    
    for fee in fees {
        db.create("fees")
            .content(fee)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    Ok(count)
}
```

### Caching Strategy
```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CachedQuery {
    data: Vec<FeeRecord>,
    timestamp: Instant,
}

pub struct QueryCache {
    cache: Mutex<HashMap<String, CachedQuery>>,
    ttl: Duration,
}

impl QueryCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<Vec<FeeRecord>> {
        let cache = self.cache.lock().unwrap();
        
        if let Some(cached) = cache.get(key) {
            if cached.timestamp.elapsed() < self.ttl {
                return Some(cached.data.clone());
            }
        }
        
        None
    }
    
    pub fn set(&self, key: String, data: Vec<FeeRecord>) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, CachedQuery {
            data,
            timestamp: Instant::now(),
        });
    }
}
```

## Error Handling

### Custom Error Types
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Database error: {0}")]
    SurrealDB(#[from] surrealdb::Error),
}

impl From<DatabaseError> for String {
    fn from(err: DatabaseError) -> Self {
        err.to_string()
    }
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_fee() {
        let db = initialize_db().await.unwrap();
        
        let fee = Fee {
            amount: 100.0,
            description: "Test fee".to_string(),
            category: "test".to_string(),
            status: "pending".to_string(),
        };
        
        let created: Option<Fee> = db
            .create("fees")
            .content(fee)
            .await
            .unwrap();
        
        assert!(created.is_some());
    }
}
```

## Migration Strategy

### Schema Migrations
```rust
pub async fn migrate_database(db: &Surreal<RocksDb>) -> Result<(), String> {
    // Check current version
    let version: Option<i32> = db
        .query("SELECT VALUE version FROM schema_version LIMIT 1")
        .await
        .map_err(|e| e.to_string())?
        .take(0)
        .map_err(|e| e.to_string())?;
    
    let current_version = version.unwrap_or(0);
    
    // Apply migrations
    if current_version < 1 {
        apply_migration_v1(db).await?;
    }
    
    if current_version < 2 {
        apply_migration_v2(db).await?;
    }
    
    Ok(())
}

async fn apply_migration_v1(db: &Surreal<RocksDb>) -> Result<(), String> {
    db.query("
        DEFINE TABLE fees SCHEMAFULL;
        -- Add fields...
        UPDATE schema_version SET version = 1;
    ")
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(())
}
```

## Best Practices

1. **Always use prepared statements** (parameterized queries) to prevent injection
2. **Use transactions** for multi-step operations
3. **Implement proper error handling** with custom error types
4. **Cache frequently accessed data** with TTL
5. **Use indexes** for commonly queried fields
6. **Validate data** before inserting
7. **Use SCHEMAFULL** tables for production
8. **Implement proper backup strategy**
9. **Monitor query performance**
10. **Use connection pooling** in high-traffic scenarios

## Useful Resources

- SurrealDB Docs: https://surrealdb.com/docs
- SurrealDB Rust SDK: https://docs.rs/surrealdb/
- SurrealQL Guide: https://surrealdb.com/docs/surrealql
