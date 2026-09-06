//! Live end-to-end check: a fee record, through the fill engine, to a PDF
//! rendered by a real gotenberg service.
//!
//! SKIPPED unless BOTH environment variables are set, so `cargo test` in CI
//! and on a laptop without the toolchain stays green:
//!
//! ```bash
//! FP_TEMPLATE_ROOT=/path/to/gtm/fp-template \
//! GOTENBERG_URL=http://10.0.23.31:3000 \
//!   cargo test -p e-fees-core --test fp_gotenberg_live -- --nocapture
//! ```
//!
//! Why this test exists: the 2026-09-05 parity measurement rendered a
//! hand-filled proposal that was already committed in fp-template. The fill
//! engine's OWN output — different asset paths, different markup, generated
//! per fee — had never been pushed through gotenberg. That was the named gap;
//! this closes it.
//!
//! It needs no database: the fee, project, company and contact are fixtures,
//! and the manifest builder is pure. It DOES need python3 with pyyaml and
//! beautifulsoup4, and (today) a local Chromium, because fp-template's
//! `engine/measure.py` measures block heights in a real render.

use std::path::PathBuf;

use e_fees_core::export::build_fee_json;
use e_fees_core::export::fp_manifest::{build_fp_manifest, FpManifestOptions, FpNarrative};
use e_fees_core::export::fp_render::{
    fill_manifest, run_release_gates, select_renderer, FpRenderConfig,
};
use e_fees_core::export::gotenberg::{
    flatten_document, pdf_page_count, GotenbergConfig, INDEX_FILENAME,
};
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

/// Both variables must be present; otherwise the test reports why it skipped.
fn live_config() -> Option<(FpRenderConfig, GotenbergConfig)> {
    let fp = FpRenderConfig::from_env()?;
    let gotenberg = GotenbergConfig::from_env()?;
    Some((fp, gotenberg))
}

#[test]
fn fill_engine_output_renders_through_gotenberg() {
    let Some((fp_config, gotenberg_config)) = live_config() else {
        eprintln!(
            "SKIPPED: set FP_TEMPLATE_ROOT and GOTENBERG_URL to run this live end-to-end test"
        );
        return;
    };
    fp_config
        .validate()
        .expect("FP_TEMPLATE_ROOT must be an fp-template checkout");
    eprintln!(
        "gotenberg: {} (timeout {}s, tagged {}, cap {} bytes)",
        gotenberg_config.base_url,
        gotenberg_config.timeout.as_secs(),
        gotenberg_config.tagged_pdf,
        gotenberg_config.max_bundle_bytes
    );

    // ---- fee record to manifest ----
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

    // ---- manifest and variables into one working directory ----
    let work: PathBuf = std::env::temp_dir().join(format!(
        "efees-gotenberg-live-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(
        work.join(&options.vars_source),
        serde_json::to_string_pretty(&build_fee_json(&fee, &project, &company, &contact)).unwrap(),
    )
    .unwrap();
    let manifest_path = work.join("manifest.md");
    std::fs::write(&manifest_path, &manifest.markdown).unwrap();

    // ---- fill ----
    let html_path = work.join("proposal.html");
    let fill_stdout = fill_manifest(&fp_config, &manifest_path, &html_path)
        .unwrap_or_else(|e| panic!("fill.py failed: {e}"));
    eprintln!("fill.py: {}", fill_stdout.trim());

    // ---- gates, on the HTML, before anything is rendered ----
    let gates = run_release_gates(&fp_config, &html_path)
        .unwrap_or_else(|block| panic!("release gate blocked the fixture proposal: {block}"));
    eprintln!(
        "gates passed, {} advisory issue-check hit(s)",
        gates.issue_hits
    );

    // ---- the bundle the renderer will upload ----
    let bundle = flatten_document(&html_path, std::slice::from_ref(&fp_config.root))
        .expect("the filled document must flatten");
    assert_eq!(bundle.files[0].filename, INDEX_FILENAME);
    assert!(
        bundle.files.len() > 1,
        "a filled proposal references at least its stylesheets: {:?}",
        bundle.filenames()
    );
    for file in &bundle.files {
        assert!(
            !file.filename.contains('/'),
            "gotenberg has no subdirectories, but {} carries one",
            file.filename
        );
    }
    eprintln!("bundle: {:?}", bundle.filenames());

    // ---- render ----
    // Deliberately through `select_renderer`, not a hand-built renderer: that
    // is what the API and the CLI call, and it is what supplies the template
    // root as a second allowed asset root. Constructing the renderer directly
    // here would test a configuration nothing in production uses.
    let renderer = select_renderer(&fp_config);
    assert_eq!(
        renderer.backend(),
        "gotenberg",
        "GOTENBERG_URL is set, so the selected backend must be gotenberg"
    );
    let pdf_path = work.join("proposal.pdf");
    renderer
        .render(&html_path, &pdf_path)
        .unwrap_or_else(|e| panic!("gotenberg render failed: {e}"));
    let pdf = std::fs::read(&pdf_path).expect("the renderer must have written a PDF");

    assert!(pdf.starts_with(b"%PDF"), "response is not a PDF");
    let pages = pdf_page_count(&pdf);
    assert!(pages > 0, "rendered PDF reports no pages");

    // The fill engine prints its own page count; the PDF must carry the same
    // number of pages, which is the check that catches a silently dropped
    // stylesheet (wrong @page size collapses many pages into few).
    if let Some(expected) = fill_stdout
        .split_whitespace()
        .zip(fill_stdout.split_whitespace().skip(1))
        .find(|(_, word)| word.starts_with("page"))
        .and_then(|(count, _)| count.parse::<usize>().ok())
    {
        assert_eq!(
            pages, expected,
            "gotenberg produced {pages} pages, fill.py built {expected}"
        );
    }
    eprintln!("rendered {} bytes, {pages} page(s)", pdf.len());

    let _ = std::fs::remove_dir_all(&work);
}
