//! Fee entity and pricing types.

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};
use surrealdb_types::Value as DbValue;

use super::common::{dbvalue_to_json, json_to_dbvalue, TimeStamps};

// ============================================================================
// OPT_DBVALUE_JSON SERDE MODULE
// ============================================================================

/// Serde module for Option<DbValue> fields that converts to/from plain JSON.
/// Use with `#[serde(default, with = "opt_dbvalue_json")]` on struct fields.
pub mod opt_dbvalue_json {
    use super::{dbvalue_to_json, json_to_dbvalue};
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
    use surrealdb_types::Value as DbValue;

    pub fn serialize<S>(val: &Option<DbValue>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            None => serializer.serialize_none(),
            Some(v) => dbvalue_to_json(v).serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DbValue>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<serde_json::Value> = Option::deserialize(deserializer)?;
        Ok(opt.map(|v| json_to_dbvalue(&v)))
    }
}

// ============================================================================
// FEE ENTITY
// ============================================================================

/// Fee entity representing fee proposals.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Fee {
    pub id: Option<RecordId>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub rev: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub issue_date: String,
    #[serde(default)]
    pub activity: String,
    #[serde(default)]
    pub package: String,
    pub project_id: RecordId,
    pub company_id: RecordId,
    pub contact_id: RecordId,
    #[serde(default)]
    pub staff_name: String,
    #[serde(default)]
    pub staff_email: String,
    #[serde(default)]
    pub staff_phone: String,
    #[serde(default)]
    pub staff_position: String,
    #[serde(default)]
    pub strap_line: String,
    #[serde(default)]
    pub revisions: Vec<Revision>,
    pub time: TimeStamps,

    // Pricing fields — use DbValue for SurrealDB v3 binary protocol compatibility.
    // serde_json::Value CANNOT handle native SurrealDB types from binary protocol.
    // DbValue handles binary deserialization natively via SurrealValue trait.
    // Custom serde via opt_dbvalue_json converts to/from plain JSON for Tauri IPC.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "opt_dbvalue_json"
    )]
    pub pricing: Option<DbValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_contract_items: Option<Vec<PostContractItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reimbursable_costs: Option<Vec<ReimbursableCost>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "opt_dbvalue_json"
    )]
    pub payment_schedule: Option<DbValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_revisions: Option<Vec<PricingRevision>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_revision_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_release_number: Option<i64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "opt_dbvalue_json"
    )]
    pub import_source: Option<DbValue>,
}

impl Fee {
    /// Convert the native DbValue pricing back to a typed PricingBreakdown.
    /// DbValue → plain JSON (via dbvalue_to_json) → PricingBreakdown (via serde),
    /// which handles int→float coercion automatically.
    pub fn pricing_typed(&self) -> Option<PricingBreakdown> {
        self.pricing.as_ref().and_then(|v| {
            let json = dbvalue_to_json(v);
            serde_json::from_value(json).ok()
        })
    }

    /// Replace the stages in the pricing breakdown, preserving all other pricing data.
    /// If pricing is None, creates a default PricingBreakdown with the given stages.
    pub fn set_pricing_stages(&mut self, stages: Vec<Stage>) {
        let mut breakdown = self.pricing_typed().unwrap_or_default();
        breakdown.stages = stages;
        if let Ok(json) = serde_json::to_value(&breakdown) {
            self.pricing = Some(json_to_dbvalue(&json));
        }
    }
}

/// Fee creation struct.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FeeCreate {
    pub name: String,
    pub number: String,
    pub rev: i64,
    pub status: String,
    pub issue_date: String,
    pub activity: String,
    pub package: String,
    pub project_id: String,
    pub company_id: String,
    pub contact_id: String,
    pub staff_name: String,
    pub staff_email: String,
    pub staff_phone: String,
    pub staff_position: String,
    pub strap_line: String,
    #[serde(default)]
    pub revisions: Vec<Revision>,

    // Pricing fields (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<PricingBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_contract_items: Option<Vec<PostContractItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reimbursable_costs: Option<Vec<ReimbursableCost>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_schedule: Option<PaymentSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_revisions: Option<Vec<PricingRevision>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_revision_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_release_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_source: Option<serde_json::Value>,
}

/// Fee update struct.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct FeeUpdate {
    pub name: String,
    pub number: String,
    pub rev: i64,
    pub status: String,
    #[serde(default)]
    pub issue_date: String,
    pub activity: Option<String>,
    pub package: Option<String>,
    pub project_id: String,
    pub company_id: String,
    pub contact_id: String,
    pub staff_name: Option<String>,
    pub staff_email: Option<String>,
    pub staff_phone: Option<String>,
    pub staff_position: Option<String>,
    pub strap_line: Option<String>,
    #[serde(default)]
    pub revisions: Vec<Revision>,

    // Pricing fields (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<PricingBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_contract_items: Option<Vec<PostContractItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reimbursable_costs: Option<Vec<ReimbursableCost>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_schedule: Option<PaymentSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_revisions: Option<Vec<PricingRevision>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_revision_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_release_number: Option<i64>,
}

/// Revision entry for fee proposals.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Revision {
    pub revision_number: i64,
    pub revision_date: String,
    pub author_email: String,
    pub author_name: String,
    pub notes: String,
}

// ============================================================================
// PRICING STRUCTURES
// ============================================================================

/// Discipline allocation for fee breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Discipline {
    pub id: String,
    pub name: String,
    pub percentage: f64,
    pub order: i64,
}

/// Design stage definition.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Stage {
    pub id: String,
    pub name: String,
    pub code: String,
    pub percentage: f64,
    pub order: i64,
    pub is_post_contract: bool,
}

/// Pricing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PricingConfig {
    #[serde(default)]
    pub target_fee: f64,
    #[serde(default)]
    pub buffer_percent: f64,
    #[serde(default)]
    pub quoted_fee: f64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub vat_percent: f64,
    #[serde(default)]
    pub vat_included: bool,
    #[serde(default)]
    pub mobilisation_percent: f64,
}

/// Discipline x Stage pricing cell.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PricingCell {
    pub discipline_id: String,
    pub stage_id: String,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_amount: Option<f64>,
}

/// Reimbursable cost/expense.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReimbursableCost {
    pub id: String,
    pub description: String,
    pub stage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discipline_id: Option<String>,
    pub base_cost: f64,
    pub markup_percent: f64,
    pub cost_to_client: f64,
    pub date_incurred: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Post-contract line item (different structure from design stages).
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PostContractItem {
    pub id: String,
    pub stage_id: String,
    pub description: String,
    pub quantity: f64,
    pub unit: String,
    pub rate: f64,
    pub amount: f64,
}

/// Payment schedule entry.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PaymentScheduleEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub payment_type: String, // "mobilisation", "milestone", "final"
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_percentage: Option<f64>,
    pub amount: f64,
    pub percentage_of_total: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    pub status: String, // "pending", "invoiced", "paid"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_date: Option<String>,
}

/// Payment tracking summary.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PaymentSchedule {
    pub entries: Vec<PaymentScheduleEntry>,
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub total_outstanding: f64,
}

/// Complete pricing breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PricingBreakdown {
    #[serde(default)]
    pub config: PricingConfig,
    #[serde(default)]
    pub disciplines: Vec<Discipline>,
    #[serde(default)]
    pub stages: Vec<Stage>,
    #[serde(default)]
    pub cells: Vec<PricingCell>,
    #[serde(default)]
    pub costs: Vec<ReimbursableCost>,
    #[serde(default)]
    pub design_phase_total: f64,
    #[serde(default)]
    pub post_contract_total: f64,
    #[serde(default)]
    pub costs_total: f64,
    #[serde(default)]
    pub subtotal: f64,
    #[serde(default)]
    pub vat_amount: f64,
    #[serde(default)]
    pub grand_total: f64,
}

/// Pricing update struct - for updating only pricing-related fields on a fee.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PricingUpdate {
    pub pricing: Option<PricingBreakdown>,
    pub post_contract_items: Option<Vec<PostContractItem>>,
    pub reimbursable_costs: Option<Vec<ReimbursableCost>>,
    pub payment_schedule: Option<PaymentSchedule>,
}

/// Pricing revision tracking (append-only).
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PricingRevision {
    pub id: String,
    pub fee_id: String,
    pub revision_number: i64,
    pub created_at: String,
    pub created_by: String,
    pub change_summary: String,
    pub pricing_snapshot: PricingBreakdown,
    pub is_client_release: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_number: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::TimeStamps;
    use surrealdb::types::RecordId;
    use surrealdb_types::Datetime;

    fn make_fee() -> Fee {
        Fee {
            id: None,
            name: String::new(),
            number: "25-97101-1".to_string(),
            rev: 0,
            status: "Draft".to_string(),
            issue_date: "202601".to_string(),
            activity: "Test".to_string(),
            package: String::new(),
            project_id: RecordId::new("projects", "test"),
            company_id: RecordId::new("company", "test"),
            contact_id: RecordId::new("contacts", "test"),
            staff_name: String::new(),
            staff_email: String::new(),
            staff_phone: String::new(),
            staff_position: String::new(),
            strap_line: String::new(),
            revisions: vec![],
            time: TimeStamps {
                created_at: Datetime::default(),
                updated_at: Datetime::default(),
            },
            pricing: None,
            post_contract_items: None,
            reimbursable_costs: None,
            payment_schedule: None,
            pricing_revisions: None,
            current_revision_number: None,
            current_release_number: None,
            import_source: None,
        }
    }

    #[test]
    fn test_set_pricing_stages_on_empty_pricing() {
        let mut fee = make_fee();
        assert!(fee.pricing.is_none());

        let stages = vec![Stage {
            id: "sd-01".to_string(),
            name: "Schematic Design".to_string(),
            code: "SD".to_string(),
            percentage: 25.0,
            order: 1,
            is_post_contract: false,
        }];

        fee.set_pricing_stages(stages);

        let breakdown = fee.pricing_typed().expect("pricing should be set");
        assert_eq!(breakdown.stages.len(), 1);
        assert_eq!(breakdown.stages[0].name, "Schematic Design");
    }

    #[test]
    fn test_set_pricing_stages_preserves_existing_pricing() {
        let mut fee = make_fee();

        let mut breakdown = PricingBreakdown::default();
        breakdown.config.quoted_fee = 50000.0;
        breakdown.stages = vec![Stage {
            id: "sd-01".to_string(),
            name: "Schematic Design".to_string(),
            code: "SD".to_string(),
            percentage: 25.0,
            order: 1,
            is_post_contract: false,
        }];
        let json = serde_json::to_value(&breakdown).unwrap();
        fee.pricing = Some(json_to_dbvalue(&json));

        let new_stages = vec![
            Stage {
                id: "sd-01".to_string(),
                name: "Schematic Design".to_string(),
                code: "SD".to_string(),
                percentage: 25.0,
                order: 1,
                is_post_contract: false,
            },
            Stage {
                id: "bim-01".to_string(),
                name: "BIM Coordination".to_string(),
                code: "BC".to_string(),
                percentage: 0.0,
                order: 2,
                is_post_contract: false,
            },
        ];
        fee.set_pricing_stages(new_stages);

        let result = fee.pricing_typed().unwrap();
        assert_eq!(result.stages.len(), 2);
        assert_eq!(result.config.quoted_fee, 50000.0); // preserved
    }
}
