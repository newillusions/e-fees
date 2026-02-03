//! Database client abstraction for HTTP and WebSocket connections.

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::engine::remote::http::{Client as HttpClient, Http};
use surrealdb::opt::auth::{Root, Namespace, Database};
use surrealdb::{Error, Surreal};
use log::{info, warn, error};
use chrono::Utc;

use super::types::{
    Project, NewProject, Company, CompanyCreate, Contact, ContactCreate,
    Fee, FeeCreate, FeeUpdate, PricingUpdate,
};
use crate::commands::{CompanyUpdate, ContactUpdate, ProjectUpdate};

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
            info!("Attempting {} WebSocket connection to {}",
                  if is_secure { "secure (WSS)" } else { "unencrypted (WS)" }, url);

            let connection_address = url
                .strip_prefix("ws://")
                .or_else(|| url.strip_prefix("wss://"))
                .unwrap_or(url);

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

                    let http_url = if url.starts_with("ws://") {
                        url.replace("ws://", "http://")
                    } else {
                        url.replace("wss://", "https://")
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
            DatabaseClient::Http(client) => client.use_ns(namespace).use_db(database).await,
            DatabaseClient::WebSocket(client) => client.use_ns(namespace).use_db(database).await,
        }
    }

    /// Select all records from a table.
    pub async fn select<T>(&self, table: &str) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        delegate_to_client!(self, select, table)
    }

    /// Execute a raw query.
    pub async fn query(&self, query: &str) -> Result<surrealdb::Response, Error> {
        delegate_to_client!(self, query, query)
    }

    /// Execute a parameterized query with bindings (SQL injection safe).
    pub async fn query_bind<T: serde::Serialize + 'static>(&self, query: &str, bindings: T) -> Result<surrealdb::Response, Error> {
        match self {
            DatabaseClient::Http(client) => client.query(query).bind(bindings).await,
            DatabaseClient::WebSocket(client) => client.query(query).bind(bindings).await,
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

        let set_clauses = vec![
            format!("name = '{}'", project.name.replace("'", "''")),
            format!("name_short = '{}'", project.name_short.replace("'", "''")),
            format!("status = '{}'", project.status.replace("'", "''")),
            format!("area = '{}'", project.area.replace("'", "''")),
            format!("city = '{}'", project.city.replace("'", "''")),
            format!("country = '{}'", project.country.replace("'", "''")),
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
               string::lowercase(name) CONTAINS string::lowercase('{}') OR
               string::lowercase(name_short) CONTAINS string::lowercase('{}') OR
               string::lowercase(number.id) CONTAINS string::lowercase('{}') OR
               string::lowercase(city) CONTAINS string::lowercase('{}') OR
               string::lowercase(area) CONTAINS string::lowercase('{}') OR
               string::lowercase(country) CONTAINS string::lowercase('{}') OR
               string::lowercase(folder) CONTAINS string::lowercase('{}')
               ORDER BY time.created_at DESC"#,
            escaped_query, escaped_query, escaped_query, escaped_query,
            escaped_query, escaped_query, escaped_query
        );

        let mut response = self.query(&search_query).await?;
        response.take(0)
    }

    // ==================== Company Operations ====================

    pub async fn create_company(&self, company: CompanyCreate) -> Result<Option<Company>, Error> {
        let query = format!(
            "CREATE company:{} SET name = '{}', name_short = '{}', abbreviation = '{}', city = '{}', country = '{}', reg_no = {}, tax_no = {}, time = {{ created_at: time::now(), updated_at: time::now() }}",
            company.abbreviation,
            company.name.replace("'", "''"),
            company.name_short.replace("'", "''"),
            company.abbreviation.replace("'", "''"),
            company.city.replace("'", "''"),
            company.country.replace("'", "''"),
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

        let query = format!(
            "CREATE fee:{} SET name = '{}', number = '{}', rev = {}, project_id = projects:{}, company_id = company:{}, contact_id = contacts:{}, status = '{}', issue_date = '{}', activity = '{}', package = '{}', strap_line = '{}', staff_name = '{}', staff_email = '{}', staff_phone = '{}', staff_position = '{}', revisions = [], time = {{ created_at: time::now(), updated_at: time::now() }}",
            fee_id,
            fee.name.replace("'", "''"),
            fee.number.replace("'", "''"),
            fee.rev,
            fee.project_id.replace("'", "''"),
            fee.company_id.replace("'", "''"),
            fee.contact_id.replace("'", "''"),
            fee.status.replace("'", "''"),
            fee.issue_date.replace("'", "''"),
            fee.activity.replace("'", "''"),
            fee.package.replace("'", "''"),
            fee.strap_line.replace("'", "''"),
            fee.staff_name.replace("'", "''"),
            fee.staff_email.replace("'", "''"),
            fee.staff_phone.replace("'", "''"),
            fee.staff_position.replace("'", "''")
        );

        info!("Executing Fee creation query");

        let mut response = self.query(&query).await?;
        let result: Result<Vec<Fee>, _> = response.take(0);
        match result {
            Ok(mut fees) => Ok(fees.pop()),
            Err(e) => Err(e),
        }
    }

    pub async fn update_fee(&self, id: &str, fee: FeeUpdate) -> Result<Option<Fee>, Error> {
        info!("DatabaseClient::update_fee called with id: '{}'", id);

        let query = format!(
            "UPDATE fee:{} SET name = '{}', number = '{}', rev = {}, project_id = projects:{}, company_id = company:{}, contact_id = contacts:{}, status = '{}', issue_date = '{}', activity = '{}', package = '{}', strap_line = '{}', staff_name = '{}', staff_email = '{}', staff_phone = '{}', staff_position = '{}', time = {{ created_at: time.created_at OR time::now(), updated_at: time::now() }} RETURN AFTER",
            id,
            fee.name.replace("'", "''"),
            fee.number.replace("'", "''"),
            fee.rev,
            fee.project_id.replace("'", "''"),
            fee.company_id.replace("'", "''"),
            fee.contact_id.replace("'", "''"),
            fee.status.replace("'", "''"),
            fee.issue_date.replace("'", "''"),
            fee.activity.as_ref().unwrap_or(&String::new()).replace("'", "''"),
            fee.package.as_ref().unwrap_or(&String::new()).replace("'", "''"),
            fee.strap_line.as_ref().unwrap_or(&String::new()).replace("'", "''"),
            fee.staff_name.as_ref().unwrap_or(&String::new()).replace("'", "''"),
            fee.staff_email.as_ref().unwrap_or(&String::new()).replace("'", "''"),
            fee.staff_phone.as_ref().unwrap_or(&String::new()).replace("'", "''"),
            fee.staff_position.as_ref().unwrap_or(&String::new()).replace("'", "''")
        );

        info!("Executing update query");

        let mut response = self.query(&query).await?;
        let result: Result<Vec<Fee>, _> = response.take(0);
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
        delegate_delete!(self, "fee", id)
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

        // Create a nested time object with updated_at
        let mut time_obj = serde_json::Map::new();
        time_obj.insert("updated_at".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        update_obj.insert("time".to_string(), serde_json::Value::Object(time_obj));

        let update_value = serde_json::Value::Object(update_obj);

        // Use MERGE to update only the specified fields (including timestamp)
        let query = format!(
            "UPDATE fee:{} MERGE $data RETURN AFTER",
            id
        );

        info!("Executing pricing update with MERGE");

        let mut response = self.query_bind(&query, ("data", update_value)).await?;
        let result: Result<Vec<Fee>, _> = response.take(0);
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

    // ==================== Country/Reference Operations ====================

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
            WHERE (name IS NOT NONE AND string::lowercase(name) CONTAINS string::lowercase($search))
               OR (name_formal IS NOT NONE AND string::lowercase(name_formal) CONTAINS string::lowercase($search))
               OR (name_official IS NOT NONE AND string::lowercase(name_official) CONTAINS string::lowercase($search))
               OR (code IS NOT NONE AND string::lowercase(code) CONTAINS string::lowercase($search))
               OR (code_alt IS NOT NONE AND string::lowercase(code_alt) CONTAINS string::lowercase($search))
               OR (dial_code IS NOT NONE AND string::contains(<string>dial_code, $search))
            ORDER BY name ASC
            LIMIT 15
        "#;

        info!("Executing country search query with parameterized binding");

        let mut response = self.query_bind(search_query, ("search", query_str.to_string())).await?;
        response.take(0)
    }
}
