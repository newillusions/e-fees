//! Live end-to-end check: a fixture fee record, through the manifest builder,
//! to a PDF rendered by a real fp-docbuilder service.
//!
//! SKIPPED unless `DOCBUILDER_URL` is set, so `cargo test` in CI and on a
//! laptop without the service stays green:
//!
//! ```bash
//! DOCBUILDER_URL=http://10.0.21.85:8080 \
//!   cargo test -p e-fees-core --test fp_docbuilder_live -- --nocapture
//! ```
//!
//! Unlike `fp_gotenberg_live.rs`, this test needs no `FP_TEMPLATE_ROOT`,
//! python3, or local browser: the service owns fill, the three pre-issue
//! gates, and rendering. This process only builds the manifest
//! (`export::fp_manifest`, pure Rust) and the variables JSON
//! (`export::build_fee_json`), then POSTs both as multipart — exactly what
//! `e-fees-api`'s route does when `DOCBUILDER_URL` is set. The manifest alone
//! is not a valid request: its frontmatter's `vars_source` resolves relative
//! to the manifest, so the vars part must travel with it under that exact
//! filename.

use e_fees_core::export::build_fee_json;
use e_fees_core::export::docbuilder::{DocbuilderClient, DocbuilderConfig};
use e_fees_core::export::fp_manifest::{build_fp_manifest, FpManifestOptions, FpNarrative};
use e_fees_core::models::common::TimeStamps;
use e_fees_core::models::fee::{
    Discipline, Fee, PostContractItem, PricingBreakdown, PricingCell, PricingConfig, Stage,
};
use e_fees_core::models::project::{Project, ProjectNumber};
use e_fees_core::models::{Company, Contact};
use surrealdb::types::RecordId;
use surrealdb_types::Datetime;

const NUMBER: &str = "25-96501";

fn timestamps() -> TimeStamps {
    TimeStamps {
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    }
}

fn pricing() -> PricingBreakdown {
    let disciplines = vec![
        Discipline {
            id: "d1".into(),
            name: "Lighting".into(),
            percentage: 60.0,
            order: 1,
        },
        Discipline {
            id: "d2".into(),
            name: "Audio".into(),
            percentage: 40.0,
            order: 2,
        },
    ];
    let stages = vec![
        Stage {
            id: "s1".into(),
            name: "Concept Design".into(),
            code: "CON".into(),
            percentage: 25.0,
            order: 1,
            is_post_contract: false,
        },
        Stage {
            id: "s2".into(),
            name: "Detailed Design".into(),
            code: "DD".into(),
            percentage: 75.0,
            order: 2,
            is_post_contract: false,
        },
        Stage {
            id: "p1".into(),
            name: "Site Meetings".into(),
            code: "SM".into(),
            percentage: 0.0,
            order: 10,
            is_post_contract: true,
        },
    ];
    let cells = disciplines
        .iter()
        .flat_map(|d| {
            vec![
                PricingCell {
                    discipline_id: d.id.clone(),
                    stage_id: "s1".into(),
                    amount: 1000.0,
                    override_amount: None,
                },
                PricingCell {
                    discipline_id: d.id.clone(),
                    stage_id: "s2".into(),
                    amount: 2000.0,
                    override_amount: None,
                },
            ]
        })
        .collect();
    PricingBreakdown {
        config: PricingConfig {
            currency: "AED".into(),
            ..Default::default()
        },
        disciplines,
        stages,
        cells,
        ..Default::default()
    }
}

fn fee() -> Fee {
    let mut fee = Fee {
        id: None,
        name: "Fee Proposal".into(),
        number: "25-96501-FP-1".into(),
        rev: 1,
        status: "Draft".into(),
        issue_date: "250820".into(),
        activity: "Design and Consultancy".into(),
        package: "Sensory Systems".into(),
        project_id: RecordId::new("projects", "25_96501"),
        company_id: RecordId::new("company", "KCC"),
        contact_id: RecordId::new("contacts", "c1"),
        staff_name: "Martin Robert".into(),
        staff_email: "martin@emittiv.com".into(),
        staff_phone: "+971 5858 555 69".into(),
        staff_position: "Lighting Director".into(),
        strap_line: "sensory design studio".into(),
        revisions: vec![],
        time: timestamps(),
        pricing: None,
        post_contract_items: Some(vec![PostContractItem {
            id: "pc1".into(),
            stage_id: "p1".into(),
            description: "Site Meetings".into(),
            quantity: 10.0,
            unit: "visit".into(),
            rate: 5200.0,
            amount: 52000.0,
        }]),
        reimbursable_costs: None,
        payment_schedule: None,
        pricing_revisions: None,
        current_revision_number: None,
        current_release_number: None,
        import_source: None,
    };
    fee.pricing = Some(e_fees_core::models::common::json_to_dbvalue(
        &serde_json::to_value(pricing()).unwrap(),
    ));
    fee
}

fn project() -> Project {
    Project {
        id: None,
        name: "Kalba Creek Centre".into(),
        name_short: "KCC".into(),
        status: "Concept Design".into(),
        area: "12,000 sq m".into(),
        city: "Kalba".into(),
        country: "U.A.E.".into(),
        folder: "/projects/25-96501".into(),
        number: ProjectNumber {
            year: 25,
            country: 965,
            seq: 1,
            id: "25_96501".into(),
        },
        time: timestamps(),
    }
}

fn company() -> Company {
    Company {
        id: None,
        name: "Kalba Development LLC".into(),
        name_short: "KDL".into(),
        abbreviation: "KDL".into(),
        city: "Kalba".into(),
        country: "U.A.E.".into(),
        reg_no: None,
        tax_no: None,
        time: timestamps(),
    }
}

fn contact() -> Contact {
    Contact {
        id: None,
        first_name: None,
        last_name: None,
        full_name: Some("Sara Nasser".into()),
        email: Some("sara@example.com".into()),
        phone: Some("+971 4 000 0000".into()),
        position: Some("Project Manager".into()),
        company: None,
        time: None,
    }
}

#[test]
fn manifest_renders_through_a_live_docbuilder_service() {
    let Some(config) = DocbuilderConfig::from_env() else {
        eprintln!("SKIPPED: set DOCBUILDER_URL to run this live end-to-end test");
        return;
    };
    eprintln!(
        "docbuilder: {} (timeout {}s, token set: {})",
        config.base_url,
        config.timeout.as_secs(),
        config.token.is_some()
    );

    // ---- fee record to manifest (pure Rust, no external process) ----
    let mut options = FpManifestOptions::for_number(NUMBER);
    options.narrative = FpNarrative {
        project_details: None,
        reference_documents: Some(vec!["Request for proposal received 20 August 2025".into()]),
        areas: Some("- All public areas and the external facade".into()),
        assumptions: None,
    };
    let (fee, project, company, contact) = (fee(), project(), company(), contact());
    let manifest = build_fp_manifest(&fee, &project, &company, &contact, &options)
        .expect("fixtures must produce a manifest");

    // ---- one call: the service owns fill, the three gates, and render ----
    // The manifest's frontmatter names vars_source, and the service resolves
    // that path relative to the manifest - so both parts must travel
    // together, the vars part under exactly that filename.
    let vars_json =
        serde_json::to_string_pretty(&build_fee_json(&fee, &project, &company, &contact))
            .expect("variables must serialise");
    let client = DocbuilderClient::new(config);
    let filename = format!("{NUMBER}-FP.pdf");
    let output = client
        .render_proposal(
            &manifest.markdown,
            &vars_json,
            &options.vars_source,
            Some(&filename),
            None,
        )
        .unwrap_or_else(|e| panic!("docbuilder render failed: {e}"));

    assert!(output.pdf.starts_with(b"%PDF"), "response is not a PDF");
    assert!(
        output.pages.is_some_and(|p| p > 0),
        "expected X-Docbuilder-Pages to report at least one page, got {:?}",
        output.pages
    );
    eprintln!(
        "rendered {} bytes, {:?} page(s), gates={:?}, template_ref={:?}, {} advisory issue-check hit(s)",
        output.pdf.len(),
        output.pages,
        output.gates,
        output.template_ref,
        output.issue_hits(),
    );
}
