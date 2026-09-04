//! fp-template fill-manifest export — turns a fee record into the manifest
//! markdown that fp-template's fill engine consumes.
//!
//! # The contract this implements
//!
//! fp-template (gtm repo, `fp-template/`) renders a proposal from a **manifest**,
//! not from a hand-edited copy of the template. Its CLI is:
//!
//! ```text
//! python3 fp-template/fill.py <manifest.md> <output.html>
//! fp-template/render.sh <output.pdf> <output.html>
//! ```
//!
//! A manifest is YAML frontmatter plus a markdown body:
//!
//! - `job`, `template_commit`, `vars_source` (required by `engine/manifest.py`)
//! - `disciplines: {in: [...], out: [...]}` over the fixed vocabulary
//!   `lighting | video | sound`
//! - `sections: {<id>: keep|cut|override}` over the 47 frozen section IDs
//! - one `## <section-id>` body block for every section marked `override`
//!
//! `vars_source` points at the job's `{number}-var Default Values.json` — the
//! 23-field export this crate ALREADY produces via
//! [`crate::export::build_fee_json`]. That file is the pre-existing half of this
//! integration; this module adds the other half.
//!
//! # Scope and honesty notes
//!
//! - The section-ID list and discipline vocabulary are validated against a
//!   COPY of fp-template's own `engine/section_inventory.json`, vendored at
//!   `resources/fp_section_inventory.json` (fp-template commit 932b4cd, the
//!   commit that froze the IDs). Tests assert our defaults never name an ID
//!   the inventory does not know. Re-copy the fixture when the template's
//!   inventory changes.
//! - Sections whose `content_model` is `override-required` MUST carry a body
//!   block. e-fees holds real data for two of them (`fees-design-phase`,
//!   `fees-post-contract`) and can derive a third (`project-details`). The
//!   remaining two (`reference-documents`, `areas`) and `assumptions` are
//!   authored per project and have no e-fees field today — the caller supplies
//!   them via [`FpNarrative`], and when it does not we emit an explicit
//!   `[TO BE COMPLETED ...]` placeholder plus a named residual. fp-template's
//!   own pre-issue gate (`render.sh --issue-check`) flags bracketed
//!   placeholders, so an unfinished manifest cannot silently become an issued
//!   PDF.
//!
//! # D2 client-visibility rule (owner-directed, mission record
//! `ns-backfill-decisions-log`)
//!
//! "We don't want to show it as SUB anywhere outside of the app." `kind=Sub`
//! and `sub_company` are internal taxonomy and must never reach a client-facing
//! export; `sub_discipline` is the client-facing label. This module enforces
//! that mechanically: [`build_fp_manifest`] refuses to emit a manifest that
//! contains the internal marker or a subcontractor company name, and
//! [`assert_no_internal_taxonomy`] is applied to the finished document text as
//! a belt-and-braces check rather than trusting each builder.

use std::collections::BTreeSet;

use crate::models::fee::{Fee, PaymentSchedule, PostContractItem, PricingBreakdown, Stage};
use crate::models::{Company, Contact, Project};

// ============================================================================
// CONTRACT CONSTANTS (mirror fp-template's engine/section_inventory.json)
// ============================================================================

/// Vendored copy of fp-template's section inventory (commit 932b4cd).
/// Used only by tests, to prove our defaults never drift from the template's
/// frozen IDs. Runtime code uses [`FP_SECTION_IDS`] below.
#[cfg(test)]
const FP_SECTION_INVENTORY_JSON: &str = include_str!("../resources/fp_section_inventory.json");

/// fp-template's discipline vocabulary. Fixed at three; the template has no
/// sections for anything else.
pub const FP_DISCIPLINES: [&str; 3] = ["lighting", "video", "sound"];

/// The 47 frozen section IDs, in inventory order.
pub const FP_SECTION_IDS: [&str; 47] = [
    "cover",
    "doc-control-header",
    "doc-control-tracking",
    "doc-control-distribution",
    "company-profile",
    "toc",
    "confidential",
    "proposal-response-page",
    "response",
    "validity",
    "project-details",
    "reference-documents",
    "packages",
    "stages-overview",
    "areas",
    "assumptions",
    "defined-role",
    "services-header",
    "stage-design-phase",
    "preliminaries",
    "stage-1-concept",
    "stage-2-schematic",
    "stage-3-detailed",
    "stage-4-tender",
    "stage-4a-tender-return",
    "stage-post-contract",
    "stage-5-construction",
    "stage-5a-focussing",
    "stage-6-handover",
    "stage-7-dlp",
    "design-phase-notes",
    "post-contract-notes",
    "optional-services",
    "contract-details-header",
    "anticipated-programme",
    "site-attendance",
    "programme-design-phase-table",
    "programme-post-contract-table",
    "fees-header",
    "fees-design-phase",
    "fees-post-contract",
    "payment-terms-header",
    "payment-schedule",
    "additional-payment-terms",
    "basis-of-appointment",
    "limitation-of-liability",
    "next-steps",
];

/// Sections cut by default. Every other ID defaults to `keep`, and the
/// override-required ones are promoted to `override` when we emit a body block.
///
/// This baseline matches the only manifest fp-template has proven end to end
/// (`proposals/26-97109/manifest.md`, "manifest #1"): the three optional-tier
/// notes/tender-return sections and the rare-tier three-column
/// `additional-payment-terms`, which the engine explicitly refuses to place.
const DEFAULT_CUT_SECTIONS: [&str; 4] = [
    "stage-4a-tender-return",
    "design-phase-notes",
    "post-contract-notes",
    "additional-payment-terms",
];

/// Sections this exporter authors from fee data or caller narrative.
pub const OVERRIDE_SECTIONS: [&str; 5] = [
    "project-details",
    "reference-documents",
    "areas",
    "fees-design-phase",
    "fees-post-contract",
];

/// The internal subcontractor marker that must never reach a client document
/// (D2 client-visibility rule).
const SUB_MARKER: &str = "Sub";

/// Sentinel this exporter writes into any `override-required` section it has no
/// data for. It appears in the rendered document, so the release gate can find
/// it there and refuse to hand the PDF to a caller.
///
/// It must be a string the TEMPLATE never produces on its own: fp-template's own
/// boilerplate legitimately contains bracketed drafting qualifiers (`[Client]`,
/// `[60]`, `[RIBA]`) which `render.sh --issue-check` also reports, so the mere
/// presence of brackets cannot be the block condition. This sentinel can.
pub const INCOMPLETE_MARKER: &str = "TO BE COMPLETED";

/// fp-template commit whose section inventory this module was written against.
/// Written into the manifest's `template_commit` field unless the caller
/// overrides it. Metadata only — fp-template does not verify it.
pub const DEFAULT_TEMPLATE_COMMIT: &str = "932b4cd";

// ============================================================================
// TYPES
// ============================================================================

/// A section's action in the manifest's `sections:` map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionAction {
    Keep,
    Cut,
    Override,
}

impl SectionAction {
    fn as_str(self) -> &'static str {
        match self {
            SectionAction::Keep => "keep",
            SectionAction::Cut => "cut",
            SectionAction::Override => "override",
        }
    }
}

/// Per-project prose the fee record cannot supply.
///
/// `reference-documents` and `areas` are `override-required` in fp-template's
/// inventory and are authored per project — e-fees has no field for either
/// today. `assumptions` is `leadin-verbatim-body-override`. The Phase 4
/// scope-assembly UI is the intended source; until it exists a caller may pass
/// them explicitly, and anything left `None` becomes a visible placeholder plus
/// a residual rather than a silent omission.
#[derive(Debug, Clone, Default)]
pub struct FpNarrative {
    /// Bullet lines describing the project. Derived from the project record
    /// when `None`.
    pub project_details: Option<Vec<String>>,
    /// Bullet lines naming the documents this proposal responds to.
    pub reference_documents: Option<Vec<String>>,
    /// Free prose (bullets or paragraphs) for the Areas section.
    pub areas: Option<String>,
    /// Bullet lines for the Assumptions clause list.
    pub assumptions: Option<Vec<String>>,
}

/// Options controlling manifest generation.
#[derive(Debug, Clone)]
pub struct FpManifestOptions {
    /// Value written to `template_commit`.
    pub template_commit: String,
    /// Filename written to `vars_source`, resolved by fp-template relative to
    /// the manifest's own directory. Must name the file produced by
    /// [`crate::export::build_fee_json`].
    pub vars_source: String,
    /// Per-project prose the fee record cannot supply.
    pub narrative: FpNarrative,
}

impl FpManifestOptions {
    /// Defaults for a fee: pinned template commit and the canonical
    /// `{number}-var Default Values.json` filename this crate already exports.
    pub fn for_number(number_dashed: &str) -> Self {
        Self {
            template_commit: DEFAULT_TEMPLATE_COMMIT.to_string(),
            vars_source: format!("{number_dashed}-var Default Values.json"),
            narrative: FpNarrative::default(),
        }
    }
}

/// A generated manifest plus everything the caller must know about it.
#[derive(Debug, Clone)]
pub struct FpManifest {
    /// The manifest markdown, ready to write next to its `vars_source` file.
    pub markdown: String,
    /// fp-template disciplines this proposal covers.
    pub disciplines_in: Vec<String>,
    /// fp-template disciplines cut from the document.
    pub disciplines_out: Vec<String>,
    /// Things this manifest could not express, named rather than approximated.
    pub residuals: Vec<String>,
}

/// Manifest-generation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FpManifestError {
    /// The fee carries no pricing breakdown, so no fee tables can be built.
    NoPricing,
    /// No e-fees discipline mapped onto fp-template's vocabulary, so the
    /// document would have no discipline content at all.
    NoMappableDiscipline { found: Vec<String> },
    /// A discipline is the internal `Sub` marker with no client-facing label.
    /// D2: the marker must never reach the document, and there is nothing to
    /// substitute, so generation fails rather than leaking or guessing.
    InternalTaxonomyLeak { detail: String },
}

impl std::fmt::Display for FpManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FpManifestError::NoPricing => write!(
                f,
                "fee has no pricing breakdown - fee tables cannot be generated"
            ),
            FpManifestError::NoMappableDiscipline { found } => write!(
                f,
                "no fee discipline maps onto fp-template's vocabulary {:?} (fee disciplines: {:?})",
                FP_DISCIPLINES, found
            ),
            FpManifestError::InternalTaxonomyLeak { detail } => {
                write!(f, "D2 client-visibility rule: {detail}")
            }
        }
    }
}

impl std::error::Error for FpManifestError {}

// ============================================================================
// DISCIPLINE MAPPING
// ============================================================================

/// Map an e-fees discipline name onto fp-template's `lighting|video|sound`.
///
/// e-fees' D2 taxonomy has five standard disciplines (Lighting, Video, Audio,
/// SFX, Show Control) plus `Sub`; fp-template's document has sections for three.
/// Anything unmapped is a named residual, never silently dropped from the fee
/// arithmetic — its money still appears in the stage totals, the document just
/// has no section describing it.
pub fn map_discipline(name: &str) -> Option<&'static str> {
    match name.trim().to_ascii_lowercase().as_str() {
        "lighting" | "light" => Some("lighting"),
        "video" | "av video" => Some("video"),
        "sound" | "audio" => Some("sound"),
        _ => None,
    }
}

/// Match a fee's free-text design-stage name onto one of fp-template's four
/// fixed design-stage sections, or `None` when the template has no section for
/// it.
///
/// e-fees stage names are authored per fee (`Construction Documents`,
/// `50% DD`, ...) while the template's Services section is fixed. This is a
/// reporting aid, not a filter: an unmatched stage still appears in the fee
/// table with its own name and its own money.
pub fn template_design_stage(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();
    if n.contains("concept") {
        Some("stage-1-concept")
    } else if n.contains("schematic") {
        Some("stage-2-schematic")
    } else if n.contains("detail") {
        Some("stage-3-detailed")
    } else if n.contains("tender") {
        Some("stage-4-tender")
    } else {
        None
    }
}

/// True when a discipline name is the internal subcontractor marker.
fn is_sub_marker(name: &str) -> bool {
    name.trim().eq_ignore_ascii_case(SUB_MARKER)
}

/// Reject any text that carries internal-only taxonomy into a client document.
///
/// The D2 rule names two forbidden things: the literal `Sub` marker and a
/// `sub_company` value. Company names are supplied by the caller because the
/// fee's discipline rows do not carry them yet (see the residual note in
/// [`build_fp_manifest`]).
///
/// The marker match is CASE-INSENSITIVE and on whole alphanumeric tokens, so it
/// matches [`is_sub_marker`]'s primary check exactly. It therefore also catches
/// a hyphenated `sub-consultant`, which splits into the tokens `sub` and
/// `consultant`. That is deliberate: this guard fails CLOSED, refusing to emit
/// rather than risking a leak, and the caller gets a message naming the line.
/// Rephrase the prose ("specialist consultant") rather than weakening the guard.
pub fn assert_no_internal_taxonomy(
    document: &str,
    forbidden_company_names: &[String],
) -> Result<(), FpManifestError> {
    for line in document.lines() {
        for word in line.split(|c: char| !c.is_ascii_alphanumeric()) {
            if word.eq_ignore_ascii_case(SUB_MARKER) {
                return Err(FpManifestError::InternalTaxonomyLeak {
                    detail: format!(
                        "rendered manifest contains the internal marker {SUB_MARKER:?}: {line:?}"
                    ),
                });
            }
        }
    }
    for company in forbidden_company_names {
        let trimmed = company.trim();
        if trimmed.is_empty() {
            continue;
        }
        if document.contains(trimmed) {
            return Err(FpManifestError::InternalTaxonomyLeak {
                detail: format!(
                    "rendered manifest contains subcontractor company name {trimmed:?}"
                ),
            });
        }
    }
    Ok(())
}

// ============================================================================
// NUMBER / TABLE FORMATTING
// ============================================================================

/// Format a money amount the way the template's own fee tables do:
/// thousands-separated, no decimals (the corpus shows whole-unit fees).
pub fn format_money(amount: f64) -> String {
    let rounded = amount.round().abs() as u64;
    let digits = rounded.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if amount.round() < 0.0 {
        format!("-{out}")
    } else {
        out
    }
}

/// Format a quantity: integral values print bare, fractional keep one decimal.
fn format_qty(qty: f64) -> String {
    if (qty - qty.round()).abs() < f64::EPSILON {
        format!("{}", qty.round() as i64)
    } else {
        format!("{qty:.1}")
    }
}

/// Sum a design stage's fee across every discipline cell, honouring per-cell
/// overrides. Every discipline counts, including ones with no fp-template
/// section — the client's total must stay truthful.
fn stage_total(pricing: &PricingBreakdown, stage: &Stage) -> f64 {
    pricing
        .cells
        .iter()
        .filter(|c| c.stage_id == stage.id)
        .map(|c| c.override_amount.unwrap_or(c.amount))
        .sum()
}

/// `fees-design-phase` body: one row per design (non-post-contract) stage.
pub fn build_design_phase_table(pricing: &PricingBreakdown, currency: &str) -> String {
    let mut stages: Vec<&Stage> = pricing
        .stages
        .iter()
        .filter(|s| !s.is_post_contract)
        .collect();
    stages.sort_by_key(|s| s.order);

    let mut out = format!("| Stage | Fee ({currency}) |\n|---|--:|\n");
    for stage in &stages {
        out.push_str(&format!(
            "| {} | {} |\n",
            stage.name,
            format_money(stage_total(pricing, stage))
        ));
    }
    out
}

/// `fees-post-contract` body: one row per post-contract line item.
pub fn build_post_contract_table(items: &[PostContractItem], currency: &str) -> String {
    let mut out =
        format!("| Service | Qty | Rate ({currency}) | Amount ({currency}) |\n|---|--:|--:|--:|\n");
    for item in items {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            item.description,
            format_qty(item.quantity),
            format_money(item.rate),
            format_money(item.amount)
        ));
    }
    out
}

/// `payment-schedule` body: one row per scheduled payment.
///
/// The inventory marks this section `recomputed-from-fees`, so the engine keeps
/// the template's own table unless a manifest overrides it. e-fees holds the
/// real schedule, so we emit it when the fee has one.
pub fn build_payment_schedule_table(schedule: &PaymentSchedule, currency: &str) -> String {
    let mut out = format!("| Payment | Percentage | Amount ({currency}) |\n|---|--:|--:|\n");
    for entry in &schedule.entries {
        out.push_str(&format!(
            "| {} | {:.1}% | {} |\n",
            entry.description,
            entry.percentage_of_total,
            format_money(entry.amount)
        ));
    }
    out
}

// ============================================================================
// MANIFEST BUILD
// ============================================================================

/// Bullet-list rendering: fp-template's `engine/prose.py` reads `- ` bullets,
/// blank-line-separated paragraphs, and pipe tables.
fn bullets(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| format!("- {}\n", l.trim()))
        .collect::<String>()
}

/// Derive `project-details` bullets from the project record when the caller
/// supplies none. Only states facts the record actually holds.
fn derive_project_details(project: &Project, fee: &Fee) -> Vec<String> {
    let mut out = Vec::new();
    if !project.name.trim().is_empty() {
        out.push(format!("Project: {}", project.name.trim()));
    }
    let location: Vec<&str> = [project.city.trim(), project.country.trim()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    if !location.is_empty() {
        out.push(format!("Location: {}", location.join(", ")));
    }
    if !project.area.trim().is_empty() {
        out.push(format!("Area: {}", project.area.trim()));
    }
    if !fee.activity.trim().is_empty() {
        out.push(format!("Activity: {}", fee.activity.trim()));
    }
    if !fee.package.trim().is_empty() {
        out.push(format!("Package: {}", fee.package.trim()));
    }
    if !project.status.trim().is_empty() {
        out.push(format!(
            "Design stage at appointment: {}",
            project.status.trim()
        ));
    }
    out
}

/// Build the fill manifest for a fee.
///
/// `company` and `contact` are not read directly — they reach the document
/// through the `vars_source` JSON that [`crate::export::build_fee_json`]
/// produces from the same four records. They are taken here so the caller
/// cannot build a manifest from a different record set than the variables
/// export, which would silently produce a mismatched document.
pub fn build_fp_manifest(
    fee: &Fee,
    project: &Project,
    company: &Company,
    contact: &Contact,
    options: &FpManifestOptions,
) -> Result<FpManifest, FpManifestError> {
    let _ = (company, contact);

    let pricing = fee.pricing_typed().ok_or(FpManifestError::NoPricing)?;
    let mut residuals: Vec<String> = Vec::new();

    // ---- disciplines --------------------------------------------------
    let mut mapped: BTreeSet<&'static str> = BTreeSet::new();
    let mut unmapped: Vec<String> = Vec::new();

    for discipline in &pricing.disciplines {
        if is_sub_marker(&discipline.name) {
            // D2: the fee's discipline rows carry no `sub_discipline` label
            // today (the D2 shape landed on `projects.disciplines[]` in
            // migration 006, not on `fee.pricing.disciplines[]`), so there is
            // nothing client-facing to substitute. Fail loudly.
            return Err(FpManifestError::InternalTaxonomyLeak {
                detail: format!(
                    "fee discipline {:?} is the internal subcontractor marker and carries no \
                     client-facing sub_discipline label; refusing to emit it. Give the \
                     discipline its client-facing name (e.g. \"Acoustics\") before exporting.",
                    discipline.name
                ),
            });
        }
        match map_discipline(&discipline.name) {
            Some(fp) => {
                mapped.insert(fp);
            }
            None => unmapped.push(discipline.name.clone()),
        }
    }

    if mapped.is_empty() {
        return Err(FpManifestError::NoMappableDiscipline {
            found: pricing.disciplines.iter().map(|d| d.name.clone()).collect(),
        });
    }

    for name in &unmapped {
        residuals.push(format!(
            "discipline {name:?} has no fp-template equivalent (template vocabulary is \
             lighting/video/sound). Its fee is still included in the stage totals, but the \
             rendered document carries no section describing it."
        ));
    }

    let disciplines_in: Vec<String> = mapped.iter().map(|s| s.to_string()).collect();
    let disciplines_out: Vec<String> = FP_DISCIPLINES
        .iter()
        .filter(|d| !mapped.contains(*d))
        .map(|d| d.to_string())
        .collect();

    // ---- body blocks --------------------------------------------------
    let currency = if pricing.config.currency.trim().is_empty() {
        "AED"
    } else {
        pricing.config.currency.trim()
    };

    let narrative = &options.narrative;
    let mut body: Vec<(String, String)> = Vec::new();

    let project_details = narrative
        .project_details
        .clone()
        .unwrap_or_else(|| derive_project_details(project, fee));
    body.push(("project-details".to_string(), bullets(&project_details)));

    match &narrative.reference_documents {
        Some(docs) if !docs.is_empty() => {
            body.push(("reference-documents".to_string(), bullets(docs)));
        }
        _ => {
            body.push((
                "reference-documents".to_string(),
                format!("- [{INCOMPLETE_MARKER}: list the documents this proposal responds to]\n"),
            ));
            residuals.push(
                "reference-documents is override-required and e-fees holds no such field; \
                 emitted a bracketed placeholder. render.sh --issue-check will flag it before \
                 the PDF can be issued."
                    .to_string(),
            );
        }
    }

    match &narrative.areas {
        Some(text) if !text.trim().is_empty() => {
            body.push(("areas".to_string(), format!("{}\n", text.trim())));
        }
        _ => {
            body.push((
                "areas".to_string(),
                format!(
                    "- [{INCOMPLETE_MARKER}: state the areas included in and excluded from the scope]\n"
                ),
            ));
            residuals.push(
                "areas is override-required and always authored per project; emitted a bracketed \
                 placeholder. render.sh --issue-check will flag it before the PDF can be issued."
                    .to_string(),
            );
        }
    }

    if let Some(assumptions) = &narrative.assumptions {
        if !assumptions.is_empty() {
            body.push(("assumptions".to_string(), bullets(assumptions)));
        }
    }

    body.push((
        "fees-design-phase".to_string(),
        build_design_phase_table(&pricing, currency),
    ));

    // The fee table's stage names come from e-fees (free text); the document's
    // Services section describes fp-template's four fixed design stages. When a
    // fee names a stage the template has no section for, the two halves of the
    // document disagree - say so rather than shipping a fee line for a stage the
    // proposal never describes.
    for stage in pricing.stages.iter().filter(|s| !s.is_post_contract) {
        if template_design_stage(&stage.name).is_none() {
            residuals.push(format!(
                "design stage {:?} does not correspond to any of fp-template's fixed stage \
                 sections (Concept, Schematic, Detailed, Tender). Its fee appears in the fee \
                 table, but the Services section of the document does not describe it.",
                stage.name
            ));
        }
    }

    let post_contract_items = fee.post_contract_items.clone().unwrap_or_default();
    let has_post_contract = !post_contract_items.is_empty();
    if has_post_contract {
        body.push((
            "fees-post-contract".to_string(),
            build_post_contract_table(&post_contract_items, currency),
        ));
    } else {
        residuals.push(
            "fee has no post-contract items; fees-post-contract and \
             programme-post-contract-table are cut from the document."
                .to_string(),
        );
    }

    let payment_schedule = fee
        .payment_schedule
        .as_ref()
        .and_then(|v| {
            serde_json::from_value::<PaymentSchedule>(crate::models::common::dbvalue_to_json(v))
                .ok()
        })
        .filter(|s| !s.entries.is_empty());

    if let Some(schedule) = &payment_schedule {
        body.push((
            "payment-schedule".to_string(),
            build_payment_schedule_table(schedule, currency),
        ));
    } else {
        residuals.push(
            "fee has no payment schedule; payment-schedule keeps the template's own table."
                .to_string(),
        );
    }

    // ---- section actions ----------------------------------------------
    let overridden: BTreeSet<&str> = body.iter().map(|(id, _)| id.as_str()).collect();
    let mut sections: Vec<(&str, SectionAction)> = Vec::with_capacity(FP_SECTION_IDS.len());
    for id in FP_SECTION_IDS {
        let post_contract_cut = !has_post_contract
            && (id == "fees-post-contract" || id == "programme-post-contract-table");
        let action = if DEFAULT_CUT_SECTIONS.contains(&id) || post_contract_cut {
            SectionAction::Cut
        } else if overridden.contains(id) {
            SectionAction::Override
        } else {
            SectionAction::Keep
        };
        sections.push((id, action));
    }

    // ---- render --------------------------------------------------------
    let job = job_id_from_number(&fee.number);
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("job: {job}\n"));
    out.push_str(&format!("template_commit: {}\n", options.template_commit));
    out.push_str(&format!(
        "vars_source: {}\n",
        yaml_scalar(&options.vars_source)
    ));
    out.push_str("disciplines:\n");
    out.push_str(&format!("  in: [{}]\n", disciplines_in.join(", ")));
    out.push_str(&format!("  out: [{}]\n", disciplines_out.join(", ")));
    out.push_str("sections:\n");
    for (id, action) in &sections {
        out.push_str(&format!("  {}: {}\n", id, action.as_str()));
    }
    out.push_str("---\n\n");
    out.push_str(&format!(
        "# {} - Fee Proposal Manifest\n\n",
        if project.name.trim().is_empty() {
            job.as_str()
        } else {
            project.name.trim()
        }
    ));
    out.push_str(&format!(
        "Generated by e-fees from fee {} (rev {}). Variables come from `{}`; \
         edit that export or this manifest's body blocks rather than the template.\n",
        fee.number, fee.rev, options.vars_source
    ));

    for (id, content) in &body {
        out.push_str(&format!("\n## {id}\n\n{content}"));
    }

    // Belt and braces: the builders above are each supposed to be safe, but the
    // guarantee the owner asked for is that NO export path can emit the internal
    // taxonomy - so the finished document is checked, not the intentions.
    assert_no_internal_taxonomy(&out, &sub_company_names(&pricing))?;

    Ok(FpManifest {
        markdown: out,
        disciplines_in,
        disciplines_out,
        residuals,
    })
}

/// Subcontractor company names that must not appear in a client document.
///
/// Empty today: the fee's discipline rows are `{id, name, percentage, order}`
/// only — the D2 `kind`/`sub_discipline`/`sub_company` shape landed on
/// `projects.disciplines[]` in migration 006 and has not reached
/// `fee.pricing.disciplines[]`. When it does, return the `sub_company` display
/// names here and the guard starts covering the second half of the rule; the
/// `Sub` marker check is the half that is live now.
fn sub_company_names(_pricing: &PricingBreakdown) -> Vec<String> {
    Vec::new()
}

/// Quote a YAML scalar when it contains characters that would otherwise change
/// the parse (the canonical `vars_source` filename contains spaces, which is
/// legal unquoted, but a leading/trailing space or a colon is not).
fn yaml_scalar(value: &str) -> String {
    let needs_quote = value.trim() != value
        || value.contains(':')
        || value.contains('#')
        || value.starts_with(['[', '{', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`']);
    if needs_quote {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

/// Reduce a fee number to the job id fp-template expects
/// (`25-96501-FP-1` and `25_96501-FP-1` both become `25-96501`).
fn job_id_from_number(number: &str) -> String {
    let dashed = number.replace('_', "-");
    let parts: Vec<&str> = dashed.split('-').collect();
    if parts.len() >= 2 && parts[0].len() == 2 && parts[1].len() == 5 {
        format!("{}-{}", parts[0], parts[1])
    } else {
        dashed
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::TimeStamps;
    use crate::models::fee::{Discipline, PaymentScheduleEntry, PricingCell, PricingConfig};
    use surrealdb::types::RecordId;
    use surrealdb_types::Datetime;

    // ---- fixtures ----

    fn timestamps() -> TimeStamps {
        TimeStamps {
            created_at: Datetime::default(),
            updated_at: Datetime::default(),
        }
    }

    fn discipline(id: &str, name: &str, order: i64) -> Discipline {
        Discipline {
            id: id.into(),
            name: name.into(),
            percentage: 50.0,
            order,
        }
    }

    fn pricing_with(disciplines: Vec<Discipline>) -> PricingBreakdown {
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

    fn make_fee(pricing: PricingBreakdown) -> Fee {
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
        fee.pricing = Some(crate::models::common::json_to_dbvalue(
            &serde_json::to_value(&pricing).unwrap(),
        ));
        fee
    }

    fn make_project() -> Project {
        Project {
            id: None,
            name: "Kalba Creek Centre".into(),
            name_short: "KCC".into(),
            status: "Concept Design".into(),
            area: "12,000 sq m".into(),
            city: "Kalba".into(),
            country: "U.A.E.".into(),
            folder: "/projects/25-96501".into(),
            number: crate::models::project::ProjectNumber {
                year: 25,
                country: 965,
                seq: 1,
                id: "25_96501".into(),
            },
            time: timestamps(),
        }
    }

    fn make_company() -> Company {
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

    fn make_contact() -> Contact {
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

    fn build(disciplines: Vec<Discipline>) -> Result<FpManifest, FpManifestError> {
        let fee = make_fee(pricing_with(disciplines));
        build_fp_manifest(
            &fee,
            &make_project(),
            &make_company(),
            &make_contact(),
            &FpManifestOptions::for_number("25-96501"),
        )
    }

    // ---- contract: our IDs must be fp-template's IDs ----

    #[test]
    fn section_ids_match_the_vendored_inventory_exactly() {
        let inv: serde_json::Value = serde_json::from_str(FP_SECTION_INVENTORY_JSON).unwrap();
        let template_ids: Vec<String> = inv["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["id"].as_str().unwrap().to_string())
            .collect();
        let ours: Vec<String> = FP_SECTION_IDS.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            ours, template_ids,
            "FP_SECTION_IDS drifted from fp-template's section_inventory.json - \
             re-copy resources/fp_section_inventory.json and update the constant"
        );
    }

    #[test]
    fn discipline_vocabulary_matches_the_vendored_inventory() {
        let inv: serde_json::Value = serde_json::from_str(FP_SECTION_INVENTORY_JSON).unwrap();
        let template: Vec<String> = inv["disciplines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d.as_str().unwrap().to_string())
            .collect();
        assert_eq!(FP_DISCIPLINES.to_vec(), template);
    }

    #[test]
    fn default_cut_and_override_sections_are_real_ids() {
        for id in DEFAULT_CUT_SECTIONS.iter().chain(OVERRIDE_SECTIONS.iter()) {
            assert!(
                FP_SECTION_IDS.contains(id),
                "{id} is not a known section id"
            );
        }
    }

    #[test]
    fn override_required_sections_are_all_authored() {
        // fp-template marks these content_model "override-required": a manifest
        // that keeps them without a body block renders template placeholder text.
        let inv: serde_json::Value = serde_json::from_str(FP_SECTION_INVENTORY_JSON).unwrap();
        let required: Vec<String> = inv["sections"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["content_model"].as_str() == Some("override-required"))
            .map(|s| s["id"].as_str().unwrap().to_string())
            .collect();
        for id in &required {
            assert!(
                OVERRIDE_SECTIONS.contains(&id.as_str()),
                "{id} is override-required in fp-template but this exporter never authors it"
            );
        }
    }

    // ---- manifest shape ----

    #[test]
    fn manifest_carries_every_required_frontmatter_key() {
        let m = build(vec![discipline("d1", "Lighting", 1)]).unwrap();
        let fm = m.markdown.split("---\n").nth(1).unwrap();
        for key in [
            "job:",
            "template_commit:",
            "vars_source:",
            "disciplines:",
            "sections:",
        ] {
            assert!(fm.contains(key), "frontmatter missing {key}\n{fm}");
        }
        assert!(fm.contains("job: 25-96501"));
    }

    #[test]
    fn every_section_id_appears_exactly_once_with_a_legal_action() {
        let m = build(vec![discipline("d1", "Lighting", 1)]).unwrap();
        let fm = m.markdown.split("---\n").nth(1).unwrap();
        for id in FP_SECTION_IDS {
            let needle = format!("\n  {id}: ");
            let count = fm.matches(&needle).count();
            assert_eq!(count, 1, "{id} appears {count} times in sections:");
            let line = fm
                .lines()
                .find(|l| l.trim_start().starts_with(&format!("{id}: ")))
                .unwrap();
            let action = line.split(": ").nth(1).unwrap().trim();
            assert!(
                ["keep", "cut", "override"].contains(&action),
                "{id} has illegal action {action}"
            );
        }
    }

    #[test]
    fn every_override_section_has_a_body_block() {
        let m = build(vec![discipline("d1", "Lighting", 1)]).unwrap();
        let fm = m.markdown.split("---\n").nth(1).unwrap();
        for line in fm.lines() {
            let line = line.trim();
            if let Some((id, action)) = line.split_once(": ") {
                if action == "override" {
                    assert!(
                        m.markdown.contains(&format!("\n## {id}\n")),
                        "section {id} is override but has no body block"
                    );
                }
            }
        }
    }

    #[test]
    fn every_body_block_is_declared_override() {
        let m = build(vec![discipline("d1", "Lighting", 1)]).unwrap();
        let (fm, body) = m.markdown.split_at(m.markdown.find("---\n\n").unwrap());
        for line in body.lines() {
            if let Some(id) = line.strip_prefix("## ") {
                assert!(
                    fm.contains(&format!("  {id}: override")),
                    "body block {id} is not declared override in frontmatter"
                );
            }
        }
    }

    // ---- disciplines ----

    #[test]
    fn maps_audio_to_sound_and_cuts_the_rest() {
        let m = build(vec![
            discipline("d1", "Lighting", 1),
            discipline("d2", "Audio", 2),
        ])
        .unwrap();
        assert_eq!(m.disciplines_in, vec!["lighting", "sound"]);
        assert_eq!(m.disciplines_out, vec!["video"]);
    }

    #[test]
    fn unmappable_discipline_is_a_named_residual_not_a_silent_drop() {
        let m = build(vec![
            discipline("d1", "Lighting", 1),
            discipline("d2", "Control", 2),
        ])
        .unwrap();
        assert_eq!(m.disciplines_in, vec!["lighting"]);
        assert!(
            m.residuals.iter().any(|r| r.contains("Control")),
            "residuals did not name the unmapped discipline: {:?}",
            m.residuals
        );
    }

    #[test]
    fn unmappable_discipline_fee_still_counts_in_the_stage_total() {
        // Lighting + Control, 1000 each at Concept: the table must show 2,000,
        // not 1,000 - dropping a discipline from the DOCUMENT must never drop
        // its money from the CLIENT'S total.
        let m = build(vec![
            discipline("d1", "Lighting", 1),
            discipline("d2", "Control", 2),
        ])
        .unwrap();
        assert!(
            m.markdown.contains("| Concept Design | 2,000 |"),
            "stage total wrong:\n{}",
            m.markdown
        );
    }

    // ---- stage-name correspondence ----

    #[test]
    fn template_design_stages_match_on_the_usual_names() {
        assert_eq!(
            template_design_stage("Concept Design"),
            Some("stage-1-concept")
        );
        assert_eq!(
            template_design_stage("Schematic Design"),
            Some("stage-2-schematic")
        );
        assert_eq!(
            template_design_stage("50% Detailed Design"),
            Some("stage-3-detailed")
        );
        assert_eq!(
            template_design_stage("Tender Documentation"),
            Some("stage-4-tender")
        );
        assert_eq!(template_design_stage("Construction Documents"), None);
    }

    #[test]
    fn stage_with_no_template_section_is_a_named_residual() {
        let mut p = pricing_with(vec![discipline("d1", "Lighting", 1)]);
        p.stages[1].name = "Construction Documents".into();
        let mut fee = make_fee(p);
        fee.post_contract_items = Some(vec![]);
        let m = build_fp_manifest(
            &fee,
            &make_project(),
            &make_company(),
            &make_contact(),
            &FpManifestOptions::for_number("25-96501"),
        )
        .unwrap();
        assert!(
            m.residuals
                .iter()
                .any(|r| r.contains("Construction Documents")),
            "residuals: {:?}",
            m.residuals
        );
        // ...and its money is still in the table.
        assert!(m.markdown.contains("| Construction Documents | 2,000 |"));
    }

    #[test]
    fn no_mappable_discipline_is_an_error() {
        let err = build(vec![discipline("d1", "Control", 1)]).unwrap_err();
        assert!(matches!(err, FpManifestError::NoMappableDiscipline { .. }));
    }

    // ---- D2 client-visibility rule ----

    #[test]
    fn sub_discipline_marker_is_refused_never_emitted() {
        let err = build(vec![
            discipline("d1", "Lighting", 1),
            discipline("d2", "Sub", 2),
        ])
        .unwrap_err();
        match err {
            FpManifestError::InternalTaxonomyLeak { detail } => {
                assert!(detail.contains("Sub"), "{detail}");
            }
            other => panic!("expected InternalTaxonomyLeak, got {other:?}"),
        }
    }

    #[test]
    fn sub_marker_is_case_insensitive() {
        for name in ["sub", "SUB", " Sub "] {
            let err = build(vec![
                discipline("d1", "Lighting", 1),
                discipline("d2", name, 2),
            ])
            .unwrap_err();
            assert!(
                matches!(err, FpManifestError::InternalTaxonomyLeak { .. }),
                "{name:?} was not caught"
            );
        }
    }

    #[test]
    fn generated_manifest_never_contains_the_sub_marker() {
        let m = build(vec![
            discipline("d1", "Lighting", 1),
            discipline("d2", "Video", 2),
        ])
        .unwrap();
        assert!(assert_no_internal_taxonomy(&m.markdown, &[]).is_ok());
    }

    #[test]
    fn taxonomy_guard_catches_a_sub_company_name() {
        let doc = "## areas\n\n- Acoustic modelling by WADG Consulting\n";
        let err = assert_no_internal_taxonomy(doc, &["WADG Consulting".to_string()]).unwrap_err();
        match err {
            FpManifestError::InternalTaxonomyLeak { detail } => {
                assert!(detail.contains("WADG Consulting"), "{detail}")
            }
            other => panic!("expected InternalTaxonomyLeak, got {other:?}"),
        }
    }

    #[test]
    fn taxonomy_guard_allows_words_merely_starting_with_sub() {
        // "Submittal" and "submission" are ordinary words; only the exact token
        // "Sub" is the internal marker.
        let doc = "| Concept Design Submittal | 30.0% | 1,000 |\nsubmission of drawings\n";
        assert!(assert_no_internal_taxonomy(doc, &[]).is_ok());
    }

    // ---- tables ----

    #[test]
    fn design_phase_table_excludes_post_contract_stages() {
        let p = pricing_with(vec![discipline("d1", "Lighting", 1)]);
        let table = build_design_phase_table(&p, "AED");
        assert!(table.contains("| Concept Design | 1,000 |"));
        assert!(table.contains("| Detailed Design | 2,000 |"));
        assert!(
            !table.contains("Site Meetings"),
            "post-contract stage leaked into the design table"
        );
    }

    #[test]
    fn design_phase_table_honours_cell_overrides() {
        let mut p = pricing_with(vec![discipline("d1", "Lighting", 1)]);
        p.cells[0].override_amount = Some(9999.0);
        let table = build_design_phase_table(&p, "AED");
        assert!(table.contains("| Concept Design | 9,999 |"), "{table}");
    }

    #[test]
    fn post_contract_table_renders_qty_rate_amount() {
        let items = vec![PostContractItem {
            id: "pc1".into(),
            stage_id: "p1".into(),
            description: "Site inspection visits".into(),
            quantity: 8.0,
            unit: "visit".into(),
            rate: 4500.0,
            amount: 36000.0,
        }];
        let table = build_post_contract_table(&items, "AED");
        assert!(
            table.contains("| Site inspection visits | 8 | 4,500 | 36,000 |"),
            "{table}"
        );
    }

    #[test]
    fn payment_schedule_table_renders_percentages() {
        let schedule = PaymentSchedule {
            entries: vec![PaymentScheduleEntry {
                id: "p1".into(),
                payment_type: "mobilisation".into(),
                description: "Mobilisation (30%)".into(),
                stage_id: None,
                stage_percentage: None,
                amount: 31605.0,
                percentage_of_total: 30.0,
                due_date: None,
                status: "pending".into(),
                invoice_number: None,
                invoice_date: None,
                paid_date: None,
            }],
            total_invoiced: 0.0,
            total_paid: 0.0,
            total_outstanding: 0.0,
        };
        let table = build_payment_schedule_table(&schedule, "AED");
        assert!(
            table.contains("| Mobilisation (30%) | 30.0% | 31,605 |"),
            "{table}"
        );
    }

    #[test]
    fn fee_without_post_contract_items_cuts_those_sections() {
        let mut fee = make_fee(pricing_with(vec![discipline("d1", "Lighting", 1)]));
        fee.post_contract_items = Some(vec![]);
        let m = build_fp_manifest(
            &fee,
            &make_project(),
            &make_company(),
            &make_contact(),
            &FpManifestOptions::for_number("25-96501"),
        )
        .unwrap();
        assert!(m.markdown.contains("fees-post-contract: cut"));
        assert!(m.markdown.contains("programme-post-contract-table: cut"));
        assert!(!m.markdown.contains("\n## fees-post-contract\n"));
    }

    #[test]
    fn fee_without_pricing_is_an_error() {
        let mut fee = make_fee(pricing_with(vec![discipline("d1", "Lighting", 1)]));
        fee.pricing = None;
        let err = build_fp_manifest(
            &fee,
            &make_project(),
            &make_company(),
            &make_contact(),
            &FpManifestOptions::for_number("25-96501"),
        )
        .unwrap_err();
        assert_eq!(err, FpManifestError::NoPricing);
    }

    // ---- narrative + residuals ----

    #[test]
    fn missing_narrative_produces_placeholders_and_residuals() {
        let m = build(vec![discipline("d1", "Lighting", 1)]).unwrap();
        assert!(m.markdown.contains("[TO BE COMPLETED"));
        assert!(m
            .residuals
            .iter()
            .any(|r| r.contains("reference-documents")));
        assert!(m.residuals.iter().any(|r| r.contains("areas")));
    }

    #[test]
    fn supplied_narrative_replaces_the_placeholders() {
        let fee = make_fee(pricing_with(vec![discipline("d1", "Lighting", 1)]));
        let mut opts = FpManifestOptions::for_number("25-96501");
        opts.narrative.reference_documents =
            Some(vec!["Request for Quotation, received 3 August 2026".into()]);
        opts.narrative.areas = Some("- All building facades".into());
        let m = build_fp_manifest(
            &fee,
            &make_project(),
            &make_company(),
            &make_contact(),
            &opts,
        )
        .unwrap();
        assert!(!m.markdown.contains("[TO BE COMPLETED"));
        assert!(m
            .markdown
            .contains("Request for Quotation, received 3 August 2026"));
    }

    #[test]
    fn derived_project_details_state_only_recorded_facts() {
        let m = build(vec![discipline("d1", "Lighting", 1)]).unwrap();
        assert!(m.markdown.contains("Project: Kalba Creek Centre"));
        assert!(m.markdown.contains("Location: Kalba, U.A.E."));
        assert!(m
            .markdown
            .contains("Design stage at appointment: Concept Design"));
    }

    // ---- formatting ----

    #[test]
    fn money_formatting_matches_the_template_convention() {
        assert_eq!(format_money(29500.0), "29,500");
        assert_eq!(format_money(1052.6315789473686), "1,053");
        assert_eq!(format_money(0.0), "0");
        assert_eq!(format_money(1234567.0), "1,234,567");
        assert_eq!(format_money(999.4), "999");
    }

    #[test]
    fn vars_source_default_matches_the_existing_export_filename() {
        let opts = FpManifestOptions::for_number("26-97109");
        assert_eq!(opts.vars_source, "26-97109-var Default Values.json");
    }

    #[test]
    fn job_id_reduces_a_fee_number_to_the_project_number() {
        assert_eq!(job_id_from_number("25-96501-FP-1"), "25-96501");
        assert_eq!(job_id_from_number("25_96501-FP-1"), "25-96501");
        assert_eq!(job_id_from_number("odd-name"), "odd-name");
    }
}
