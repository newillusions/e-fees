//! fp-template proposal endpoints — fee record to fill manifest to rendered PDF.
//!
//! Two routes, deliberately split so the cheap half works everywhere:
//!
//! | Route | Needs | Returns |
//! |---|---|---|
//! | `GET /fees/{id}/fp-manifest` | database only | the manifest markdown |
//! | `POST /fees/{id}/fp-proposal` | database + a proposal backend | `application/pdf` |
//!
//! The manifest route is pure computation over records this API already
//! serves (`export::fp_manifest`, no I/O), so it is always available. The
//! render route builds the same manifest and hands it to whichever backend
//! [`select_proposal_backend`] chose:
//!
//! | `DOCBUILDER_URL` | Backend | Needs |
//! |---|---|---|
//! | set | the remote fp-docbuilder service | a reachable docbuilder service; no `FP_TEMPLATE_ROOT`, no python3, no browser |
//! | unset | the local fill+gate+render pipeline | `FP_TEMPLATE_ROOT`, python3, and (unless `GOTENBERG_URL` is also set) a local headless Chrome |
//!
//! `DOCBUILDER_URL` is the production path: the service owns fill, the three
//! pre-issue gates, and rendering end to end, so this deployment needs none of
//! fp-template's own toolchain. The local pipeline (`GOTENBERG_URL` set or
//! not) remains for development. When neither `DOCBUILDER_URL` nor
//! `FP_TEMPLATE_ROOT` is set the route answers `503 Service Unavailable`
//! rather than failing obscurely; see `e_fees_core::export::fp_render`'s
//! module docs for the remaining local-browser dependency in the fill step.
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
    build_proposal_pdf_gated_with, select_proposal_backend, select_renderer, FpRenderError,
    GateBlock, GatedRenderError, ProposalBackend, ProposalBuildError, ProposalBuilder,
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

    let backend = select_proposal_backend().ok_or_else(|| {
        ApiError::service_unavailable(
            "Proposal rendering not configured (set DOCBUILDER_URL for the remote service, \
             or FP_TEMPLATE_ROOT for local rendering). The manifest is still available at \
             /fees/{id}/fp-manifest."
                .to_string(),
        )
    })?;
    // Only the local pipeline needs a checkout to validate; the docbuilder
    // path needs nothing from this host beyond the manifest it is about to
    // build below.
    if let ProposalBackend::Local(config) = &backend {
        config
            .validate()
            .map_err(|e| ApiError::service_unavailable(e.to_string()))?;
    }

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
    let residual_count = manifest.residuals.len();
    let filename = format!("{}-FP.pdf", number);

    // Building the PDF is slow either way — one blocking HTTP call to
    // docbuilder, or the local fill+gate+render pipeline — so it stays off
    // the async runtime's worker threads. Both paths gate BEFORE returning a
    // PDF, so a document that must not be issued never becomes a response
    // body, and with a remote backend it never leaves this host.
    let render: Result<ProposalRenderResult, tokio::task::JoinError> = match backend {
        ProposalBackend::Docbuilder(builder) => {
            let markdown = manifest.markdown.clone();
            // vars_filename MUST match the manifest frontmatter's
            // `vars_source` exactly - the service resolves that path
            // relative to the manifest's own directory, same as the local
            // pipeline resolves it relative to a real file on disk.
            let vars_json = serde_json::to_string_pretty(&variables).unwrap_or_default();
            let vars_filename = options.vars_source.clone();
            let filename_for_task = filename.clone();
            tokio::task::spawn_blocking(move || {
                builder
                    .build(
                        &markdown,
                        &vars_json,
                        &vars_filename,
                        Some(&filename_for_task),
                    )
                    .map(|out| (out.pdf, out.issue_hits, out.backend))
                    .map_err(ProposalRenderError::Build)
            })
            .await
        }
        ProposalBackend::Local(config) => {
            // Per-request scratch directory. The manifest and its vars
            // file MUST share a directory - fp-template resolves
            // vars_source relative to the manifest.
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
            let work_dir_for_task = work_dir.clone();

            let result = tokio::task::spawn_blocking(move || -> ProposalRenderResult {
                let write = |path: std::path::PathBuf, body: String| -> Result<(), FpRenderError> {
                    std::fs::write(path, body)?;
                    Ok(())
                };
                std::fs::create_dir_all(&work_dir_for_task).map_err(|e| {
                    ProposalRenderError::Gated(GatedRenderError::Render(FpRenderError::Io(e)))
                })?;
                write(
                    work_dir_for_task.join(&vars_source),
                    serde_json::to_string_pretty(&variables).unwrap_or_default(),
                )
                .map_err(|e| ProposalRenderError::Gated(GatedRenderError::Render(e)))?;
                let manifest_path = work_dir_for_task.join("manifest.md");
                write(manifest_path.clone(), markdown)
                    .map_err(|e| ProposalRenderError::Gated(GatedRenderError::Render(e)))?;

                let html_path = work_dir_for_task.join("proposal.html");
                let pdf_path = work_dir_for_task.join("proposal.pdf");
                let renderer = select_renderer(&config);
                let built = build_proposal_pdf_gated_with(
                    &config,
                    renderer.as_ref(),
                    &manifest_path,
                    &html_path,
                    &pdf_path,
                )
                .map_err(ProposalRenderError::Gated)?;
                let bytes = std::fs::read(&pdf_path).map_err(|e| {
                    ProposalRenderError::Gated(GatedRenderError::Render(FpRenderError::Io(e)))
                })?;
                Ok((bytes, built.gates.issue_hits, built.backend))
            })
            .await;
            // Always clean up, whatever happened.
            let _ = std::fs::remove_dir_all(&work_dir);
            result
        }
    };

    let (pdf, issue_hits, backend) = match render {
        Ok(Ok(ok)) => ok,
        Ok(Err(e)) => {
            warn!("fp proposal refused for fee {key}: {e}");
            return Err(proposal_render_error_to_api(e));
        }
        Err(join_err) => {
            warn!("fp render task panicked for fee {key}: {join_err}");
            return Err(ApiError::service_unavailable(
                "Proposal rendering failed unexpectedly".to_string(),
            ));
        }
    };

    info!(
        "rendered fp proposal for fee {key} via {backend} ({} bytes, \
         {residual_count} residual(s), {issue_hits} advisory issue-check hit(s))",
        pdf.len(),
    );

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
    if let Ok(value) = HeaderValue::from_str(&issue_hits.to_string()) {
        response.headers_mut().insert("x-fp-issue-hits", value);
    }
    // Which backend produced this PDF, so a caller (or a support session) can
    // tell a locally rendered document from a gotenberg one without guessing.
    if let Ok(value) = HeaderValue::from_str(backend) {
        response.headers_mut().insert("x-fp-render-backend", value);
    }
    Ok(response)
}

/// Map a gated-render failure onto the right HTTP status.
///
/// A gate BLOCK is 422 carrying the gate's own report - never a PDF. A missing
/// or invalid template checkout is 503, because it is this host's configuration
/// and not the caller's request. Everything else (a fill failure, which is
/// fp-template's manifest-check report) is 422 with the report verbatim.
///
/// Gotenberg splits the same way, and the split is measured rather than
/// assumed (2026-09-06, gotenberg 8.36.0): a bundle it cannot accept answers
/// `400 Invalid form data: ...`, which is OUR malformed request and therefore
/// 422; a connection failure, a timeout, or any other non-2xx means the
/// service is unavailable and is 503, exactly like an unconfigured backend.
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
        GatedRenderError::Render(FpRenderError::Gotenberg(g)) => {
            if g.is_caller_fault() {
                ApiError::unprocessable(g.to_string())
            } else {
                ApiError::service_unavailable(g.to_string())
            }
        }
        GatedRenderError::Render(other) => ApiError::unprocessable(other.to_string()),
    }
}

/// A successful build: PDF bytes, advisory issue-check hit count, and the
/// backend name — the shape both of [`render_fp_proposal`]'s branches produce.
type ProposalRenderResult = Result<(Vec<u8>, usize, &'static str), ProposalRenderError>;

/// Backend-independent render failure. Exists only to let
/// [`render_fp_proposal`]'s two backend branches share one `match` and one
/// error-to-status mapping; it is never constructed anywhere else.
enum ProposalRenderError {
    /// The local fill+gate+render pipeline failed.
    Gated(GatedRenderError),
    /// The remote docbuilder service refused or failed to build the document.
    Build(ProposalBuildError),
}

impl std::fmt::Display for ProposalRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalRenderError::Gated(e) => write!(f, "{e}"),
            ProposalRenderError::Build(e) => write!(f, "{e}"),
        }
    }
}

/// Map either backend's failure onto the right HTTP status.
///
/// [`ProposalBuildError::Blocked`] (a docbuilder `422`) is the document's own
/// fault, exactly like a local [`GateBlock`] — 422 carrying the report.
/// [`ProposalBuildError::Unavailable`] (a docbuilder `503`, or a transport
/// failure) is this deployment's, exactly like an unreachable gotenberg — 503.
fn proposal_render_error_to_api(e: ProposalRenderError) -> ApiError {
    match e {
        ProposalRenderError::Gated(gated) => render_error_to_api(gated),
        ProposalRenderError::Build(ProposalBuildError::Blocked(msg)) => {
            ApiError::unprocessable(msg)
        }
        ProposalRenderError::Build(ProposalBuildError::Unavailable(msg)) => {
            ApiError::service_unavailable(msg)
        }
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
    use e_fees_core::export::fp_render::{classify_gates, GateBlock, FP_TEMPLATE_ROOT_ENV};
    use e_fees_core::export::gotenberg::GotenbergError;

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

    // ---- gotenberg backend mapping ----

    #[test]
    fn a_gotenberg_rejection_is_422_because_our_bundle_was_wrong() {
        // Measured shape: gotenberg answers 400 with this exact text when the
        // upload carries no index.html member.
        let err = render_error_to_api(GatedRenderError::Render(FpRenderError::Gotenberg(
            GotenbergError::BadRequest {
                status: 400,
                body: "Invalid form data: form file 'index.html' is required".into(),
            },
        )));
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(err.message.contains("index.html"), "{}", err.message);
    }

    #[test]
    fn an_unreachable_gotenberg_is_503_not_422() {
        let err = render_error_to_api(GatedRenderError::Render(FpRenderError::Gotenberg(
            GotenbergError::Transport {
                url: "http://10.0.23.31:3000/forms/chromium/convert/html".into(),
                message: "connection refused".into(),
            },
        )));
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(err.message.contains("did not answer"), "{}", err.message);
    }

    #[test]
    fn a_gotenberg_server_error_is_503() {
        let err = render_error_to_api(GatedRenderError::Render(FpRenderError::Gotenberg(
            GotenbergError::Status {
                status: 500,
                body: "chromium failed".into(),
            },
        )));
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn a_missing_asset_is_422_naming_the_reference() {
        // The document is unrenderable as written; the caller sees which
        // reference could not be resolved, never a silent blank page.
        let err = render_error_to_api(GatedRenderError::Render(FpRenderError::Gotenberg(
            GotenbergError::AssetMissing {
                referrer: "/tmp/efees-fp-26-97109/proposal.html".into(),
                reference: "../../assets/logo white.svg".into(),
            },
        )));
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(err.message.contains("logo white.svg"), "{}", err.message);
    }

    #[test]
    fn a_reference_escaping_the_allowed_roots_is_422_and_names_no_host_path() {
        // The document asked for a file outside the template and working
        // directories. It is refused before the file is read, and the caller
        // sees the reference, never the container's directory layout.
        let err = render_error_to_api(GatedRenderError::Render(FpRenderError::Gotenberg(
            GotenbergError::EscapesRoot {
                referrer: "proposal.html".into(),
                reference: "../../../../etc/hosts".into(),
            },
        )));
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            err.message.contains("../../../../etc/hosts"),
            "{}",
            err.message
        );
        assert!(err.message.contains("refused"), "{}", err.message);
        assert!(
            !err.message.contains("/tmp/") && !err.message.contains("/var/folders"),
            "no absolute host path may reach the caller: {}",
            err.message
        );
    }

    #[test]
    fn an_oversized_bundle_is_422_not_503() {
        // Our document is too big to send; that is the request's problem, not
        // the service's.
        let err = render_error_to_api(GatedRenderError::Render(FpRenderError::Gotenberg(
            GotenbergError::BundleTooLarge {
                bytes: 40_000_000,
                limit: 26_214_400,
            },
        )));
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(err.message.contains("nothing was sent"), "{}", err.message);
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

    // ---- proposal_render_error_to_api: the docbuilder/local split ----

    #[test]
    fn proposal_render_error_gated_variant_delegates_to_render_error_to_api() {
        // A representative case from render_error_to_api's own suite above,
        // proving the Local branch's mapping is shared, not reimplemented.
        let err = proposal_render_error_to_api(ProposalRenderError::Gated(
            GatedRenderError::Render(FpRenderError::NotConfigured),
        ));
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(err.message.contains(FP_TEMPLATE_ROOT_ENV));
    }

    #[test]
    fn proposal_render_error_build_blocked_is_422_carrying_the_message() {
        let err =
            proposal_render_error_to_api(ProposalRenderError::Build(ProposalBuildError::Blocked(
                "fp-docbuilder refused the document at the gate stage".into(),
            )));
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(err.message.contains("gate stage"), "{}", err.message);
    }

    #[test]
    fn proposal_render_error_build_unavailable_is_503_carrying_the_message() {
        let err = proposal_render_error_to_api(ProposalRenderError::Build(
            ProposalBuildError::Unavailable(
                "fp-docbuilder at http://10.0.21.85:8080 did not answer: refused".into(),
            ),
        ));
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(err.message.contains("did not answer"), "{}", err.message);
    }
}
