//! fp-template proposal endpoints — fee record to fill manifest to rendered PDF.
//!
//! Two routes, deliberately split so the cheap half works everywhere:
//!
//! | Route | Needs | Returns |
//! |---|---|---|
//! | `GET /fees/{id}/fp-manifest` | database only | the manifest markdown |
//! | `POST /fees/{id}/fp-proposal` | database + a local fp-template checkout | `application/pdf` |
//!
//! The manifest route is pure computation over records this API already
//! serves, so it is always available. The render route shells out to
//! fp-template's own `fill.py` and `render.sh`, which need python3, a headless
//! Chrome, and a checkout of the template on the same host — none of which the
//! `e-fees-api` container image carries today. When `FP_TEMPLATE_ROOT` is
//! unset the route answers `503 Service Unavailable` with that reason rather
//! than failing obscurely. Packaging the renderer is a deployment decision;
//! see `e_fees_core::export::fp_render`'s module docs.
//!
//! Neither route writes to the database, and neither writes into the Nextcloud
//! project folder — that is `fee_export.rs`'s job and stays there.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use e_fees_core::export::fp_manifest::{build_fp_manifest, FpManifestOptions, FpNarrative};
use e_fees_core::export::fp_render::{
    build_proposal_pdf_gated, FpRenderConfig, FpRenderError, GateBlock, GateReport,
    GatedRenderError, FP_TEMPLATE_ROOT_ENV,
};
use e_fees_core::export::{build_fee_json, clean_number_for_path};

use crate::error::ApiError;
use crate::validation::validate_id;
use crate::AppState;

use super::fee_export::fetch_fee_with_links;

/// Per-project prose the fee record cannot supply.
///
/// Every field is optional. Anything omitted becomes a visible
/// `[TO BE COMPLETED ...]` placeholder in the manifest plus a named residual in
/// the response, so an unfinished proposal is obvious rather than silent.
#[derive(Debug, Default, Deserialize, utoipa::ToSchema)]
pub struct FpNarrativeRequest {
    #[serde(default)]
    pub project_details: Option<Vec<String>>,
    #[serde(default)]
    pub reference_documents: Option<Vec<String>>,
    #[serde(default)]
    pub areas: Option<String>,
    #[serde(default)]
    pub assumptions: Option<Vec<String>>,
}

impl From<FpNarrativeRequest> for FpNarrative {
    fn from(r: FpNarrativeRequest) -> Self {
        FpNarrative {
            project_details: r.project_details,
            reference_documents: r.reference_documents,
            areas: r.areas,
            assumptions: r.assumptions,
        }
    }
}

/// The manifest plus everything the caller needs to act on it.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FpManifestResponse {
    /// fp-template job id (the project number).
    pub job: String,
    /// Filename the manifest's `vars_source` names; the caller must write the
    /// variables export under exactly this name next to the manifest.
    pub vars_source: String,
    /// The 23-field variable export, identical to `/fees/{id}/json-export`'s
    /// payload. Returned here so a caller can write both files in one step.
    pub variables: serde_json::Value,
    /// The manifest markdown.
    pub manifest: String,
    pub disciplines_in: Vec<String>,
    pub disciplines_out: Vec<String>,
    /// Things the manifest could not express. Never empty silently.
    pub residuals: Vec<String>,
}

/// Build the manifest for a fee without rendering anything.
///
/// `GET /fees/{id}/fp-manifest`
#[utoipa::path(
    get,
    path = "/fees/{id}/fp-manifest",
    tag = "fee-export",
    params(("id" = String, Path, description = "Fee record key")),
    responses(
        (status = 200, description = "Manifest generated", body = FpManifestResponse),
        (status = 404, description = "Fee or a linked record not found"),
        (status = 422, description = "Fee cannot produce a manifest (no pricing, or an unmappable/internal discipline)"),
    )
)]
pub async fn get_fp_manifest(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<FpManifestResponse>, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);
    validate_id(key)?;

    let (fee, project, company, contact) = fetch_fee_with_links(&state, key).await?;
    let options = FpManifestOptions::for_number(&clean_number_for_path(&project.number.id));

    let manifest =
        build_fp_manifest(&fee, &project, &company, &contact, &options).map_err(|e| {
            // A refusal here is a data problem the caller must fix (missing
            // pricing, an unmappable discipline set, or the D2 internal marker),
            // not a server fault - 422, with the reason.
            warn!("fp manifest generation refused for fee {key}: {e}");
            ApiError::unprocessable(e.to_string())
        })?;

    Ok(Json(FpManifestResponse {
        job: clean_number_for_path(&project.number.id),
        vars_source: options.vars_source.clone(),
        variables: build_fee_json(&fee, &project, &company, &contact),
        manifest: manifest.markdown,
        disciplines_in: manifest.disciplines_in,
        disciplines_out: manifest.disciplines_out,
        residuals: manifest.residuals,
    }))
}

/// Render a fee to a proposal PDF.
///
/// `POST /fees/{id}/fp-proposal`
///
/// The manifest and its variables file are written to a per-request temporary
/// directory (fp-template resolves `vars_source` relative to the manifest), the
/// fill engine and renderer run under a timeout, and the PDF is streamed back.
/// Nothing is left on disk and nothing is written to the project folder.
#[utoipa::path(
    post,
    path = "/fees/{id}/fp-proposal",
    tag = "fee-export",
    params(("id" = String, Path, description = "Fee record key")),
    request_body = Option<FpNarrativeRequest>,
    responses(
        (status = 200, description = "Rendered proposal", content_type = "application/pdf"),
        (status = 404, description = "Fee or a linked record not found"),
        (status = 422, description = "Fee cannot produce a manifest"),
        (status = 503, description = "Render backend not configured on this host"),
    )
)]
pub async fn render_fp_proposal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    body: Option<Json<FpNarrativeRequest>>,
) -> Result<Response, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);
    validate_id(key)?;

    let config = FpRenderConfig::from_env().ok_or_else(|| {
        ApiError::service_unavailable(format!(
            "Proposal rendering not configured ({FP_TEMPLATE_ROOT_ENV} not set). \
             The manifest is still available at /fees/{{id}}/fp-manifest."
        ))
    })?;
    config
        .validate()
        .map_err(|e| ApiError::service_unavailable(e.to_string()))?;

    let (fee, project, company, contact) = fetch_fee_with_links(&state, key).await?;

    let number = clean_number_for_path(&project.number.id);
    let mut options = FpManifestOptions::for_number(&number);
    if let Some(Json(narrative)) = body {
        options.narrative = narrative.into();
    }

    let manifest =
        build_fp_manifest(&fee, &project, &company, &contact, &options).map_err(|e| {
            warn!("fp manifest generation refused for fee {key}: {e}");
            ApiError::unprocessable(e.to_string())
        })?;
    let variables = build_fee_json(&fee, &project, &company, &contact);

    // Per-request scratch directory. The manifest and its vars file MUST share
    // a directory - fp-template resolves vars_source relative to the manifest.
    let work_dir = std::env::temp_dir().join(format!(
        "efees-fp-{}-{}",
        number,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));

    let vars_source = options.vars_source.clone();
    let markdown = manifest.markdown.clone();
    let residual_count = manifest.residuals.len();
    let work_dir_for_task = work_dir.clone();

    // fill.py + headless Chrome are blocking and slow; keep them off the async
    // runtime's worker threads. The gate runs inside the same task, so a
    // document that must not be issued never becomes a response body.
    let render =
        tokio::task::spawn_blocking(move || -> Result<(Vec<u8>, GateReport), GatedRenderError> {
            let write = |path: std::path::PathBuf, body: String| -> Result<(), FpRenderError> {
                std::fs::write(path, body)?;
                Ok(())
            };
            std::fs::create_dir_all(&work_dir_for_task)
                .map_err(|e| GatedRenderError::Render(FpRenderError::Io(e)))?;
            write(
                work_dir_for_task.join(&vars_source),
                serde_json::to_string_pretty(&variables).unwrap_or_default(),
            )
            .map_err(GatedRenderError::Render)?;
            let manifest_path = work_dir_for_task.join("manifest.md");
            write(manifest_path.clone(), markdown).map_err(GatedRenderError::Render)?;

            let html_path = work_dir_for_task.join("proposal.html");
            let pdf_path = work_dir_for_task.join("proposal.pdf");
            let built = build_proposal_pdf_gated(&config, &manifest_path, &html_path, &pdf_path)?;
            let bytes = std::fs::read(&pdf_path)
                .map_err(|e| GatedRenderError::Render(FpRenderError::Io(e)))?;
            Ok((bytes, built.gates))
        })
        .await;

    // Always clean up, whatever happened.
    let _ = std::fs::remove_dir_all(&work_dir);

    let (pdf, gates) = match render {
        Ok(Ok(ok)) => ok,
        Ok(Err(e)) => {
            warn!("fp proposal refused for fee {key}: {e}");
            return Err(render_error_to_api(e));
        }
        Err(join_err) => {
            warn!("fp render task panicked for fee {key}: {join_err}");
            return Err(ApiError::service_unavailable(
                "Proposal rendering failed unexpectedly".to_string(),
            ));
        }
    };

    info!(
        "rendered fp proposal for fee {key} ({} bytes, {residual_count} residual(s), \
         {} advisory issue-check hit(s))",
        pdf.len(),
        gates.issue_hits
    );

    let filename = format!("{}-FP.pdf", number);
    let mut response = (StatusCode::OK, Body::from(pdf)).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    // Residual count travels in a header so a caller streaming the PDF still
    // learns the manifest was incomplete without a second request.
    if let Ok(value) = HeaderValue::from_str(&residual_count.to_string()) {
        response.headers_mut().insert("x-fp-residuals", value);
    }
    // Advisory: issue-check hits belonging to the template's own boilerplate,
    // which a human may still want to judge. Blocking hits never reach here.
    if let Ok(value) = HeaderValue::from_str(&gates.issue_hits.to_string()) {
        response.headers_mut().insert("x-fp-issue-hits", value);
    }
    Ok(response)
}

/// Map a gated-render failure onto the right HTTP status.
///
/// A gate BLOCK is 422 carrying the gate's own report - never a PDF. A missing
/// or invalid template checkout is 503, because it is this host's configuration
/// and not the caller's request. Everything else (a fill failure, which is
/// fp-template's manifest-check report) is 422 with the report verbatim.
pub(crate) fn render_error_to_api(e: GatedRenderError) -> ApiError {
    match e {
        GatedRenderError::Gate(block) => match block {
            GateBlock::Incomplete { .. } | GateBlock::Mechanical { .. } => {
                ApiError::unprocessable(block.to_string())
            }
        },
        GatedRenderError::Render(
            err @ (FpRenderError::NotConfigured | FpRenderError::TemplateRootInvalid { .. }),
        ) => ApiError::service_unavailable(err.to_string()),
        GatedRenderError::Render(other) => ApiError::unprocessable(other.to_string()),
    }
}

// ============================================================================
// TESTS
// ============================================================================
//
// These cover the decision layer of the two routes: which HTTP status a given
// outcome produces, and the invariant that a blocked document never becomes a
// PDF response. Exercising the handlers over real HTTP would need both a live
// database and the render toolchain (python3 + playwright + headless Chrome),
// neither of which CI has - that path is covered by the `fp_export` CLI against
// the dev database instead, and is recorded in
// docs/development/FP-TEMPLATE-INTEGRATION.md.

#[cfg(test)]
mod tests {
    use super::*;
    use e_fees_core::export::fp_manifest::INCOMPLETE_MARKER;
    use e_fees_core::export::fp_render::{classify_gates, GateBlock};

    fn issue_failure(stdout: &str) -> FpRenderError {
        FpRenderError::Failed {
            command: "--issue-check".into(),
            code: Some(1),
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    #[test]
    fn unfilled_placeholder_is_422_carrying_the_gate_report() {
        let report = format!(
            "issue-check: proposal.html\n  2 hits\n    \
             [PLACEHOLDER] page 5 line 290: [{INCOMPLETE_MARKER}: list the documents this \
             proposal responds to]\n"
        );
        let block = classify_gates(
            Ok(String::new()),
            Ok(String::new()),
            Err(issue_failure(&report)),
            INCOMPLETE_MARKER,
        )
        .unwrap_err();
        assert!(matches!(block, GateBlock::Incomplete { .. }));

        let err = render_error_to_api(GatedRenderError::Gate(block));
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            err.message.contains("must not be issued"),
            "{}",
            err.message
        );
        assert!(
            err.message.contains("list the documents"),
            "the gate report must reach the caller: {}",
            err.message
        );
    }

    #[test]
    fn a_complete_proposal_passes_the_gate_and_keeps_its_advisory_hits() {
        // Template-only boilerplate hits: issue-check exits 1, but the document
        // is complete and must still be returned.
        let report = "issue-check: proposal.html\n  22 hits (4 red-text, 4 XXX, 14 bracketed) \
                      across 8 page(s):\n    [PLACEHOLDER] page 9 line 440: [RIBA]\n";
        let gates = classify_gates(
            Ok(String::new()),
            Ok(String::new()),
            Err(issue_failure(report)),
            INCOMPLETE_MARKER,
        )
        .expect("template boilerplate must not block a complete proposal");
        assert_eq!(gates.issue_hits, 22);
    }

    #[test]
    fn a_mechanical_gate_failure_is_422_not_a_pdf() {
        let block = classify_gates(
            Err(FpRenderError::Failed {
                command: "--column-check".into(),
                code: Some(1),
                stdout: "1 element not at top of column on page 4".into(),
                stderr: String::new(),
            }),
            Ok(String::new()),
            Ok(String::new()),
            INCOMPLETE_MARKER,
        )
        .unwrap_err();
        let err = render_error_to_api(GatedRenderError::Gate(block));
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(err.message.contains("--column-check failed"));
        assert!(err.message.contains("not at top of column"));
    }

    #[test]
    fn an_unconfigured_render_backend_is_503_not_422() {
        let err = render_error_to_api(GatedRenderError::Render(FpRenderError::NotConfigured));
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(err.message.contains(FP_TEMPLATE_ROOT_ENV));
    }

    #[test]
    fn an_invalid_template_root_is_503_naming_the_missing_file() {
        let err = render_error_to_api(GatedRenderError::Render(
            FpRenderError::TemplateRootInvalid {
                root: "/nope".into(),
                missing: "/nope/fill.py".into(),
            },
        ));
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(err.message.contains("fill.py"));
    }

    #[test]
    fn a_fill_failure_is_422_with_fp_templates_own_report() {
        // fill.py's non-zero exit is manifest-check's failure list.
        let err = render_error_to_api(GatedRenderError::Render(FpRenderError::Failed {
            command: "fill.py".into(),
            code: Some(1),
            stdout: String::new(),
            stderr: "manifest-check FAILED - 1 issue(s):\n  - section 'areas' action=override \
                     but no '## areas' body block found"
                .into(),
        }));
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(err.message.contains("no '## areas' body block"));
    }

    #[test]
    fn a_missing_fee_is_404_from_the_shared_fetch_helper() {
        // The route delegates to fee_export::fetch_fee_with_links, whose
        // not-found path is ApiError::not_found. Asserting the mapping here
        // keeps the 404 contract visible at the route.
        let err = ApiError::not_found("Fee", "25_96501_9");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert!(err.message.contains("25_96501_9"));
    }

    #[test]
    fn narrative_request_maps_onto_the_core_narrative_type() {
        let req = FpNarrativeRequest {
            project_details: None,
            reference_documents: Some(vec!["RFQ received 3 August 2026".into()]),
            areas: Some("- All building facades".into()),
            assumptions: None,
        };
        let narrative: FpNarrative = req.into();
        assert_eq!(
            narrative.reference_documents.as_deref(),
            Some(&["RFQ received 3 August 2026".to_string()][..])
        );
        assert_eq!(narrative.areas.as_deref(), Some("- All building facades"));
        assert!(narrative.project_details.is_none());
    }
}
