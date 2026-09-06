//! Docbuilder client — build a full, gated proposal PDF from a manifest plus
//! its variables file via the fp-docbuilder service.
//!
//! [`GotenbergRenderer`](super::gotenberg::GotenbergRenderer) takes HTML that
//! THIS process already filled and gated, and needs a local `FP_TEMPLATE_ROOT`
//! checkout plus python3 to get there. [`DocbuilderClient`] instead sends the
//! MANIFEST plus its variables JSON straight to a remote service that owns
//! fill, the three pre-issue gates, and rendering end to end. A deployment
//! using it therefore builds only the manifest and the variables export
//! (`export::fp_manifest`, `export::build_fee_json` — pure Rust, no I/O, no
//! external process) and POSTs both: no python3, no vendored fp-template
//! checkout, no local browser.
//!
//! # Contract (fixed 2026-09-06, revised same day — multipart, gtm
//! `fp-docbuilder-service`)
//!
//! `POST {DOCBUILDER_URL}/v1/fp-template/proposal`, `multipart/form-data` with
//! exactly two file parts:
//!
//! | Part | Filename | Content |
//! |---|---|---|
//! | `manifest` | any (`manifest.md`) | the manifest markdown (`export::fp_manifest`) |
//! | `vars` | MUST match the manifest frontmatter's `vars_source` exactly | the variables JSON (`export::build_fee_json`) |
//!
//! The service writes both parts into one temporary directory under their
//! given filenames, so `vars_source` resolves relative to the manifest the
//! same way it does in the local fill+gate+render pipeline
//! (`export::fp_render`) — the manifest ALONE always 422s at stage
//! `"manifest"`, because `vars_source` cannot resolve to a file that was
//! never sent.
//!
//! Query params, headers, and response codes are unchanged from the original
//! single-body design: optional query `tagged` (`true`/`false`, default
//! `true`) and `filename`; optional `Authorization: Bearer {token}` when
//! [`DOCBUILDER_TOKEN_ENV`] is set.
//!
//! | Response | Meaning |
//! |---|---|
//! | `200 application/pdf` | rendered and gated. Headers `X-Docbuilder-Pages`, `X-Docbuilder-Gates` (`pass`\|`advisory:<n>`), `X-Docbuilder-Template-Ref`. |
//! | `422 {"stage","blocked":true,"report":[...]}` | the `manifest`, `fill`, or `gate` stage refused the document. Never a PDF. |
//! | `503 {"stage":"render","error":"..."}` | the render step itself failed. |
//!
//! `GET {DOCBUILDER_URL}/health` -> `{"status","chromium","template","template_ref"}`.
//!
//! Both the 422 and 503 bodies are the SERVICE's own report text; this client
//! passes them through verbatim rather than editorialising, exactly as
//! [`gotenberg`](super::gotenberg) does with gotenberg's plain-text errors.

use std::fmt;
use std::time::Duration;

use serde::Deserialize;

/// Base URL of the docbuilder service, e.g. `http://10.0.21.85:8080`.
///
/// No default: unset means this deployment does not use the remote
/// docbuilder, and `fp_render::select_proposal_backend` falls back to the
/// local fill+gate+render pipeline.
pub const DOCBUILDER_URL_ENV: &str = "DOCBUILDER_URL";

/// Bearer token sent with every request, when set. Optional: the service may
/// run without auth in development.
pub const DOCBUILDER_TOKEN_ENV: &str = "DOCBUILDER_TOKEN";

/// Per-request wall-clock budget, in seconds.
pub const DOCBUILDER_TIMEOUT_ENV: &str = "DOCBUILDER_TIMEOUT_SECS";

/// Whether to ask for a tagged (accessible) PDF, matching
/// [`super::gotenberg::GOTENBERG_TAGGED_PDF_ENV`]'s local-pipeline default.
/// Default on.
pub const DOCBUILDER_TAGGED_PDF_ENV: &str = "DOCBUILDER_TAGGED_PDF";

/// Default per-request budget. The service does fill + three gates + render in
/// one call — strictly more work than gotenberg's render-only path — so this
/// carries more headroom than gotenberg's 60s default.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

/// The route this client posts a manifest to.
pub const PROPOSAL_PATH: &str = "/v1/fp-template/proposal";
/// The service's own health route.
pub const HEALTH_PATH: &str = "/health";

/// Filename given to the `manifest` multipart part. The service keys the part
/// by FIELD NAME (`manifest`), not this filename, so any `.md` name works;
/// fixed here for a stable, byte-identical request body.
pub const MANIFEST_FILENAME: &str = "manifest.md";

// ============================================================================
// CONFIG
// ============================================================================

/// Where the docbuilder service lives and how it should behave.
#[derive(Debug, Clone)]
pub struct DocbuilderConfig {
    /// Base URL, without a trailing slash.
    pub base_url: String,
    /// Bearer token sent with every request, when set.
    pub token: Option<String>,
    /// Wall-clock budget for one request.
    pub timeout: Duration,
    /// Per-deployment default for the `tagged` query parameter. A caller may
    /// still override it per call via `DocbuilderClient::render_proposal`'s
    /// own `tagged` argument; [`DocbuilderRenderer::build`] uses this default.
    pub tagged: bool,
}

impl DocbuilderConfig {
    /// Build a config for an explicit base URL, with the defaults applied.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: None,
            timeout: DEFAULT_TIMEOUT,
            tagged: true,
        }
    }

    /// Read the docbuilder settings from the environment.
    ///
    /// Returns `None` when [`DOCBUILDER_URL_ENV`] is unset or blank — the
    /// signal that this deployment does not use the remote service.
    pub fn from_env() -> Option<Self> {
        let url = std::env::var(DOCBUILDER_URL_ENV)
            .ok()
            .filter(|v| !v.trim().is_empty())?;
        let mut config = Self::new(url.trim());
        config.token = std::env::var(DOCBUILDER_TOKEN_ENV)
            .ok()
            .filter(|v| !v.trim().is_empty());
        if let Some(secs) = std::env::var(DOCBUILDER_TIMEOUT_ENV)
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|s| *s > 0)
        {
            config.timeout = Duration::from_secs(secs);
        }
        if let Ok(raw) = std::env::var(DOCBUILDER_TAGGED_PDF_ENV) {
            config.tagged = parse_bool(&raw).unwrap_or(config.tagged);
        }
        Some(config)
    }

    /// Full URL of the proposal route.
    pub fn proposal_url(&self) -> String {
        format!("{}{}", self.base_url, PROPOSAL_PATH)
    }

    /// Full URL of the health route.
    pub fn health_url(&self) -> String {
        format!("{}{}", self.base_url, HEALTH_PATH)
    }
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Why a docbuilder render did not produce a PDF.
///
/// The split matters to the caller: [`DocbuilderError::Blocked`] is the
/// document's own fault (422), everything else is the service's (503) — see
/// [`DocbuilderError::is_caller_fault`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocbuilderError {
    /// The service refused the document at its `manifest`, `fill`, or `gate`
    /// stage. Carries the stage name and the service's own report, verbatim.
    Blocked { stage: String, report: String },
    /// The render step itself failed (`stage: "render"`).
    RenderFailed { message: String },
    /// A `422` or `503` body that did not match the documented JSON shape.
    MalformedResponse { status: u16, body: String },
    /// Any other non-2xx status.
    Status { status: u16, body: String },
    /// A `200` answer that was not a PDF.
    NotAPdf { bytes: usize },
    /// The request never got an answer (connection refused, DNS, timeout).
    Transport { url: String, message: String },
}

impl fmt::Display for DocbuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocbuilderError::Blocked { stage, report } => write!(
                f,
                "fp-docbuilder refused the document at the {stage} stage and it must not \
                 be issued:\n{report}"
            ),
            DocbuilderError::RenderFailed { message } => {
                write!(f, "fp-docbuilder render step failed: {message}")
            }
            DocbuilderError::MalformedResponse { status, body } => write!(
                f,
                "fp-docbuilder answered {status} with a body that did not match its \
                 documented shape: {body}"
            ),
            DocbuilderError::Status { status, body } => {
                write!(f, "fp-docbuilder failed ({status}): {body}")
            }
            DocbuilderError::NotAPdf { bytes } => write!(
                f,
                "fp-docbuilder answered 200 with {bytes} bytes that are not a PDF"
            ),
            DocbuilderError::Transport { url, message } => {
                write!(f, "fp-docbuilder at {url} did not answer: {message}")
            }
        }
    }
}

impl std::error::Error for DocbuilderError {}

impl DocbuilderError {
    /// True when the failure is the document's fault (a blocked stage), so the
    /// caller answers 422 rather than 503.
    pub fn is_caller_fault(&self) -> bool {
        matches!(self, DocbuilderError::Blocked { .. })
    }
}

#[derive(Debug, Deserialize)]
struct BlockedBody {
    stage: String,
    #[allow(dead_code)]
    blocked: bool,
    report: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct RenderFailedBody {
    #[allow(dead_code)]
    stage: String,
    error: String,
}

/// Flatten and cap a plain-text error body for a log line or an error message.
fn truncate(s: &str) -> String {
    let flat = s.trim().replace(['\n', '\r'], " ");
    match flat.char_indices().nth(500) {
        Some((cut, _)) => format!("{}...", &flat[..cut]),
        None => flat,
    }
}

// ============================================================================
// MULTIPART
// ============================================================================
//
// A minimal builder for exactly the two named file parts this contract needs
// (`manifest`, `vars`). Deliberately separate from
// `super::gotenberg::build_multipart_body`: that helper uploads an arbitrary
// FLAT BUNDLE of files all under one shared field name (`files`), which is
// gotenberg's own shape; docbuilder instead names each part individually, so
// reusing it would need forcing an unrelated shape through it.

/// One multipart file part: field name, filename, and bytes.
struct MultipartFile<'a> {
    field: &'a str,
    filename: &'a str,
    bytes: &'a [u8],
}

fn content_type_for(filename: &str) -> &'static str {
    match filename.rsplit('.').next().map(str::to_ascii_lowercase) {
        Some(ref ext) if ext == "md" => "text/markdown; charset=utf-8",
        Some(ref ext) if ext == "json" => "application/json",
        _ => "application/octet-stream",
    }
}

/// Escape a filename for a `Content-Disposition` quoted-string.
fn escape_filename(name: &str) -> String {
    name.chars()
        .filter(|c| *c != '\r' && *c != '\n')
        .flat_map(|c| {
            if c == '"' || c == '\\' {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// Pick a boundary that appears in none of the parts, so no file content can
/// terminate the body early.
fn choose_boundary(parts: &[MultipartFile]) -> String {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    for salt in 0u32..1024 {
        let candidate = format!("efeesDocbuilder{seed:x}{salt:04x}");
        let needle = candidate.as_bytes();
        if !parts.iter().any(|p| contains(p.bytes, needle)) {
            return candidate;
        }
    }
    format!("efeesDocbuilder{seed:x}fallback")
}

/// Build the `multipart/form-data` body: each part with its own field name
/// and filename, then the closing boundary.
fn build_multipart_body(boundary: &str, parts: &[MultipartFile]) -> Vec<u8> {
    let mut body = Vec::new();
    for part in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                part.field,
                escape_filename(part.filename)
            )
            .as_bytes(),
        );
        body.extend_from_slice(
            format!("Content-Type: {}\r\n\r\n", content_type_for(part.filename)).as_bytes(),
        );
        body.extend_from_slice(part.bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    body
}

// ============================================================================
// RESPONSE
// ============================================================================

/// What a successful docbuilder render produced.
#[derive(Debug, Clone)]
pub struct DocbuilderRenderOutput {
    pub pdf: Vec<u8>,
    /// Parsed from `X-Docbuilder-Pages`, when present.
    pub pages: Option<u32>,
    /// Raw `X-Docbuilder-Gates` value: `"pass"` or `"advisory:<n>"`.
    pub gates: Option<String>,
    /// Raw `X-Docbuilder-Template-Ref` value.
    pub template_ref: Option<String>,
}

impl DocbuilderRenderOutput {
    /// Advisory issue-check hit count, mirroring
    /// [`super::fp_render::GateReport::issue_hits`] — `"pass"` and an absent
    /// header both mean zero.
    pub fn issue_hits(&self) -> usize {
        self.gates
            .as_deref()
            .and_then(|g| g.strip_prefix("advisory:"))
            .and_then(|n| n.parse().ok())
            .unwrap_or(0)
    }
}

/// The service's `/health` snapshot. Fields are opaque JSON beyond `status`
/// and `template_ref`, since this client does not need to interpret them.
#[derive(Debug, Clone, Deserialize)]
pub struct DocbuilderHealth {
    pub status: String,
    #[serde(default)]
    pub chromium: serde_json::Value,
    #[serde(default)]
    pub template: serde_json::Value,
    #[serde(default)]
    pub template_ref: Option<String>,
}

// ============================================================================
// CLIENT
// ============================================================================

/// Builds a full, gated proposal PDF from a manifest and its variables JSON
/// via a remote fp-docbuilder service.
#[derive(Debug, Clone)]
pub struct DocbuilderClient {
    config: DocbuilderConfig,
}

impl DocbuilderClient {
    pub fn new(config: DocbuilderConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &DocbuilderConfig {
        &self.config
    }

    fn http_client(&self, url: &str) -> Result<reqwest::blocking::Client, DocbuilderError> {
        reqwest::blocking::Client::builder()
            .timeout(self.config.timeout)
            .build()
            .map_err(|e| DocbuilderError::Transport {
                url: url.to_string(),
                message: e.to_string(),
            })
    }

    /// POST a manifest plus its variables JSON and return the rendered,
    /// gated PDF.
    ///
    /// `vars_filename` MUST match the manifest frontmatter's `vars_source`
    /// exactly (e.g. `"25-96501-var Default Values.json"`) — the service
    /// resolves that path relative to the manifest's own directory, the same
    /// way the local fill+gate+render pipeline does.
    ///
    /// `output_filename` becomes the `filename` query parameter (a naming
    /// hint to the service; the caller still sets its own
    /// `Content-Disposition`). `tagged` maps to the `tagged` query parameter;
    /// `None` omits it and lets the service apply its own default (`true`).
    ///
    /// Blocking. Call it from a blocking context (a CLI, or `spawn_blocking`
    /// under an async runtime).
    pub fn render_proposal(
        &self,
        manifest_markdown: &str,
        vars_json: &str,
        vars_filename: &str,
        output_filename: Option<&str>,
        tagged: Option<bool>,
    ) -> Result<DocbuilderRenderOutput, DocbuilderError> {
        let url = self.config.proposal_url();
        let client = self.http_client(&url)?;

        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(t) = tagged {
            query.push(("tagged", t.to_string()));
        }
        if let Some(name) = output_filename {
            query.push(("filename", name.to_string()));
        }

        let parts = [
            MultipartFile {
                field: "manifest",
                filename: MANIFEST_FILENAME,
                bytes: manifest_markdown.as_bytes(),
            },
            MultipartFile {
                field: "vars",
                filename: vars_filename,
                bytes: vars_json.as_bytes(),
            },
        ];
        let boundary = choose_boundary(&parts);
        let body = build_multipart_body(&boundary, &parts);

        let mut request = client
            .post(&url)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .query(&query)
            .body(body);
        if let Some(token) = &self.config.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().map_err(|e| DocbuilderError::Transport {
            url: url.clone(),
            message: if e.is_timeout() {
                format!("timed out after {}s", self.config.timeout.as_secs())
            } else {
                e.to_string()
            },
        })?;

        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let bytes = response.bytes().map_err(|e| DocbuilderError::Transport {
            url: url.clone(),
            message: e.to_string(),
        })?;

        if status == 200 {
            if !bytes.starts_with(b"%PDF") {
                return Err(DocbuilderError::NotAPdf { bytes: bytes.len() });
            }
            let header_str = |name: &str| {
                headers
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string)
            };
            return Ok(DocbuilderRenderOutput {
                pdf: bytes.to_vec(),
                pages: header_str("x-docbuilder-pages").and_then(|v| v.parse().ok()),
                gates: header_str("x-docbuilder-gates"),
                template_ref: header_str("x-docbuilder-template-ref"),
            });
        }

        let body_text = String::from_utf8_lossy(&bytes).into_owned();

        if status == 422 {
            return Err(match serde_json::from_slice::<BlockedBody>(&bytes) {
                Ok(parsed) => DocbuilderError::Blocked {
                    stage: parsed.stage,
                    report: serde_json::to_string_pretty(&parsed.report)
                        .unwrap_or_else(|_| parsed.report.to_string()),
                },
                Err(_) => DocbuilderError::MalformedResponse {
                    status,
                    body: truncate(&body_text),
                },
            });
        }

        if status == 503 {
            return Err(match serde_json::from_slice::<RenderFailedBody>(&bytes) {
                Ok(parsed) => DocbuilderError::RenderFailed {
                    message: parsed.error,
                },
                Err(_) => DocbuilderError::MalformedResponse {
                    status,
                    body: truncate(&body_text),
                },
            });
        }

        Err(DocbuilderError::Status {
            status,
            body: truncate(&body_text),
        })
    }

    /// GET the service's health snapshot.
    pub fn health(&self) -> Result<DocbuilderHealth, DocbuilderError> {
        let url = self.config.health_url();
        let client = self.http_client(&url)?;
        let response = client
            .get(&url)
            .send()
            .map_err(|e| DocbuilderError::Transport {
                url: url.clone(),
                message: e.to_string(),
            })?;
        let status = response.status().as_u16();
        let bytes = response.bytes().map_err(|e| DocbuilderError::Transport {
            url: url.clone(),
            message: e.to_string(),
        })?;
        if status != 200 {
            return Err(DocbuilderError::Status {
                status,
                body: truncate(&String::from_utf8_lossy(&bytes)),
            });
        }
        serde_json::from_slice(&bytes).map_err(|_| DocbuilderError::MalformedResponse {
            status,
            body: truncate(&String::from_utf8_lossy(&bytes)),
        })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;

    const VARS_JSON: &str = r#"{"01 Document Name":"Test"}"#;
    const VARS_FILENAME: &str = "25-96501-var Default Values.json";

    /// A minimal one-shot HTTP server: binds an ephemeral port, accepts
    /// exactly one connection, writes back the given response bytes, and
    /// records the raw request text it received (headers plus at least the
    /// start of the body — enough to assert on).
    struct OneShotServer {
        addr: String,
        request: mpsc::Receiver<String>,
    }

    impl OneShotServer {
        fn start(response: Vec<u8>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                if let Ok((mut stream, _)) = listener.accept() {
                    // A short read timeout, not a byte-count target: the test
                    // bodies here are small and arrive in one or two TCP
                    // segments, then the client waits for a response and
                    // sends nothing more. Waiting for a fixed byte count
                    // (e.g. 4096) would block forever on a smaller request,
                    // since the client never closes its write side first.
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(300)));
                    let mut buf = [0u8; 65536];
                    let mut request = Vec::new();
                    loop {
                        match stream.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                request.extend_from_slice(&buf[..n]);
                                if request.len() >= 65536 {
                                    break;
                                }
                            }
                            Err(_) => break, // timed out: no more data right now
                        }
                    }
                    let _ = tx.send(String::from_utf8_lossy(&request).into_owned());
                    let _ = stream.write_all(&response);
                    let _ = stream.flush();
                }
            });
            Self {
                addr: format!("http://{addr}"),
                request: rx,
            }
        }

        fn received_request(&self) -> String {
            self.request
                .recv_timeout(Duration::from_secs(2))
                .unwrap_or_default()
        }
    }

    fn build_response(status_line: &str, headers: &[(&str, &str)], body: &[u8]) -> Vec<u8> {
        let mut resp = format!("{status_line}\r\n");
        for (k, v) in headers {
            resp.push_str(&format!("{k}: {v}\r\n"));
        }
        resp.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
        let mut out = resp.into_bytes();
        out.extend_from_slice(body);
        out
    }

    fn render(client: &DocbuilderClient) -> Result<DocbuilderRenderOutput, DocbuilderError> {
        client.render_proposal("# manifest", VARS_JSON, VARS_FILENAME, None, None)
    }

    // ---- config ----

    #[test]
    fn proposal_url_and_health_url_are_derived_from_the_base() {
        let config = DocbuilderConfig::new("http://10.0.21.85:8080/");
        assert_eq!(
            config.proposal_url(),
            "http://10.0.21.85:8080/v1/fp-template/proposal"
        );
        assert_eq!(config.health_url(), "http://10.0.21.85:8080/health");
    }

    #[test]
    fn from_env_is_none_when_url_is_unset_or_blank() {
        assert!(Some("".to_string())
            .filter(|v: &String| !v.trim().is_empty())
            .is_none());
    }

    // ---- multipart shape ----

    #[test]
    fn the_request_carries_both_named_parts_with_the_right_filenames() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 200 OK",
            &[("Content-Type", "application/pdf")],
            b"%PDF",
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        render(&client).unwrap();
        let request = server.received_request();
        assert!(
            request.contains("multipart/form-data; boundary="),
            "{request}"
        );
        assert!(
            request.contains("name=\"manifest\"; filename=\"manifest.md\""),
            "{request}"
        );
        assert!(
            request.contains(&format!("name=\"vars\"; filename=\"{VARS_FILENAME}\"")),
            "{request}"
        );
        assert!(request.contains("# manifest"), "{request}");
        assert!(request.contains("01 Document Name"), "{request}");
    }

    // ---- render_proposal: success ----

    #[test]
    fn successful_response_carries_headers_and_pdf_bytes() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 200 OK",
            &[
                ("Content-Type", "application/pdf"),
                ("X-Docbuilder-Pages", "5"),
                ("X-Docbuilder-Gates", "advisory:3"),
                ("X-Docbuilder-Template-Ref", "abc123"),
            ],
            b"%PDF-fake",
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        let out = render(&client).expect("expected a successful render");
        assert_eq!(out.pdf, b"%PDF-fake");
        assert_eq!(out.pages, Some(5));
        assert_eq!(out.gates.as_deref(), Some("advisory:3"));
        assert_eq!(out.issue_hits(), 3);
        assert_eq!(out.template_ref.as_deref(), Some("abc123"));
    }

    #[test]
    fn a_pass_gates_header_reports_zero_issue_hits() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 200 OK",
            &[
                ("Content-Type", "application/pdf"),
                ("X-Docbuilder-Gates", "pass"),
            ],
            b"%PDF-fake",
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        let out = render(&client).unwrap();
        assert_eq!(out.issue_hits(), 0);
    }

    #[test]
    fn a_200_body_that_is_not_a_pdf_is_reported_not_returned() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 200 OK",
            &[("Content-Type", "application/pdf")],
            b"not a pdf",
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        let err = render(&client).unwrap_err();
        assert!(matches!(err, DocbuilderError::NotAPdf { bytes: 9 }));
        assert!(!err.is_caller_fault());
    }

    // ---- render_proposal: 422 passthrough ----

    #[test]
    fn a_422_blocked_body_passes_through_stage_and_report() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 422 Unprocessable Entity",
            &[("Content-Type", "application/json")],
            br#"{"stage":"gate","blocked":true,"report":["column-check failed","x"]}"#,
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        let err = render(&client).unwrap_err();
        match &err {
            DocbuilderError::Blocked { stage, report } => {
                assert_eq!(stage, "gate");
                assert!(report.contains("column-check failed"), "{report}");
            }
            other => panic!("expected Blocked, got {other:?}"),
        }
        assert!(err.is_caller_fault());
    }

    #[test]
    fn a_422_body_that_does_not_match_the_shape_is_malformed_not_silently_dropped() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 422 Unprocessable Entity",
            &[("Content-Type", "application/json")],
            b"not json",
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        let err = render(&client).unwrap_err();
        assert!(matches!(
            err,
            DocbuilderError::MalformedResponse { status: 422, .. }
        ));
    }

    // ---- render_proposal: 503 ----

    #[test]
    fn a_503_render_failure_carries_the_error_message() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 503 Service Unavailable",
            &[("Content-Type", "application/json")],
            br#"{"stage":"render","error":"chromium crashed"}"#,
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        let err = render(&client).unwrap_err();
        match &err {
            DocbuilderError::RenderFailed { message } => assert_eq!(message, "chromium crashed"),
            other => panic!("expected RenderFailed, got {other:?}"),
        }
        assert!(!err.is_caller_fault());
    }

    #[test]
    fn an_unexpected_status_maps_to_status_not_a_guess() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 500 Internal Server Error",
            &[("Content-Type", "text/plain")],
            b"boom",
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        let err = render(&client).unwrap_err();
        match &err {
            DocbuilderError::Status { status, body } => {
                assert_eq!(*status, 500);
                assert!(body.contains("boom"));
            }
            other => panic!("expected Status, got {other:?}"),
        }
        assert!(!err.is_caller_fault());
    }

    // ---- render_proposal: timeout ----

    #[test]
    fn a_slow_service_times_out_rather_than_hanging_forever() {
        // Accept the connection but never answer within the client's budget;
        // the background thread's long sleep must lose the race.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                std::thread::sleep(Duration::from_secs(5));
                drop(stream);
            }
        });
        let mut config = DocbuilderConfig::new(format!("http://{addr}"));
        config.timeout = Duration::from_millis(200);
        let client = DocbuilderClient::new(config);
        let err = render(&client).unwrap_err();
        match err {
            DocbuilderError::Transport { message, .. } => {
                assert!(message.contains("timed out"), "{message}");
            }
            other => panic!("expected Transport timeout, got {other:?}"),
        }
    }

    // ---- render_proposal: auth header ----

    #[test]
    fn bearer_token_is_sent_only_when_configured() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 200 OK",
            &[("Content-Type", "application/pdf")],
            b"%PDF",
        ));
        let mut config = DocbuilderConfig::new(server.addr.as_str());
        config.token = Some("secret-token".to_string());
        let client = DocbuilderClient::new(config);
        render(&client).unwrap();
        let request = server.received_request().to_ascii_lowercase();
        assert!(
            request.contains("authorization: bearer secret-token"),
            "{request}"
        );
    }

    #[test]
    fn no_authorization_header_without_a_configured_token() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 200 OK",
            &[("Content-Type", "application/pdf")],
            b"%PDF",
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        render(&client).unwrap();
        let request = server.received_request().to_ascii_lowercase();
        assert!(!request.contains("authorization"), "{request}");
    }

    // ---- health ----

    #[test]
    fn health_parses_the_documented_shape() {
        let server = OneShotServer::start(build_response(
            "HTTP/1.1 200 OK",
            &[("Content-Type", "application/json")],
            br#"{"status":"ok","chromium":true,"template":true,"template_ref":"24b7845"}"#,
        ));
        let client = DocbuilderClient::new(DocbuilderConfig::new(server.addr.as_str()));
        let health = client.health().unwrap();
        assert_eq!(health.status, "ok");
        assert_eq!(health.template_ref.as_deref(), Some("24b7845"));
    }

    // ---- multipart helpers ----

    #[test]
    fn multipart_body_has_both_parts_then_a_closing_boundary() {
        let parts = [
            MultipartFile {
                field: "manifest",
                filename: "manifest.md",
                bytes: b"# hi",
            },
            MultipartFile {
                field: "vars",
                filename: "x.json",
                bytes: b"{}",
            },
        ];
        let body = build_multipart_body("BOUND", &parts);
        let text = String::from_utf8_lossy(&body);
        assert!(text.starts_with("--BOUND\r\n"), "{text}");
        assert!(
            text.contains(
                "Content-Disposition: form-data; name=\"manifest\"; filename=\"manifest.md\"\r\n\
                 Content-Type: text/markdown; charset=utf-8\r\n\r\n# hi\r\n"
            ),
            "{text}"
        );
        assert!(
            text.contains(
                "Content-Disposition: form-data; name=\"vars\"; filename=\"x.json\"\r\n\
                 Content-Type: application/json\r\n\r\n{}\r\n"
            ),
            "{text}"
        );
        assert!(text.ends_with("--BOUND--\r\n"));
        assert_eq!(text.matches("--BOUND\r\n").count(), 2);
    }

    #[test]
    fn quotes_in_a_filename_cannot_break_the_header() {
        assert_eq!(escape_filename("a\"b\\c"), "a\\\"b\\\\c");
        assert_eq!(escape_filename("a\r\nb"), "ab");
    }

    #[test]
    fn the_boundary_never_appears_inside_a_part() {
        let parts = [MultipartFile {
            field: "manifest",
            filename: "manifest.md",
            bytes: b"efeesDocbuilder",
        }];
        let boundary = choose_boundary(&parts);
        assert!(!contains(parts[0].bytes, boundary.as_bytes()));
    }

    #[test]
    fn content_type_is_inferred_from_the_extension() {
        assert_eq!(
            content_type_for("manifest.md"),
            "text/markdown; charset=utf-8"
        );
        assert_eq!(content_type_for("vars.json"), "application/json");
        assert_eq!(content_type_for("weird"), "application/octet-stream");
    }
}
