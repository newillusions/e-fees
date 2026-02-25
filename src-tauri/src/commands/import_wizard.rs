//! Import wizard commands for reading RFPs JSON exports and importing into E-Fees.
//!
//! Supports two proposal structures:
//! - Custom milestones (e.g., Hittin: M1-M4)
//! - Standard phased stages (e.g., Thakher: 4 phases x 6 stages)

use serde::{Deserialize, Serialize};
use tauri::State;
use log::{error, info, warn};
use std::path::Path;

use super::AppState;
use crate::db::types::record_key_string;

// ============================================================================
// IMPORT DATA STRUCTURES (matching RFPs JSON export format)
// ============================================================================

/// Staff rate entry from staff-rates.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStaffRate {
    pub role: String,
    pub hourly_rate: f64,
    pub daily_rate: f64,
    pub weekly_rate: f64,
    pub monthly_rate: f64,
}

/// Staff rates export file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffRatesExport {
    pub export_date: String,
    pub source_file: String,
    pub currency: String,
    pub rates: Vec<ImportStaffRate>,
}

/// Stage entry in a proposal (used for both custom milestones and standard stages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStage {
    pub name: String,
    pub percentage: f64,
    pub fee: f64,
    #[serde(default)]
    pub deliverables: Vec<String>,
}

/// Phase entry (Thakher-style: phase containing multiple stages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPhase {
    pub name: String,
    pub total_fee: f64,
    pub stages: Vec<ImportStage>,
    #[serde(default)]
    pub duration_weeks: Option<i32>,
}

/// Add-on entry (Hittin-style optional scope items)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAddOn {
    pub code: String,
    pub description: String,
    pub fee_aed: f64,
    #[serde(default)]
    pub fee_sar: Option<f64>,
}

/// Proposal data structure (covers both Hittin and Thakher formats)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProposal {
    pub project_name: String,
    pub project_number: String,
    pub client: String,
    pub lead_consultant: String,
    pub total_fee: f64,
    pub currency: String,
    #[serde(default)]
    pub stages: Vec<ImportStage>,
    #[serde(default)]
    pub phases: Vec<ImportPhase>,
    #[serde(default)]
    pub add_ons: Vec<ImportAddOn>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub conversion_rate: Option<f64>,
    #[serde(default)]
    pub base_scope_fee: Option<f64>,
}

/// Proposal export file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalExport {
    pub export_date: String,
    pub source_file: String,
    pub proposal: ImportProposal,
}

/// Fee methodology export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeMethodologyExport {
    pub export_date: String,
    pub methodology: FeeMethodology,
}

/// Fee methodology details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeMethodology {
    pub wht_formula: String,
    pub wht_rate: f64,
    pub currency_rates: CurrencyRates,
    pub default_stages: Vec<DefaultStageConfig>,
    pub overhead_rate: f64,
    pub margin_rate: f64,
    pub rounding: String,
}

/// Currency conversion rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyRates {
    #[serde(rename = "AED_to_SAR")]
    pub aed_to_sar: f64,
}

/// Default stage configuration from methodology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultStageConfig {
    pub name: String,
    pub percentage: f64,
}

/// Client/consultant record from clients-consultants.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportClient {
    pub name: String,
    #[serde(rename = "type")]
    pub client_type: String,
    pub contact: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub projects: Vec<String>,
}

/// Clients export file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientsExport {
    pub export_date: String,
    pub clients: Vec<ImportClient>,
}

// ============================================================================
// PREVIEW/SUMMARY STRUCTURES (sent to frontend for confirmation)
// ============================================================================

/// Summary of what will be imported, shown to user for confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub staff_rates: Option<StaffRatesPreview>,
    pub proposals: Vec<ProposalPreview>,
    pub methodology: Option<MethodologyPreview>,
    pub clients: Vec<ClientPreview>,
    pub warnings: Vec<String>,
}

/// Preview of staff rates import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffRatesPreview {
    pub count: usize,
    pub currency: String,
    pub roles: Vec<String>,
}

/// Preview of a single proposal import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalPreview {
    pub source_file: String,
    pub project_name: String,
    pub project_number: String,
    pub client: String,
    pub total_fee: f64,
    pub currency: String,
    pub stage_type: String, // "custom_milestones" or "phased_stages"
    pub stage_count: usize,
    pub phase_count: usize,
    pub add_on_count: usize,
    pub has_conversion_rate: bool,
}

/// Preview of methodology import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodologyPreview {
    pub wht_rate: f64,
    pub aed_to_sar: f64,
    pub overhead_rate: f64,
    pub margin_rate: f64,
    pub default_stage_count: usize,
}

/// Preview of a client record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPreview {
    pub name: String,
    pub client_type: String,
    pub project_count: usize,
}

// ============================================================================
// IMPORT RESULT STRUCTURES
// ============================================================================

/// Result of the import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub projects_created: usize,
    pub fees_created: usize,
    pub companies_created: usize,
    pub companies_skipped: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Scan a directory for RFPs JSON export files and return a preview.
#[tauri::command]
pub async fn import_scan_directory(directory: String) -> Result<ImportPreview, String> {
    info!("Scanning import directory: {}", directory);

    let dir_path = Path::new(&directory);
    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {}", directory));
    }
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", directory));
    }

    let mut preview = ImportPreview {
        staff_rates: None,
        proposals: Vec::new(),
        methodology: None,
        clients: Vec::new(),
        warnings: Vec::new(),
    };

    // Scan for known file patterns
    let staff_rates_path = dir_path.join("staff-rates.json");
    let methodology_path = dir_path.join("fee-methodology.json");
    let clients_path = dir_path.join("clients-consultants.json");

    // Parse staff rates
    if staff_rates_path.exists() {
        match parse_json_file::<StaffRatesExport>(&staff_rates_path) {
            Ok(data) => {
                preview.staff_rates = Some(StaffRatesPreview {
                    count: data.rates.len(),
                    currency: data.currency.clone(),
                    roles: data.rates.iter().map(|r| r.role.clone()).collect(),
                });
            }
            Err(e) => preview.warnings.push(format!("Failed to parse staff-rates.json: {}", e)),
        }
    }

    // Parse methodology
    if methodology_path.exists() {
        match parse_json_file::<FeeMethodologyExport>(&methodology_path) {
            Ok(data) => {
                preview.methodology = Some(MethodologyPreview {
                    wht_rate: data.methodology.wht_rate,
                    aed_to_sar: data.methodology.currency_rates.aed_to_sar,
                    overhead_rate: data.methodology.overhead_rate,
                    margin_rate: data.methodology.margin_rate,
                    default_stage_count: data.methodology.default_stages.len(),
                });
            }
            Err(e) => preview.warnings.push(format!("Failed to parse fee-methodology.json: {}", e)),
        }
    }

    // Parse clients
    if clients_path.exists() {
        match parse_json_file::<ClientsExport>(&clients_path) {
            Ok(data) => {
                preview.clients = data.clients.iter().map(|c| ClientPreview {
                    name: c.name.clone(),
                    client_type: c.client_type.clone(),
                    project_count: c.projects.len(),
                }).collect();
            }
            Err(e) => preview.warnings.push(format!("Failed to parse clients-consultants.json: {}", e)),
        }
    }

    // Scan for proposal files (any JSON file with a "proposal" key)
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                // Skip known non-proposal files
                if filename == "staff-rates.json"
                    || filename == "fee-methodology.json"
                    || filename == "clients-consultants.json"
                {
                    continue;
                }

                match parse_json_file::<ProposalExport>(&path) {
                    Ok(data) => {
                        let proposal = &data.proposal;
                        let (stage_type, stage_count, phase_count) = if !proposal.phases.is_empty() {
                            let total_stages: usize = proposal.phases.iter()
                                .map(|p| p.stages.len())
                                .sum();
                            ("phased_stages".to_string(), total_stages, proposal.phases.len())
                        } else {
                            ("custom_milestones".to_string(), proposal.stages.len(), 0)
                        };

                        preview.proposals.push(ProposalPreview {
                            source_file: filename,
                            project_name: proposal.project_name.clone(),
                            project_number: proposal.project_number.clone(),
                            client: proposal.client.clone(),
                            total_fee: proposal.total_fee,
                            currency: proposal.currency.clone(),
                            stage_type,
                            stage_count,
                            phase_count,
                            add_on_count: proposal.add_ons.len(),
                            has_conversion_rate: proposal.conversion_rate.is_some(),
                        });
                    }
                    Err(_) => {
                        // Not a proposal file, skip silently
                    }
                }
            }
        }
    }

    if preview.proposals.is_empty() && preview.staff_rates.is_none() && preview.clients.is_empty() {
        return Err("No importable JSON files found in the selected directory.".to_string());
    }

    info!(
        "Import scan complete: {} proposals, {} clients, staff_rates={}, methodology={}",
        preview.proposals.len(),
        preview.clients.len(),
        preview.staff_rates.is_some(),
        preview.methodology.is_some()
    );

    Ok(preview)
}

/// Execute the import of RFPs JSON data into E-Fees SurrealDB.
#[tauri::command]
pub async fn import_execute(
    directory: String,
    import_proposals: bool,
    import_companies: bool,
    state: State<'_, AppState>,
) -> Result<ImportResult, String> {
    info!("Executing import from: {}", directory);

    let dir_path = Path::new(&directory);
    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {}", directory));
    }

    let mut result = ImportResult {
        success: true,
        projects_created: 0,
        fees_created: 0,
        companies_created: 0,
        companies_skipped: 0,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Import companies first (proposals reference them)
    if import_companies {
        let clients_path = dir_path.join("clients-consultants.json");
        if clients_path.exists() {
            match parse_json_file::<ClientsExport>(&clients_path) {
                Ok(data) => {
                    import_companies_from_json(&data, &state, &mut result).await;
                }
                Err(e) => {
                    result.errors.push(format!("Failed to parse clients: {}", e));
                    result.success = false;
                }
            }
        }
    }

    // Import proposals
    if import_proposals {
        // Read methodology for defaults
        let methodology_path = dir_path.join("fee-methodology.json");
        let methodology = if methodology_path.exists() {
            parse_json_file::<FeeMethodologyExport>(&methodology_path).ok()
        } else {
            None
        };

        // Find and import proposal files
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if filename == "staff-rates.json"
                        || filename == "fee-methodology.json"
                        || filename == "clients-consultants.json"
                    {
                        continue;
                    }

                    if let Ok(data) = parse_json_file::<ProposalExport>(&path) {
                        import_proposal_from_json(
                            &data,
                            methodology.as_ref(),
                            &state,
                            &mut result,
                        ).await;
                    }
                }
            }
        }
    }

    if result.errors.is_empty() {
        info!("Import completed successfully: {} projects, {} fees, {} companies",
            result.projects_created, result.fees_created, result.companies_created);
    } else {
        warn!("Import completed with {} errors", result.errors.len());
        result.success = result.projects_created > 0 || result.companies_created > 0;
    }

    Ok(result)
}

// ============================================================================
// INTERNAL HELPERS
// ============================================================================

/// Parse a JSON file into a typed structure.
fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Import companies from clients-consultants.json into SurrealDB.
async fn import_companies_from_json(
    data: &ClientsExport,
    state: &State<'_, AppState>,
    result: &mut ImportResult,
) {
    let manager = state.read().await;

    // Get existing companies to avoid duplicates
    let existing_companies = match manager.get_companies().await {
        Ok(companies) => companies,
        Err(e) => {
            result.errors.push(format!("Failed to fetch existing companies: {}", e));
            return;
        }
    };

    let existing_names: Vec<String> = existing_companies.iter()
        .map(|c| c.name.to_lowercase())
        .collect();

    for client in &data.clients {
        // Skip if company already exists (case-insensitive match)
        if existing_names.contains(&client.name.to_lowercase()) {
            result.companies_skipped += 1;
            result.warnings.push(format!("Company '{}' already exists, skipping", client.name));
            continue;
        }

        // Create abbreviation from name (first 3 chars uppercase)
        let abbreviation = client.name.chars()
            .filter(|c| c.is_alphanumeric())
            .take(3)
            .collect::<String>()
            .to_uppercase();

        // Determine city and country from context
        let (city, country) = match client.client_type.as_str() {
            "client" => ("Riyadh".to_string(), "Saudi Arabia".to_string()),
            "consultant" => {
                if client.name.contains("Dubai") {
                    ("Dubai".to_string(), "UAE".to_string())
                } else {
                    ("".to_string(), "".to_string())
                }
            }
            _ => ("Dubai".to_string(), "UAE".to_string()),
        };

        let company_create = crate::db::CompanyCreate {
            name: client.name.clone(),
            name_short: abbreviation.clone(),
            abbreviation,
            city,
            country,
            reg_no: None,
            tax_no: None,
        };

        match manager.create_company(company_create).await {
            Ok(_) => {
                result.companies_created += 1;
                info!("Created company: {}", client.name);
            }
            Err(e) => {
                result.errors.push(format!("Failed to create company '{}': {}", client.name, e));
            }
        }
    }
}

/// Import a proposal from JSON into E-Fees as a project + fee record.
async fn import_proposal_from_json(
    data: &ProposalExport,
    methodology: Option<&FeeMethodologyExport>,
    state: &State<'_, AppState>,
    result: &mut ImportResult,
) {
    let manager = state.read().await;
    let proposal = &data.proposal;

    // Determine country code from currency
    let (country_code, city, country_name) = match proposal.currency.as_str() {
        "SAR" => (966, "Riyadh".to_string(), "Saudi Arabia".to_string()),
        _ => (971, "Dubai".to_string(), "UAE".to_string()),
    };

    // Parse project number (e.g., "1735" -> year=17, seq=35 -- or use as-is)
    let project_number_str = &proposal.project_number;
    let (year, seq) = if project_number_str.len() == 4 {
        let y: i64 = project_number_str[0..2].parse().unwrap_or(26);
        let s: i64 = project_number_str[2..4].parse().unwrap_or(1);
        (y, s)
    } else {
        (26, 1)
    };

    let project_id = format!("{:02}-{}{:02}", year, country_code, seq);

    // Check if project already exists
    match manager.get_project_by_id(&project_id).await {
        Ok(Some(_)) => {
            result.warnings.push(format!(
                "Project {} ({}) already exists, skipping project creation",
                project_id, proposal.project_name
            ));
        }
        Ok(None) => {
            // Create the project
            let new_project = crate::db::NewProject {
                name: proposal.project_name.clone(),
                name_short: proposal.project_name.chars().take(30).collect(),
                status: "RFP".to_string(),
                area: "".to_string(),
                city: city.clone(),
                country: country_name.clone(),
                folder: "".to_string(),
                venue_id: None,
                number: crate::db::ProjectNumber {
                    year,
                    country: country_code,
                    seq,
                    id: project_id.clone(),
                },
            };

            match manager.create_new_project(new_project).await {
                Ok(_) => {
                    result.projects_created += 1;
                    info!("Created project: {} ({})", proposal.project_name, project_id);
                }
                Err(e) => {
                    result.errors.push(format!(
                        "Failed to create project '{}': {}",
                        proposal.project_name, e
                    ));
                    return;
                }
            }
        }
        Err(e) => {
            result.errors.push(format!("Failed to check existing project: {}", e));
            return;
        }
    }

    // Build pricing breakdown from the proposal data
    let pricing = build_pricing_from_proposal(proposal, methodology);

    // Find or create the company for this proposal
    let company_id = find_company_id(&manager, &proposal.client).await
        .unwrap_or_else(|| "EMT".to_string());

    // Create the fee record
    let fee_create = crate::db::FeeCreate {
        name: proposal.project_name.clone(),
        number: format!("{}-R1", project_id),
        rev: 1,
        status: "Draft".to_string(),
        issue_date: chrono::Local::now().format("%y%m%d").to_string(),
        activity: "Design and Consultancy".to_string(),
        package: "Lighting Design".to_string(),
        project_id: project_id.clone(),
        company_id,
        contact_id: "".to_string(),
        staff_name: "".to_string(),
        staff_email: "".to_string(),
        staff_phone: "".to_string(),
        staff_position: "".to_string(),
        strap_line: "".to_string(),
        revisions: Vec::new(),
        pricing: Some(pricing),
        post_contract_items: None,
        reimbursable_costs: None,
        payment_schedule: None,
        pricing_revisions: None,
        current_revision_number: Some(1),
        current_release_number: Some(0),
        import_source: None,
    };

    match manager.create_fee(fee_create).await {
        Ok(_) => {
            result.fees_created += 1;
            info!("Created fee for project: {}", project_id);
        }
        Err(e) => {
            result.errors.push(format!(
                "Failed to create fee for '{}': {}",
                proposal.project_name, e
            ));
        }
    }
}

/// Build a PricingBreakdown from proposal JSON data.
fn build_pricing_from_proposal(
    proposal: &ImportProposal,
    methodology: Option<&FeeMethodologyExport>,
) -> crate::db::PricingBreakdown {
    let wht_rate = methodology.map(|m| m.methodology.wht_rate).unwrap_or(0.05);
    let is_sar = proposal.currency == "SAR";

    // Build stages from either phases or custom milestones
    let mut stages: Vec<crate::db::Stage> = Vec::new();
    let mut cells: Vec<crate::db::PricingCell> = Vec::new();
    let discipline_id = format!("disc_{}", chrono::Utc::now().timestamp_millis());

    if !proposal.phases.is_empty() {
        // Phased structure (Thakher-style): use first phase's stages as template
        if let Some(first_phase) = proposal.phases.first() {
            for (i, stage) in first_phase.stages.iter().enumerate() {
                let stage_id = format!("stage_{}_{}", chrono::Utc::now().timestamp_millis(), i);
                let code = stage.name.chars()
                    .filter(|c| c.is_uppercase())
                    .take(2)
                    .collect::<String>();

                stages.push(crate::db::Stage {
                    id: stage_id.clone(),
                    name: stage.name.clone(),
                    code: if code.is_empty() { format!("S{}", i + 1) } else { code },
                    percentage: stage.percentage * 100.0,
                    order: (i + 1) as i64,
                    is_post_contract: false,
                });

                // Calculate total fee across all phases for this stage position
                let stage_total: f64 = proposal.phases.iter()
                    .filter_map(|p| p.stages.get(i))
                    .map(|s| s.fee)
                    .sum();

                cells.push(crate::db::PricingCell {
                    discipline_id: discipline_id.clone(),
                    stage_id,
                    amount: stage_total,
                    override_amount: Some(stage_total),
                });
            }
        }
    } else {
        // Custom milestones (Hittin-style)
        for (i, stage) in proposal.stages.iter().enumerate() {
            let stage_id = format!("stage_{}_{}", chrono::Utc::now().timestamp_millis(), i);
            let code = format!("M{}", i + 1);

            stages.push(crate::db::Stage {
                id: stage_id.clone(),
                name: stage.name.clone(),
                code,
                percentage: stage.percentage * 100.0,
                order: (i + 1) as i64,
                is_post_contract: false,
            });

            cells.push(crate::db::PricingCell {
                discipline_id: discipline_id.clone(),
                stage_id,
                amount: stage.fee,
                override_amount: Some(stage.fee),
            });
        }
    }

    let disciplines = vec![
        crate::db::Discipline {
            id: discipline_id,
            name: "Lighting Design".to_string(),
            percentage: 100.0,
            order: 1,
        },
    ];

    let design_total: f64 = cells.iter().map(|c| c.override_amount.unwrap_or(c.amount)).sum();

    // Determine tax type: SAR proposals typically use withholding tax
    let tax_type_str = if is_sar { "withholding" } else { "vat" };

    crate::db::PricingBreakdown {
        config: crate::db::PricingConfig {
            target_fee: proposal.base_scope_fee.unwrap_or(proposal.total_fee),
            buffer_percent: 0.0,
            quoted_fee: proposal.base_scope_fee.unwrap_or(proposal.total_fee),
            currency: proposal.currency.clone(),
            vat_percent: if is_sar { wht_rate * 100.0 } else { 5.0 },
            vat_included: false,
            mobilisation_percent: 5.0,
        },
        disciplines,
        stages,
        cells,
        costs: Vec::new(),
        design_phase_total: design_total,
        post_contract_total: 0.0,
        costs_total: 0.0,
        subtotal: design_total,
        vat_amount: 0.0,
        grand_total: design_total,
    }
}

/// Find a company ID by name (case-insensitive partial match).
async fn find_company_id(
    manager: &crate::db::DatabaseManager,
    client_name: &str,
) -> Option<String> {
    match manager.get_companies().await {
        Ok(companies) => {
            let lower_name = client_name.to_lowercase();
            companies.iter()
                .find(|c| c.name.to_lowercase().contains(&lower_name)
                    || lower_name.contains(&c.name.to_lowercase()))
                .and_then(|c| c.id.as_ref())
                .map(|id| record_key_string(&id.key))
        }
        Err(_) => None,
    }
}
