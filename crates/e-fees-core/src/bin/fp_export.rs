//! Export a fee as an fp-template fill manifest, and optionally render it.
//!
//! This is the read-only, orchestrator-runnable path for the fp-template
//! integration: it reads one fee (plus its project/company/contact) from the
//! database, writes the two files fp-template's fill engine needs, and can then
//! drive `fill.py` + `render.sh` to produce the proposal PDF.
//!
//! ```text
//! cargo run -p e-fees-core --bin fp_export -- \
//!   --target dev --fee 25_96501_1 --out-dir /tmp/25-96501 [--render] [--gates]
//! ```
//!
//! Writes into `--out-dir`:
//!   - `{number}-var Default Values.json` — the 23-field variable export
//!     (`export::build_fee_json`, unchanged, the pre-existing half of this
//!     integration)
//!   - `manifest.md` — the fill manifest (`export::fp_manifest`)
//!   - with `--render`: `proposal.html` and `proposal.pdf`
//!
//! NEVER WRITES TO THE DATABASE. Rendering needs `FP_TEMPLATE_ROOT` (or
//! `--fp-template-root`) pointing at an fp-template checkout, plus python3 and
//! a headless Chrome on this host.
//!
//! Credentials: `EFEES_SURREALDB_USER` / `EFEES_SURREALDB_PASS` env vars, the
//! same pair the backfill binaries use. Never read from a file, never printed.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use e_fees_core::export::fp_manifest::{build_fp_manifest, FpManifestOptions};
use e_fees_core::export::fp_render::{build_proposal_pdf, run_release_gates, FpRenderConfig};
use e_fees_core::export::{build_fee_json, clean_number_for_path};
use e_fees_core::models::{record_id_string, Company, Contact, Fee, Project};

struct TargetConfig {
    name: &'static str,
    host: &'static str,
    ns: &'static str,
    db: &'static str,
}

const DEV: TargetConfig = TargetConfig {
    name: "dev",
    host: "10.0.23.12:8000",
    ns: "emittiv_dev",
    db: "projects",
};
const PROD: TargetConfig = TargetConfig {
    name: "prod",
    host: "10.0.23.11:8000",
    ns: "emittiv",
    db: "projects",
};

struct Args {
    target: &'static TargetConfig,
    fee_key: String,
    out_dir: PathBuf,
    render: bool,
    gates: bool,
    fp_template_root: Option<PathBuf>,
    reference_docs: Vec<String>,
    areas: Option<String>,
}

fn usage() -> &'static str {
    "Usage: fp_export --fee <record-key> --out-dir <dir> [--target dev|prod] \
     [--render] [--gates] [--fp-template-root <dir>] \
     [--reference-doc <text> ...] [--areas <text>]"
}

fn parse_args() -> Result<Args, String> {
    let mut target = &DEV;
    let mut fee_key = None;
    let mut out_dir = None;
    let mut render = false;
    let mut gates = false;
    let mut fp_template_root = None;
    let mut reference_docs: Vec<String> = Vec::new();
    let mut areas = None;

    let argv: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--target" => {
                i += 1;
                target = match argv.get(i).map(|s| s.as_str()) {
                    Some("dev") => &DEV,
                    Some("prod") => &PROD,
                    other => return Err(format!("unknown --target {other:?}")),
                };
            }
            "--fee" => {
                i += 1;
                fee_key = argv.get(i).cloned();
            }
            "--out-dir" => {
                i += 1;
                out_dir = argv.get(i).map(PathBuf::from);
            }
            "--fp-template-root" => {
                i += 1;
                fp_template_root = argv.get(i).map(PathBuf::from);
            }
            "--reference-doc" => {
                i += 1;
                if let Some(v) = argv.get(i) {
                    reference_docs.push(v.clone());
                }
            }
            "--areas" => {
                i += 1;
                areas = argv.get(i).cloned();
            }
            "--render" => render = true,
            "--gates" => gates = true,
            "-h" | "--help" => return Err(usage().to_string()),
            other => return Err(format!("unknown argument {other:?}\n{}", usage())),
        }
        i += 1;
    }

    Ok(Args {
        target,
        fee_key: fee_key.ok_or_else(|| format!("--fee is required\n{}", usage()))?,
        out_dir: out_dir.ok_or_else(|| format!("--out-dir is required\n{}", usage()))?,
        render,
        gates,
        fp_template_root,
        reference_docs,
        areas,
    })
}

/// Split "table:key" into its key half, matching `fee_export.rs`'s own helper.
fn key_of(id: &surrealdb::types::RecordId) -> String {
    let s = record_id_string(id);
    s.split_once(':').map(|x| x.1).unwrap_or("").to_string()
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    let user = match env::var("EFEES_SURREALDB_USER") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("EFEES_SURREALDB_USER is not set");
            return ExitCode::FAILURE;
        }
    };
    let pass = match env::var("EFEES_SURREALDB_PASS") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("EFEES_SURREALDB_PASS is not set");
            return ExitCode::FAILURE;
        }
    };

    let db = match Surreal::new::<Ws>(args.target.host).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("connect to {} failed: {e}", args.target.host);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = db
        .signin(Root {
            username: user,
            password: pass,
        })
        .await
    {
        eprintln!("signin failed: {e}");
        return ExitCode::FAILURE;
    }
    if let Err(e) = db.use_ns(args.target.ns).use_db(args.target.db).await {
        eprintln!("use ns/db failed: {e}");
        return ExitCode::FAILURE;
    }
    eprintln!(
        "connected: {} ({} ns={} db={})",
        args.target.name, args.target.host, args.target.ns, args.target.db
    );

    // ---- read the four records (read-only) ----
    let fee: Option<Fee> = match db.select(("fee", args.fee_key.as_str())).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("select fee failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(fee) = fee else {
        eprintln!("fee:{} not found", args.fee_key);
        return ExitCode::FAILURE;
    };

    let project: Option<Project> = db
        .select(("projects", key_of(&fee.project_id).as_str()))
        .await
        .unwrap_or(None);
    let company: Option<Company> = db
        .select(("company", key_of(&fee.company_id).as_str()))
        .await
        .unwrap_or(None);
    let contact: Option<Contact> = db
        .select(("contacts", key_of(&fee.contact_id).as_str()))
        .await
        .unwrap_or(None);

    let (Some(project), Some(company), Some(contact)) = (project, company, contact) else {
        eprintln!("fee:{} has a missing linked record", args.fee_key);
        return ExitCode::FAILURE;
    };

    // ---- build the two artefacts ----
    let number = clean_number_for_path(&fee.number);
    let project_number = clean_number_for_path(&project.number.id);
    let mut options = FpManifestOptions::for_number(&project_number);
    if !args.reference_docs.is_empty() {
        options.narrative.reference_documents = Some(args.reference_docs.clone());
    }
    if args.areas.is_some() {
        options.narrative.areas = args.areas.clone();
    }

    let manifest = match build_fp_manifest(&fee, &project, &company, &contact, &options) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("manifest generation failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = std::fs::create_dir_all(&args.out_dir) {
        eprintln!("could not create {}: {e}", args.out_dir.display());
        return ExitCode::FAILURE;
    }

    let vars_path = args.out_dir.join(&options.vars_source);
    let vars_json = build_fee_json(&fee, &project, &company, &contact);
    if let Err(e) = std::fs::write(
        &vars_path,
        serde_json::to_string_pretty(&vars_json).unwrap_or_default(),
    ) {
        eprintln!("could not write {}: {e}", vars_path.display());
        return ExitCode::FAILURE;
    }

    let manifest_path = args.out_dir.join("manifest.md");
    if let Err(e) = std::fs::write(&manifest_path, &manifest.markdown) {
        eprintln!("could not write {}: {e}", manifest_path.display());
        return ExitCode::FAILURE;
    }

    println!("fee:            {} ({})", args.fee_key, number);
    println!("project:        {}", project.name);
    println!("variables:      {}", vars_path.display());
    println!("manifest:       {}", manifest_path.display());
    println!("disciplines in: {}", manifest.disciplines_in.join(", "));
    println!("disciplines out:{}", manifest.disciplines_out.join(", "));
    if manifest.residuals.is_empty() {
        println!("residuals:      none");
    } else {
        println!("residuals:      {}", manifest.residuals.len());
        for r in &manifest.residuals {
            println!("  - {r}");
        }
    }

    if !args.render && !args.gates {
        return ExitCode::SUCCESS;
    }

    // ---- render ----
    let config = match args
        .fp_template_root
        .map(FpRenderConfig::new)
        .or_else(FpRenderConfig::from_env)
    {
        Some(c) => c,
        None => {
            eprintln!(
                "render requested but no fp-template checkout given \
                 (set FP_TEMPLATE_ROOT or pass --fp-template-root)"
            );
            return ExitCode::FAILURE;
        }
    };

    let html_path = args.out_dir.join("proposal.html");
    let pdf_path = args.out_dir.join("proposal.pdf");

    match build_proposal_pdf(&config, &manifest_path, &html_path, &pdf_path) {
        Ok(out) => {
            println!("html:           {}", out.html_path.display());
            println!("pdf:            {}", out.pdf_path.display());
            for line in out.fill_stdout.lines() {
                println!("fill.py:        {line}");
            }
        }
        Err(e) => {
            eprintln!("render failed: {e}");
            return ExitCode::FAILURE;
        }
    }

    if args.gates {
        // Same policy the API route applies, so the CLI cannot report a
        // proposal as releasable when the route would refuse it.
        match run_release_gates(&config, &html_path) {
            Ok(report) => {
                println!("release gate:   PASS");
                println!(
                    "issue-check:    {} advisory hit(s) from the template's own boilerplate",
                    report.issue_hits
                );
            }
            Err(block) => {
                println!("release gate:   BLOCK - this document must not be issued");
                for line in block.to_string().lines() {
                    println!("  {line}");
                }
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
