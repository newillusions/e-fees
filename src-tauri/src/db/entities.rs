// SurrealDB Entity Definitions for Rust
// Auto-generated from database schema

use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

// Enums
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ProjectActivity {
    #[serde(rename = "Design and Consultancy")]
    DesignAndConsultancy,
    Management,
    Construction,
    Maintenance,
    Research,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ProjectStatus {
    Draft,
    #[serde(rename = "RFP")]
    RFP,
    Active,
    #[serde(rename = "On Hold")]
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ProjectStage {
    Concept,
    #[serde(rename = "Design Development")]
    DesignDevelopment,
    Documentation,
    Tender,
    Construction,
    Handover,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RfpStatus {
    Draft,
    Active,
    Sent,
    Awarded,
    Lost,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RfpStage {
    Draft,
    Prepared,
    Sent,
    #[serde(rename = "Under Review")]
    UnderReview,
    Clarification,
    Negotiation,
    Awarded,
    Lost,
}

// Base types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeInfo {
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNumber {
    pub year: u32,    // 20-50
    pub country: u32, // dial code
    pub seq: u32,     // 1-999
    pub id: String,   // computed: YY-CCCNN
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub revision_number: i32,
    pub revision_date: Datetime,
    pub author_email: String,
    pub author_name: String,
    pub notes: String,
}

// Main entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    pub name_short: String,
    pub activity: ProjectActivity,
    pub package: String,
    pub status: ProjectStatus,
    #[serde(default = "default_project_stage")]
    pub stage: ProjectStage,
    pub area: String,
    pub city: String,
    pub country: String,
    pub folder: String,
    pub number: ProjectNumber,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rfp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    pub number: String,
    pub project_id: Thing,
    pub company_id: Thing,
    pub contact_id: Thing,
    #[serde(default = "default_rfp_status")]
    pub status: RfpStatus,
    #[serde(default = "default_rfp_stage")]
    pub stage: RfpStage,
    pub issue_date: String, // YYMMDD format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strap_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staff_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<i32>, // auto-computed
    #[serde(default)]
    pub revisions: Vec<Revision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reg_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company: Thing,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>, // auto-computed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub name: String,
    pub name_formal: String,
    pub name_official: String,
    pub code: String,
    pub code_alt: String,
    pub dial_code: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<Thing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    pub code: String,
    pub name: String,
}

// Create types (for inserting new records)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreate {
    pub name: String,
    pub name_short: String,
    pub activity: ProjectActivity,
    pub package: String,
    pub status: ProjectStatus,
    pub stage: ProjectStage,
    pub area: String,
    pub city: String,
    pub country: String,
    pub folder: String,
    pub number: ProjectNumber,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfpCreate {
    pub name: String,
    pub number: String,
    pub project_id: Thing,
    pub company_id: Thing,
    pub contact_id: Thing,
    pub status: RfpStatus,
    pub stage: RfpStage,
    pub issue_date: String,
    pub activity: Option<String>,
    pub package: Option<String>,
    pub strap_line: Option<String>,
    pub staff_name: Option<String>,
    pub staff_email: Option<String>,
    pub staff_phone: Option<String>,
    pub staff_position: Option<String>,
    pub revisions: Vec<Revision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyCreate {
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
    pub reg_no: Option<String>,
    pub tax_no: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCreate {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company: Thing,
}

// Default functions
fn default_project_stage() -> ProjectStage {
    ProjectStage::Concept
}

fn default_rfp_status() -> RfpStatus {
    RfpStatus::Draft
}

fn default_rfp_stage() -> RfpStage {
    RfpStage::Draft
}

// Helper implementations
impl ProjectNumber {
    pub fn new(year: u32, country: u32, seq: u32) -> Self {
        let id = format!("{:02}-{}{:02}", year, country, seq);
        Self { year, country, seq, id }
    }
    
    pub fn parse(id: &str) -> Result<Self, String> {
        let parts: Vec<&str> = id.split('-').collect();
        if parts.len() != 2 {
            return Err("Invalid project ID format".to_string());
        }
        
        let year = parts[0].parse::<u32>()
            .map_err(|_| "Invalid year")?;
        
        if parts[1].len() != 5 {
            return Err("Invalid country/seq format".to_string());
        }
        
        let country = parts[1][..3].parse::<u32>()
            .map_err(|_| "Invalid country code")?;
        let seq = parts[1][3..].parse::<u32>()
            .map_err(|_| "Invalid sequence")?;
        
        Ok(Self::new(year, country, seq))
    }
}

// Validation trait
pub trait Validate {
    fn validate(&self) -> Result<(), Vec<String>>;
}

impl Validate for ProjectCreate {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.name.is_empty() {
            errors.push("Project name cannot be empty".to_string());
        }
        
        if self.number.year < 20 || self.number.year > 50 {
            errors.push("Year must be between 20 and 50".to_string());
        }
        
        if self.number.seq < 1 || self.number.seq > 999 {
            errors.push("Sequence must be between 1 and 999".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for RfpCreate {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.name.is_empty() {
            errors.push("RFP name cannot be empty".to_string());
        }
        
        if self.issue_date.len() != 6 || !self.issue_date.chars().all(|c| c.is_numeric()) {
            errors.push("Issue date must be 6 digits in YYMMDD format".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for CompanyCreate {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.name.is_empty() {
            errors.push("Company name cannot be empty".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for ContactCreate {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if !self.email.contains('@') {
            errors.push("Invalid email format".to_string());
        }
        
        if !self.phone.contains('+') || self.phone.is_empty() {
            errors.push("Phone must contain '+' and not be empty".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
