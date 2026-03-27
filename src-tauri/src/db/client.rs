//! Database client abstraction for HTTP and WebSocket connections.

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::engine::remote::http::{Client as HttpClient, Http};
use surrealdb::opt::auth::{Root, Namespace, Database};
use surrealdb::{Error, Surreal};
use log::{info, warn, error};

use super::types::{
    Project, NewProject, Company, CompanyCreate, Contact, ContactCreate,
    Fee, FeeCreate, FeeUpdate,
};
use crate::commands::{CompanyUpdate, ContactUpdate, ProjectUpdate};

/// Normalize a SurrealDB WebSocket URL to ensure it ends with `/rpc`.
/// SurrealDB SDK v3 requires the `/rpc` endpoint; without it, connections
/// silently fail to execute queries.
pub(crate) fn normalize_ws_url(url: &str) -> String {
    if url.ends_with("/rpc") {
        url.to_string()
    } else {
        format!("{}/rpc", url.trim_end_matches('/'))
    }
}

/// Validate a record ID contains only alphanumeric chars, underscores, and hyphens.
/// Returns Error if invalid, preventing SQL injection in table:key references.
fn validate_record_id(id: &str, field_name: &str) -> Result<(), Error> {
    if id.is_empty() || id.len() > 100 {
        return Err(Error::thrown(format!("Invalid {} length", field_name)));
    }
    if !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(Error::thrown(format!(
            "Invalid {} format: only alphanumeric, underscores, hyphens allowed", field_name
        )));
    }
    Ok(())
}

/// Database client enum to handle different connection types.
#[derive(Clone)]
pub enum DatabaseClient {
    Http(Surreal<HttpClient>),
    WebSocket(Surreal<Client>),
}

/// Macro to eliminate repeated match patterns across Http and WebSocket clients.
/// This macro takes a method call and applies it to both client types.
macro_rules! delegate_to_client {
    ($self:expr, $method:ident $(, $args:expr)* $(,)?) => {
        match $self {
            DatabaseClient::Http(client) => client.$method($($args),*).await,
            DatabaseClient::WebSocket(client) => client.$method($($args),*).await,
        }
    };
}

/// Macro for delete operations on a table with ID.
/// QUAL-H1: Reduces match statement duplication for delete patterns.
macro_rules! delegate_delete {
    ($self:expr, $table:expr, $id:expr) => {
        match $self {
            DatabaseClient::Http(client) => client.delete(($table, $id)).await,
            DatabaseClient::WebSocket(client) => client.delete(($table, $id)).await,
        }
    };
}

/// Macro for update operations with merge.
/// QUAL-H1: Reduces match statement duplication for update patterns.
macro_rules! delegate_update_merge {
    ($self:expr, $table:expr, $id:expr, $data:expr) => {
        match $self {
            DatabaseClient::Http(client) => client.update(($table, $id)).merge($data).await,
            DatabaseClient::WebSocket(client) => client.update(($table, $id)).merge($data).await,
        }
    };
}

impl DatabaseClient {
    /// Connect via WebSocket with HTTP fallback.
    pub async fn connect(url: &str) -> Result<Self, Error> {
        if url.starts_with("ws://") || url.starts_with("wss://") {
            let is_secure = url.starts_with("wss://");

            let normalized_url = normalize_ws_url(url);
            if normalized_url != url {
                warn!("SURREALDB_URL missing /rpc suffix — auto-corrected to: {}", normalized_url);
            }

            info!("Attempting {} WebSocket connection to {}",
                  if is_secure { "secure (WSS)" } else { "unencrypted (WS)" }, normalized_url);

            let connection_address = normalized_url
                .strip_prefix("ws://")
                .or_else(|| normalized_url.strip_prefix("wss://"))
                .unwrap_or(&normalized_url)
                .to_string();

            match Surreal::new::<Ws>(connection_address).await {
                Ok(connection) => {
                    info!("Successfully established WebSocket connection");
                    if !is_secure {
                        warn!("WARNING: Using unencrypted WebSocket connection (ws://)");
                    }
                    Ok(DatabaseClient::WebSocket(connection))
                }
                Err(ws_err) => {
                    warn!("WebSocket connection failed: {}, attempting HTTP fallback", ws_err);

                    let http_url = if normalized_url.starts_with("ws://") {
                        normalized_url.replace("ws://", "http://")
                    } else {
                        normalized_url.replace("wss://", "https://")
                    };

                    match Surreal::new::<Http>(&http_url).await {
                        Ok(connection) => {
                            info!("Successfully established HTTP fallback connection");
                            Ok(DatabaseClient::Http(connection))
                        }
                        Err(http_err) => {
                            error!("Both WebSocket and HTTP connections failed:");
                            error!("  WebSocket error: {}", ws_err);
                            error!("  HTTP error: {}", http_err);
                            Err(ws_err)
                        }
                    }
                }
            }
        } else {
            info!("Connecting via HTTP to {}", url);
            match Surreal::new::<Http>(url).await {
                Ok(connection) => {
                    info!("Successfully established HTTP connection");
                    Ok(DatabaseClient::Http(connection))
                }
                Err(err) => {
                    error!("Failed to establish HTTP connection: {}", err);
                    Err(err)
                }
            }
        }
    }

    /// Check connection health.
    pub async fn health(&self) -> Result<(), Error> {
        delegate_to_client!(self, health)
    }

    /// Sign in with root credentials.
    pub async fn signin_root(&self, username: &str, password: &str) -> Result<(), Error> {
        let username = username.to_string();
        let password = password.to_string();
        match self {
            DatabaseClient::Http(client) => {
                client.signin(Root { username, password }).await?;
                Ok(())
            },
            DatabaseClient::WebSocket(client) => {
                client.signin(Root { username, password }).await?;
                Ok(())
            },
        }
    }

    /// Sign in with namespace credentials.
    pub async fn signin_namespace(&self, namespace: &str, username: &str, password: &str) -> Result<(), Error> {
        let namespace = namespace.to_string();
        let username = username.to_string();
        let password = password.to_string();
        match self {
            DatabaseClient::Http(client) => {
                client.signin(Namespace { namespace, username, password }).await?;
                Ok(())
            },
            DatabaseClient::WebSocket(client) => {
                client.signin(Namespace { namespace, username, password }).await?;
                Ok(())
            },
        }
    }

    /// Sign in with database credentials.
    pub async fn signin_database(&self, namespace: &str, database: &str, username: &str, password: &str) -> Result<(), Error> {
        let namespace = namespace.to_string();
        let database = database.to_string();
        let username = username.to_string();
        let password = password.to_string();
        match self {
            DatabaseClient::Http(client) => {
                client.signin(Database { namespace, database, username, password }).await?;
                Ok(())
            },
            DatabaseClient::WebSocket(client) => {
                client.signin(Database { namespace, database, username, password }).await?;
                Ok(())
            },
        }
    }

    /// Select namespace and database.
    pub async fn use_ns_db(&self, namespace: &str, database: &str) -> Result<(), Error> {
        match self {
            DatabaseClient::Http(client) => { client.use_ns(namespace).use_db(database).await?; Ok(()) },
            DatabaseClient::WebSocket(client) => { client.use_ns(namespace).use_db(database).await?; Ok(()) },
        }
    }

    /// Execute a raw query.
    pub async fn query(&self, query: &str) -> Result<surrealdb::IndexedResults, Error> {
        delegate_to_client!(self, query, query)
    }

    /// Execute a parameterized query with bindings (SQL injection safe).
    pub async fn query_bind<T: surrealdb::types::SurrealValue + 'static>(&self, query: &str, bindings: T) -> Result<surrealdb::IndexedResults, Error> {
        match self {
            DatabaseClient::Http(client) => client.query(query).bind(bindings).await,
            DatabaseClient::WebSocket(client) => client.query(query).bind(bindings).await,
        }
    }

    /// Execute a parameterized query with multiple named bindings.
    /// Each map entry becomes a `$key` parameter in the query.
    pub async fn query_bind_map(&self, query: &str, bindings: serde_json::Map<String, serde_json::Value>) -> Result<surrealdb::IndexedResults, Error> {
        match self {
            DatabaseClient::Http(client) => {
                let mut q = client.query(query);
                for (key, value) in bindings {
                    q = q.bind((key, value));
                }
                q.await
            },
            DatabaseClient::WebSocket(client) => {
                let mut q = client.query(query);
                for (key, value) in bindings {
                    q = q.bind((key, value));
                }
                q.await
            },
        }
    }

    // ==================== Project Operations ====================

    pub async fn create_project(&self, project: Project) -> Result<Option<Project>, Error> {
        match self {
            DatabaseClient::Http(client) => client.create("projects").content(project).await,
            DatabaseClient::WebSocket(client) => client.create("projects").content(project).await,
        }
    }

    pub async fn create_new_project(&self, project: NewProject) -> Result<Option<Project>, Error> {
        let project_id = project.number.id.replace("-", "_");

        // Normalize country name via fn::resolve_country; fall back to raw input if unresolved.
        let resolved_country = self.resolve_country(&project.country).await
            .ok()
            .flatten()
            .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| project.country.clone());

        let set_clauses = vec![
            format!("name = '{}'", project.name.replace("'", "''")),
            format!("name_short = '{}'", project.name_short.replace("'", "''")),
            format!("status = '{}'", project.status.replace("'", "''")),
            format!("area = '{}'", project.area.replace("'", "''")),
            format!("city = '{}'", project.city.replace("'", "''")),
            format!("country = '{}'", resolved_country.replace("'", "''")),
            format!("folder = '{}'", project.folder.replace("'", "''")),
            format!("number = {{ year: {}, country: {}, seq: {}, id: '{}' }}",
                project.number.year, project.number.country, project.number.seq,
                project.number.id.replace("'", "''"))
        ];

        let query = format!("CREATE projects:{} SET {}", project_id, set_clauses.join(", "));
        info!("Executing project creation query");

        let mut response = self.query(&query).await?;
        let result: Result<Vec<Project>, _> = response.take(0);
        match result {
            Ok(mut projects) => Ok(projects.pop()),
            Err(e) => Err(e),
        }
    }

    pub async fn update_project(&self, id: &str, project_data: ProjectUpdate) -> Result<Option<Project>, Error> {
        delegate_update_merge!(self, "projects", id, project_data)
    }

    pub async fn delete_project(&self, id: &str) -> Result<Option<Project>, Error> {
        delegate_delete!(self, "projects", id)
    }

    pub async fn search_projects(&self, query_str: &str) -> Result<Vec<Project>, Error> {
        // Sanitize input to prevent SQL injection
        // Only allow alphanumeric characters, spaces, hyphens, and underscores
        let sanitized_query: String = query_str
            .chars()
            .filter(|c| c.is_alphanumeric() || " -_".contains(*c))
            .collect();

        // Validate length
        if sanitized_query.is_empty() || sanitized_query.len() > 100 {
            info!("Project search query invalid or too long, returning empty results");
            return Ok(vec![]);
        }

        // Escape single quotes for SQL
        let escaped_query = sanitized_query.replace("'", "''");

        let search_query = format!(
            r#"SELECT * FROM projects WHERE
               string::lower(name) CONTAINS string::lower('{}') OR
               string::lower(name_short) CONTAINS string::lower('{}') OR
               string::lower(number.id) CONTAINS string::lower('{}') OR
               string::lower(city) CONTAINS string::lower('{}') OR
               string::lower(area) CONTAINS string::lower('{}') OR
               string::lower(country) CONTAINS string::lower('{}') OR
               string::lower(folder) CONTAINS string::lower('{}')
               ORDER BY time.created_at DESC"#,
            escaped_query, escaped_query, escaped_query, escaped_query,
            escaped_query, escaped_query, escaped_query
        );

        let mut response = self.query(&search_query).await?;
        response.take(0)
    }

    // ==================== Company Operations ====================

    pub async fn create_company(&self, company: CompanyCreate) -> Result<Option<Company>, Error> {
        // Normalize country name via fn::resolve_country; fall back to raw input if unresolved.
        let resolved_country = self.resolve_country(&company.country).await
            .ok()
            .flatten()
            .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| company.country.clone());

        let query = format!(
            "CREATE company:{} SET name = '{}', name_short = '{}', abbreviation = '{}', city = '{}', country = '{}', reg_no = {}, tax_no = {}, time = {{ created_at: time::now(), updated_at: time::now() }}",
            company.abbreviation,
            company.name.replace("'", "''"),
            company.name_short.replace("'", "''"),
            company.abbreviation.replace("'", "''"),
            company.city.replace("'", "''"),
            resolved_country.replace("'", "''"),
            company.reg_no.map_or("NONE".to_string(), |v| format!("'{}'", v.replace("'", "''"))),
            company.tax_no.map_or("NONE".to_string(), |v| format!("'{}'", v.replace("'", "''")))
        );

        let mut response = self.query(&query).await?;
        let result: Result<Vec<Company>, _> = response.take(0);
        match result {
            Ok(mut companies) => Ok(companies.pop()),
            Err(e) => Err(e),
        }
    }

    pub async fn update_company_partial(&self, id: &str, company_update: CompanyUpdate) -> Result<Option<Company>, Error> {
        delegate_update_merge!(self, "company", id, company_update)
    }

    pub async fn delete_company(&self, id: &str) -> Result<Option<Company>, Error> {
        delegate_delete!(self, "company", id)
    }

    // ==================== Contact Operations ====================

    pub async fn create_contact(&self, contact: ContactCreate) -> Result<Option<Contact>, Error> {
        info!("Creating contact with company ID: {}", contact.company);

        let full_name = format!("{} {}", contact.first_name, contact.last_name);
        let query = format!(
            "CREATE contacts SET first_name = '{}', last_name = '{}', full_name = '{}', email = '{}', phone = '{}', position = '{}', company = company:{}, time = {{ created_at: time::now(), updated_at: time::now() }}",
            contact.first_name.replace("'", "''"),
            contact.last_name.replace("'", "''"),
            full_name.replace("'", "''"),
            contact.email.replace("'", "''"),
            contact.phone.replace("'", "''"),
            contact.position.replace("'", "''"),
            contact.company
        );

        info!("Executing contact creation query");

        let mut response = self.query(&query).await?;
        let result: Result<Vec<Contact>, _> = response.take(0);
        match result {
            Ok(mut contacts) => Ok(contacts.pop()),
            Err(e) => Err(e),
        }
    }

    pub async fn update_contact_partial(&self, id: &str, contact_update: ContactUpdate) -> Result<Option<Contact>, Error> {
        let mut set_clauses = Vec::new();

        if let Some(first_name) = &contact_update.first_name {
            set_clauses.push(format!("first_name = '{}'", first_name.replace("'", "''")));
        }
        if let Some(last_name) = &contact_update.last_name {
            set_clauses.push(format!("last_name = '{}'", last_name.replace("'", "''")));
        }
        if let Some(full_name) = &contact_update.full_name {
            set_clauses.push(format!("full_name = '{}'", full_name.replace("'", "''")));
        }
        if let Some(email) = &contact_update.email {
            set_clauses.push(format!("email = '{}'", email.replace("'", "''")));
        }
        if let Some(phone) = &contact_update.phone {
            set_clauses.push(format!("phone = '{}'", phone.replace("'", "''")));
        }
        if let Some(position) = &contact_update.position {
            set_clauses.push(format!("position = '{}'", position.replace("'", "''")));
        }
        if let Some(company) = &contact_update.company {
            set_clauses.push(format!("company = company:{}", company));
        }

        set_clauses.push("time.updated_at = time::now()".to_string());

        if set_clauses.len() == 1 {
            return Ok(None); // Nothing to update except timestamp
        }

        let query = format!(
            "UPDATE contacts:{} SET {} RETURN AFTER",
            id,
            set_clauses.join(", ")
        );

        info!("Executing contact update query");

        let mut response = self.query(&query).await?;
        let result: Result<Vec<Contact>, _> = response.take(0);
        match result {
            Ok(mut contacts) => {
                info!("Update query returned {} contacts", contacts.len());
                Ok(contacts.pop())
            },
            Err(e) => {
                info!("Update query failed with error: {:?}", e);
                Err(e)
            },
        }
    }

    pub async fn delete_contact(&self, id: &str) -> Result<Option<Contact>, Error> {
        delegate_delete!(self, "contacts", id)
    }

    // ==================== Fee Operations ====================

    pub async fn create_fee(&self, fee: FeeCreate) -> Result<Option<Fee>, Error> {
        let fee_id = format!("{}_{}", fee.project_id.replace("-", "_"), fee.rev);

        // Validate all IDs (alphanumeric + underscores only)
        validate_record_id(&fee_id, "fee_id")?;
        validate_record_id(&fee.project_id.replace("-", "_"), "project_id")?;
        validate_record_id(&fee.company_id, "company_id")?;
        validate_record_id(&fee.contact_id, "contact_id")?;

        // Single-statement CREATE SET with all fields inline.
        // SurrealDB v3 strict schema rejects NONE for record<T> fields,
        // so record links must be in the same CREATE, not a separate UPDATE.
        let project_key = fee.project_id.replace("-", "_");
        let query = format!(
            "CREATE fee:{fee_id} SET \
             name = $name, \
             number = $number, \
             rev = $rev, \
             status = $status, \
             issue_date = $issue_date, \
             activity = $activity, \
             package = $package, \
             strap_line = $strap_line, \
             staff_name = $staff_name, \
             staff_email = $staff_email, \
             staff_phone = $staff_phone, \
             staff_position = $staff_position, \
             revisions = $revisions, \
             project_id = type::record('projects', $project_key), \
             company_id = type::record('company', $company_id), \
             contact_id = type::record('contacts', $contact_id), \
             time = {{ created_at: time::now(), updated_at: time::now() }};",
            fee_id = fee_id,
        );

        let mut bindings = serde_json::Map::new();
        bindings.insert("name".into(), serde_json::Value::String(fee.name));
        bindings.insert("number".into(), serde_json::Value::String(fee.number));
        bindings.insert("rev".into(), serde_json::json!(fee.rev));
        bindings.insert("status".into(), serde_json::Value::String(fee.status));
        bindings.insert("issue_date".into(), serde_json::Value::String(fee.issue_date));
        bindings.insert("activity".into(), serde_json::Value::String(fee.activity));
        bindings.insert("package".into(), serde_json::Value::String(fee.package));
        bindings.insert("strap_line".into(), serde_json::Value::String(fee.strap_line));
        bindings.insert("staff_name".into(), serde_json::Value::String(fee.staff_name));
        bindings.insert("staff_email".into(), serde_json::Value::String(fee.staff_email));
        bindings.insert("staff_phone".into(), serde_json::Value::String(fee.staff_phone));
        bindings.insert("staff_position".into(), serde_json::Value::String(fee.staff_position));
        bindings.insert("revisions".into(), serde_json::to_value(&fee.revisions).unwrap_or(serde_json::json!([])));
        bindings.insert("project_key".into(), serde_json::Value::String(project_key));
        bindings.insert("company_id".into(), serde_json::Value::String(fee.company_id));
        bindings.insert("contact_id".into(), serde_json::Value::String(fee.contact_id));

        info!("Executing Fee creation query (parameterized)");

        let mut response = self.query_bind_map(&query, bindings).await?;
        let result: Result<Vec<Fee>, _> = response.take(0);
        match result {
            Ok(mut fees) => Ok(fees.pop()),
            Err(e) => Err(e),
        }
    }

    pub async fn update_fee(&self, id: &str, fee: FeeUpdate) -> Result<Option<Fee>, Error> {
        info!("DatabaseClient::update_fee called with id: '{}'", id);

        // Validate all IDs
        validate_record_id(id, "fee_id")?;
        validate_record_id(&fee.project_id.replace("-", "_"), "project_id")?;
        validate_record_id(&fee.company_id, "company_id")?;
        validate_record_id(&fee.contact_id, "contact_id")?;

        // Build parameterized data map for string/number fields
        let mut data = serde_json::Map::new();
        data.insert("name".into(), serde_json::Value::String(fee.name));
        data.insert("number".into(), serde_json::Value::String(fee.number));
        data.insert("rev".into(), serde_json::json!(fee.rev));
        data.insert("status".into(), serde_json::Value::String(fee.status));
        data.insert("issue_date".into(), serde_json::Value::String(fee.issue_date));
        data.insert("activity".into(), serde_json::Value::String(fee.activity.unwrap_or_default()));
        data.insert("package".into(), serde_json::Value::String(fee.package.unwrap_or_default()));
        data.insert("strap_line".into(), serde_json::Value::String(fee.strap_line.unwrap_or_default()));
        data.insert("staff_name".into(), serde_json::Value::String(fee.staff_name.unwrap_or_default()));
        data.insert("staff_email".into(), serde_json::Value::String(fee.staff_email.unwrap_or_default()));
        data.insert("staff_phone".into(), serde_json::Value::String(fee.staff_phone.unwrap_or_default()));
        data.insert("staff_position".into(), serde_json::Value::String(fee.staff_position.unwrap_or_default()));
        data.insert("revisions".into(), serde_json::to_value(&fee.revisions).unwrap_or(serde_json::json!([])));

        let data_value = serde_json::Value::Object(data);

        // Record-id fields use type::record() with parameterized bindings.
        // String fields go through $data (parameterized, no injection risk).
        let project_key = fee.project_id.replace("-", "_");
        let query = format!(
            "UPDATE fee:{id} MERGE $data RETURN NONE; \
             UPDATE fee:{id} SET \
             project_id = type::record('projects', $project_key), \
             company_id = type::record('company', $company_id), \
             contact_id = type::record('contacts', $contact_id), \
             time.updated_at = time::now() RETURN NONE; \
             SELECT * OMIT import_source FROM fee:{id};",
            id = id,
        );

        let mut bindings = serde_json::Map::new();
        bindings.insert("data".into(), data_value);
        bindings.insert("project_key".into(), serde_json::Value::String(project_key));
        bindings.insert("company_id".into(), serde_json::Value::String(fee.company_id));
        bindings.insert("contact_id".into(), serde_json::Value::String(fee.contact_id));

        info!("Executing fee update query (parameterized)");

        let mut response = self.query_bind_map(&query, bindings).await?;
        let result: Result<Vec<Fee>, _> = response.take(2);
        match result {
            Ok(mut fees) => {
                info!("Update query returned {} records", fees.len());
                if fees.is_empty() {
                    warn!("Update query returned empty result set");
                }
                Ok(fees.pop())
            },
            Err(e) => {
                error!("Failed to parse update response: {}", e);
                Err(e)
            },
        }
    }

    pub async fn delete_fee(&self, id: &str) -> Result<Option<Fee>, Error> {
        // Custom delete: fetch first with OMIT, then delete.
        // Can't use delegate_delete! because the returned Fee may contain
        // import_source with native datetime that serde_json::Value can't handle.
        validate_record_id(id, "fee_id")?;
        let query = format!(
            "SELECT * OMIT import_source FROM fee:{id}; DELETE fee:{id};",
            id = id
        );
        let mut response = self.query(&query).await?;
        let result: Vec<Fee> = response.take(0)?;
        Ok(result.into_iter().next())
    }

    /// Update only the pricing-related fields of a fee.
    /// Uses MERGE to update only specified fields without affecting others.
    pub async fn update_fee_pricing(&self, id: &str, pricing: super::PricingUpdate) -> Result<Option<Fee>, Error> {
        info!("DatabaseClient::update_fee_pricing called with id: '{}'", id);

        // Build a dynamic update object with only non-None fields
        let mut update_obj = serde_json::Map::new();

        if let Some(ref p) = pricing.pricing {
            update_obj.insert("pricing".to_string(), serde_json::to_value(p).unwrap_or(serde_json::Value::Null));
        }
        if let Some(ref items) = pricing.post_contract_items {
            update_obj.insert("post_contract_items".to_string(), serde_json::to_value(items).unwrap_or(serde_json::Value::Null));
        }
        if let Some(ref costs) = pricing.reimbursable_costs {
            update_obj.insert("reimbursable_costs".to_string(), serde_json::to_value(costs).unwrap_or(serde_json::Value::Null));
        }
        if let Some(ref schedule) = pricing.payment_schedule {
            update_obj.insert("payment_schedule".to_string(), serde_json::to_value(schedule).unwrap_or(serde_json::Value::Null));
        }

        let update_value = serde_json::Value::Object(update_obj);

        // Two-step update: MERGE pricing data, then SET timestamp with native datetime
        // (MERGE with a JSON string for datetime fails SurrealDB schema validation)
        let query = format!(
            "UPDATE fee:{id} MERGE $data RETURN NONE; \
             UPDATE fee:{id} SET time.updated_at = time::now() RETURN NONE; \
             SELECT * OMIT import_source FROM fee:{id};",
            id = id
        );

        info!("Executing pricing update with MERGE + timestamp SET");

        let mut response = self.query_bind(&query, ("data", update_value)).await?;
        let result: Result<Vec<Fee>, _> = response.take(2);
        match result {
            Ok(mut fees) => {
                info!("Pricing update returned {} records", fees.len());
                Ok(fees.pop())
            },
            Err(e) => {
                error!("Failed to parse pricing update response: {}", e);
                Err(e)
            },
        }
    }

    // ==================== Batch Operations ====================

    /// Delete multiple records from a table by their IDs.
    /// Table name is validated against an allowlist. IDs are parameterized.
    pub async fn batch_delete(&self, table: &str, ids: &[String]) -> Result<Vec<serde_json::Value>, Error> {
        Self::validate_table_name(table)?;
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let omit_clause = if table == "fee" { " OMIT import_source" } else { "" };
        let query = format!(
            "SELECT *{omit_clause} FROM {table} WHERE meta::id(id) IN $ids; \
             DELETE FROM {table} WHERE meta::id(id) IN $ids;",
            table = table,
            omit_clause = omit_clause,
        );

        let ids_value = serde_json::to_value(ids).unwrap_or(serde_json::json!([]));
        let mut response = self.query_bind(&query, ("ids", ids_value)).await?;
        let result: Vec<serde_json::Value> = response.take(0)?;
        Ok(result)
    }

    /// Update the status field of multiple records in a table.
    /// Table name is validated against an allowlist. Status is parameterized.
    pub async fn batch_update_status(&self, table: &str, ids: &[String], status: &str) -> Result<usize, Error> {
        Self::validate_table_name(table)?;
        if ids.is_empty() {
            return Ok(0);
        }

        let query = format!(
            "UPDATE {table} SET status = $status, time.updated_at = time::now() \
             WHERE meta::id(id) IN $ids RETURN AFTER",
            table = table,
        );

        let mut bindings = serde_json::Map::new();
        bindings.insert("status".into(), serde_json::Value::String(status.to_string()));
        bindings.insert("ids".into(), serde_json::to_value(ids).unwrap_or(serde_json::json!([])));

        let mut response = self.query_bind_map(&query, bindings).await?;
        let result: Vec<serde_json::Value> = response.take(0)?;
        Ok(result.len())
    }

    /// Validate table name against allowlist to prevent injection.
    fn validate_table_name(table: &str) -> Result<(), Error> {
        match table {
            "projects" | "company" | "contacts" | "fee" => Ok(()),
            _ => Err(Error::thrown(format!("Invalid table name: {}", table))),
        }
    }

    // ==================== Country/Reference Operations ====================

    /// Resolve a country name/code/alias to its canonical record using the fn::resolve_country function.
    /// Returns None if no match is found.
    pub async fn resolve_country(&self, input: &str) -> Result<Option<serde_json::Value>, Error> {
        let query = "RETURN fn::resolve_country($input);";
        let mut response = self.query_bind(query, ("input", input.to_string())).await?;
        let result: Option<serde_json::Value> = response.take(0)?;
        Ok(result)
    }

    pub async fn search_countries(&self, query_str: &str) -> Result<Vec<serde_json::Value>, Error> {
        // Validate input length
        if query_str.is_empty() || query_str.len() > 50 {
            info!("Country search query invalid or too long, returning empty results");
            return Ok(vec![]);
        }

        // Use parameterized query to prevent SQL injection
        let search_query = r#"
            SELECT name, name_formal, name_official, code, code_alt, dial_code
            FROM country
            WHERE (name IS NOT NONE AND string::lower(name) CONTAINS string::lower($search))
               OR (name_formal IS NOT NONE AND string::lower(name_formal) CONTAINS string::lower($search))
               OR (name_official IS NOT NONE AND string::lower(name_official) CONTAINS string::lower($search))
               OR (code IS NOT NONE AND string::lower(code) CONTAINS string::lower($search))
               OR (code_alt IS NOT NONE AND string::lower(code_alt) CONTAINS string::lower($search))
               OR (dial_code IS NOT NONE AND string::contains(dial_code, $search))
            ORDER BY name ASC
            LIMIT 15
        "#;

        info!("Executing country search query with parameterized binding");

        let mut response = self.query_bind(search_query, ("search", query_str.to_string())).await?;
        response.take(0)
    }
}
