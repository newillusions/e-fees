//! fp-template render driver — runs the fill engine and the PDF renderer.
//!
//! Two external commands, both owned by fp-template and invoked exactly as its
//! own docs specify:
//!
//! ```text
//! python3 <root>/fill.py <manifest.md> <output.html>
//! <root>/render.sh <output.pdf> <output.html>
//! ```
//!
//! `fill.py` runs fp-template's `manifest-check.py` gate first and exits
//! non-zero on a bad manifest, so a failed fill is a real manifest problem, not
//! a rendering accident. `render.sh` drives headless Chrome; it also exposes
//! three read-only pre-issue gates (`--issue-check`, `--column-check`,
//! `--quote-check`) which [`run_gate`] wraps.
//!
//! # Two render backends, one selected by environment
//!
//! The PDF step is behind [`ProposalPdfRenderer`]:
//!
//! | Backend | Selected when | Needs |
//! |---|---|---|
//! | [`GotenbergRenderer`] | `GOTENBERG_URL` is set | an HTTP reachable gotenberg |
//! | [`LocalChromiumRenderer`] | otherwise | a local headless Chrome |
//!
//! [`select_renderer`] applies that rule, and every entry point in this module
//! goes through it, so the CLI and the API agree without either passing a flag.
//!
//! # Deployment note (named, not hidden)
//!
//! This driver always needs a checkout of fp-template plus python3 (with
//! `pyyaml`, `beautifulsoup4`), because the FILL step and the three pre-issue
//! gates are fp-template's own python. [`FpRenderConfig::from_env`] returns
//! `None` unless `FP_TEMPLATE_ROOT` is set, and callers report "render backend
//! not configured" rather than pretend.
//!
//! Gotenberg removes the browser from the RENDER step only. The FILL step
//! still launches a local Chromium through Playwright: fp-template's
//! `engine/measure.py` measures every block's height in a real render, has no
//! remote or cached mode, and raises rather than guessing. So an image built
//! with python3 and no browser can serve the manifest route and run the gates,
//! but `fill.py` will fail — see `docs/development/FP-TEMPLATE-INTEGRATION.md`.
//!
//! # A third option: skip this driver entirely
//!
//! Everything above is the LOCAL pipeline: this process runs `fill.py`, the
//! three gates, and a PDF renderer, in that order. [`ProposalBuilder`] and its
//! only implementation, [`DocbuilderRenderer`], are a wholly separate seam for
//! a REMOTE pipeline: the fp-docbuilder service owns fill, gate, and render
//! together, so a caller using it never touches [`FpRenderConfig`],
//! `fill.py`, or a browser at all. [`select_proposal_backend`] is the single
//! decision point between the two, keyed on `DOCBUILDER_URL`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::export::docbuilder::{DocbuilderClient, DocbuilderConfig};
use crate::export::gotenberg::{GotenbergConfig, GotenbergError, GotenbergRenderer};

/// Environment variable naming the fp-template checkout to render with.
pub const FP_TEMPLATE_ROOT_ENV: &str = "FP_TEMPLATE_ROOT";

/// Default wall-clock budget for one fill+render pass. The proven Al Furjan
/// manifest fills in ~2s and renders in ~2s on a warm host; 120s leaves room
/// for a cold Chromium start without hanging a request forever.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

/// Where fp-template lives and how long it may take.
#[derive(Debug, Clone)]
pub struct FpRenderConfig {
    /// Absolute path to the `fp-template` directory (the one containing
    /// `fill.py`, `render.sh` and `engine/`).
    pub root: PathBuf,
    /// Wall-clock budget applied to each external command.
    pub timeout: Duration,
    /// Interpreter used for `fill.py`.
    pub python: String,
}

impl FpRenderConfig {
    /// Build a config for an explicit root.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            timeout: DEFAULT_TIMEOUT,
            python: "python3".to_string(),
        }
    }

    /// Read `FP_TEMPLATE_ROOT` from the environment.
    ///
    /// Returns `None` when unset — the caller should surface "render backend
    /// not configured" rather than guessing a path.
    pub fn from_env() -> Option<Self> {
        std::env::var(FP_TEMPLATE_ROOT_ENV)
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(Self::new)
    }

    fn fill_py(&self) -> PathBuf {
        self.root.join("fill.py")
    }

    fn render_sh(&self) -> PathBuf {
        self.root.join("render.sh")
    }

    /// Verify the configured root actually looks like an fp-template checkout.
    pub fn validate(&self) -> Result<(), FpRenderError> {
        for path in [
            self.fill_py(),
            self.render_sh(),
            self.root.join("index.html"),
        ] {
            if !path.exists() {
                return Err(FpRenderError::TemplateRootInvalid {
                    root: self.root.clone(),
                    missing: path,
                });
            }
        }
        Ok(())
    }
}

/// What a successful render produced.
#[derive(Debug, Clone)]
pub struct FpRenderOutput {
    pub html_path: PathBuf,
    pub pdf_path: PathBuf,
    /// `fill.py`'s stdout — carries the page count and any named residuals.
    pub fill_stdout: String,
}

/// Render failures, each distinguishable by the caller.
#[derive(Debug)]
pub enum FpRenderError {
    /// `FP_TEMPLATE_ROOT` is unset.
    NotConfigured,
    /// The configured root is missing one of fp-template's own entry points.
    TemplateRootInvalid {
        root: PathBuf,
        missing: PathBuf,
    },
    /// The command could not be started at all (missing python3, no execute bit).
    Spawn {
        command: String,
        source: std::io::Error,
    },
    /// The command ran and failed. `stderr` carries fp-template's own message,
    /// which for `fill.py` is the manifest-check failure list.
    Failed {
        command: String,
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    /// The command exceeded the configured budget and was killed.
    TimedOut {
        command: String,
        timeout: Duration,
    },
    /// The command reported success but produced no artefact.
    MissingOutput {
        command: String,
        expected: PathBuf,
    },
    /// The gotenberg render backend failed. Carries its own classification of
    /// whose fault it was — see [`GotenbergError::is_caller_fault`].
    Gotenberg(GotenbergError),
    Io(std::io::Error),
}

impl std::fmt::Display for FpRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FpRenderError::NotConfigured => write!(
                f,
                "fp-template render backend not configured (set {FP_TEMPLATE_ROOT_ENV})"
            ),
            FpRenderError::TemplateRootInvalid { root, missing } => write!(
                f,
                "{} is not an fp-template checkout: {} is missing",
                root.display(),
                missing.display()
            ),
            FpRenderError::Spawn { command, source } => {
                write!(f, "could not start {command}: {source}")
            }
            FpRenderError::Failed {
                command,
                code,
                stderr,
                ..
            } => write!(
                f,
                "{command} failed (exit {}): {}",
                code.map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".into()),
                stderr.trim()
            ),
            FpRenderError::TimedOut { command, timeout } => {
                write!(
                    f,
                    "{command} exceeded {}s and was killed",
                    timeout.as_secs()
                )
            }
            FpRenderError::MissingOutput { command, expected } => write!(
                f,
                "{command} reported success but {} was not written",
                expected.display()
            ),
            FpRenderError::Gotenberg(e) => write!(f, "{e}"),
            FpRenderError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for FpRenderError {}

impl From<std::io::Error> for FpRenderError {
    fn from(e: std::io::Error) -> Self {
        FpRenderError::Io(e)
    }
}

/// Run one external command with a wall-clock budget, capturing both streams.
///
/// Blocking. Call it from a blocking context (a CLI, or `spawn_blocking` under
/// an async runtime).
fn run_with_timeout(
    label: &str,
    mut command: Command,
    timeout: Duration,
) -> Result<String, FpRenderError> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| FpRenderError::Spawn {
            command: label.to_string(),
            source,
        })?;

    let started = Instant::now();
    loop {
        match child.try_wait()? {
            Some(_) => break,
            None => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(FpRenderError::TimedOut {
                        command: label.to_string(),
                        timeout,
                    });
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }

    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(FpRenderError::Failed {
            command: label.to_string(),
            code: output.status.code(),
            stdout,
            stderr,
        });
    }
    Ok(stdout)
}

/// Fill a manifest into a proposal HTML document.
///
/// `manifest_path` must sit next to the `vars_source` file it names —
/// fp-template resolves that path relative to the manifest's own directory.
pub fn fill_manifest(
    config: &FpRenderConfig,
    manifest_path: &Path,
    html_out: &Path,
) -> Result<String, FpRenderError> {
    config.validate()?;
    if let Some(parent) = html_out.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new(&config.python);
    cmd.arg(config.fill_py()).arg(manifest_path).arg(html_out);
    cmd.current_dir(&config.root);

    let stdout = run_with_timeout("fill.py", cmd, config.timeout)?;
    if !html_out.exists() {
        return Err(FpRenderError::MissingOutput {
            command: "fill.py".into(),
            expected: html_out.to_path_buf(),
        });
    }
    Ok(stdout)
}

/// Render a proposal HTML document to PDF via fp-template's own `render.sh`.
pub fn render_pdf(
    config: &FpRenderConfig,
    html_in: &Path,
    pdf_out: &Path,
) -> Result<(), FpRenderError> {
    config.validate()?;
    if let Some(parent) = pdf_out.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new(config.render_sh());
    cmd.arg(pdf_out).arg(html_in);
    cmd.current_dir(&config.root);

    run_with_timeout("render.sh", cmd, config.timeout)?;
    if !pdf_out.exists() {
        return Err(FpRenderError::MissingOutput {
            command: "render.sh".into(),
            expected: pdf_out.to_path_buf(),
        });
    }
    Ok(())
}

// ============================================================================
// RENDER BACKENDS
// ============================================================================

/// The PDF half of the pipeline: turn a filled HTML document into a PDF file.
///
/// Deliberately narrow. Everything else — the fill, the gates, the temporary
/// directory, the release policy — is backend independent and stays where it
/// is, so swapping the browser out cannot change what a proposal contains.
pub trait ProposalPdfRenderer: Send + Sync {
    fn render(&self, html_in: &Path, pdf_out: &Path) -> Result<(), FpRenderError>;
    /// Short name for logs and the `x-fp-render-backend` response header.
    fn backend(&self) -> &'static str;
}

/// The original backend: fp-template's own `render.sh`, driving a local
/// headless Chrome. Kept for development and for hosts that have a browser.
#[derive(Debug, Clone)]
pub struct LocalChromiumRenderer {
    config: FpRenderConfig,
}

impl LocalChromiumRenderer {
    pub fn new(config: FpRenderConfig) -> Self {
        Self { config }
    }
}

impl ProposalPdfRenderer for LocalChromiumRenderer {
    fn render(&self, html_in: &Path, pdf_out: &Path) -> Result<(), FpRenderError> {
        render_pdf(&self.config, html_in, pdf_out)
    }

    fn backend(&self) -> &'static str {
        "local-chromium"
    }
}

impl ProposalPdfRenderer for GotenbergRenderer {
    fn render(&self, html_in: &Path, pdf_out: &Path) -> Result<(), FpRenderError> {
        self.render_to_file(html_in, pdf_out)
            .map_err(FpRenderError::Gotenberg)
    }

    fn backend(&self) -> &'static str {
        "gotenberg"
    }
}

/// Choose the render backend from the environment: gotenberg when
/// `GOTENBERG_URL` names one, the local browser otherwise.
///
/// There is no host default for `GOTENBERG_URL` on purpose — an unset value
/// means "this deployment renders locally", never "try some address".
pub fn select_renderer(config: &FpRenderConfig) -> Box<dyn ProposalPdfRenderer> {
    match GotenbergConfig::from_env() {
        // The fp-template checkout is the second allowed asset root: a filled
        // proposal legitimately points at its stylesheets, fonts and images.
        // Anything resolving outside it and the document's own directory is
        // refused rather than uploaded.
        Some(gotenberg) => {
            Box::new(GotenbergRenderer::new(gotenberg).with_roots(vec![config.root.clone()]))
        }
        None => Box::new(LocalChromiumRenderer::new(config.clone())),
    }
}

// ============================================================================
// REMOTE PROPOSAL BUILD (docbuilder) — a second, independent seam
// ============================================================================

/// Builds a full, gated proposal PDF directly from manifest markdown.
///
/// Unlike [`ProposalPdfRenderer`] (HTML in, PDF out — fill and gate stay in
/// THIS process), a [`ProposalBuilder`] owns fill, gate, and render together.
/// [`DocbuilderRenderer`] is the only implementation: the fp-docbuilder
/// service does all three steps remotely, so a caller using it needs no
/// `FP_TEMPLATE_ROOT`, no python3, and no local browser.
pub trait ProposalBuilder: Send + Sync {
    /// `vars_filename` MUST match the manifest frontmatter's `vars_source`
    /// exactly — the only current implementation ([`DocbuilderRenderer`])
    /// sends it as a second multipart part under that name.
    fn build(
        &self,
        manifest_markdown: &str,
        vars_json: &str,
        vars_filename: &str,
        output_filename: Option<&str>,
    ) -> Result<ProposalBuildOutput, ProposalBuildError>;
    /// Short name for logs and the `x-fp-render-backend` response header.
    fn backend(&self) -> &'static str;
}

/// What a successful remote proposal build produced.
#[derive(Debug, Clone)]
pub struct ProposalBuildOutput {
    pub pdf: Vec<u8>,
    pub backend: &'static str,
    /// Advisory issue-check hit count, mirroring [`GateReport::issue_hits`] —
    /// 0 when the gate ran clean or the backend reports none.
    pub issue_hits: usize,
}

/// Why a remote proposal build failed, deliberately as narrow as
/// [`GatedRenderError`]'s two cases so a caller maps both the same way.
#[derive(Debug)]
pub enum ProposalBuildError {
    /// The document must not be issued. Carries the service's own report.
    Blocked(String),
    /// The service is unavailable, misconfigured, or failed to render.
    Unavailable(String),
}

impl std::fmt::Display for ProposalBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalBuildError::Blocked(msg) => write!(f, "{msg}"),
            ProposalBuildError::Unavailable(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ProposalBuildError {}

/// Renders a proposal end to end via a remote fp-docbuilder service.
#[derive(Debug, Clone)]
pub struct DocbuilderRenderer {
    client: DocbuilderClient,
}

impl DocbuilderRenderer {
    pub fn new(config: DocbuilderConfig) -> Self {
        Self {
            client: DocbuilderClient::new(config),
        }
    }
}

impl ProposalBuilder for DocbuilderRenderer {
    fn build(
        &self,
        manifest_markdown: &str,
        vars_json: &str,
        vars_filename: &str,
        output_filename: Option<&str>,
    ) -> Result<ProposalBuildOutput, ProposalBuildError> {
        match self.client.render_proposal(
            manifest_markdown,
            vars_json,
            vars_filename,
            output_filename,
            Some(self.client.config().tagged),
        ) {
            Ok(out) => {
                let issue_hits = out.issue_hits();
                Ok(ProposalBuildOutput {
                    pdf: out.pdf,
                    backend: "docbuilder",
                    issue_hits,
                })
            }
            Err(e) if e.is_caller_fault() => Err(ProposalBuildError::Blocked(e.to_string())),
            Err(e) => Err(ProposalBuildError::Unavailable(e.to_string())),
        }
    }

    fn backend(&self) -> &'static str {
        "docbuilder"
    }
}

/// Either backend a caller may build a proposal through, chosen once by
/// [`select_proposal_backend`].
pub enum ProposalBackend {
    /// The remote fp-docbuilder service — production. Owns fill, gate, and
    /// render; the caller supplies only manifest markdown.
    Docbuilder(DocbuilderRenderer),
    /// The local fill+gate+render pipeline in this module — development-only,
    /// or any host that has `FP_TEMPLATE_ROOT` and (for `LocalChromiumRenderer`)
    /// a browser.
    Local(FpRenderConfig),
}

/// Choose how a caller should build a proposal PDF: the remote docbuilder
/// service when `DOCBUILDER_URL` is set, else the local pipeline when
/// `FP_TEMPLATE_ROOT` is set, else `None` ("not configured").
///
/// `DOCBUILDER_URL` takes priority: it names the production path, and when it
/// is set the caller need not have `FP_TEMPLATE_ROOT` at all (see
/// `e-fees-api`'s Dockerfile and `docs/development/FP-TEMPLATE-INTEGRATION.md`).
pub fn select_proposal_backend() -> Option<ProposalBackend> {
    if let Some(config) = DocbuilderConfig::from_env() {
        return Some(ProposalBackend::Docbuilder(DocbuilderRenderer::new(config)));
    }
    FpRenderConfig::from_env().map(ProposalBackend::Local)
}

/// One of fp-template's read-only pre-issue gates.
#[derive(Debug, Clone, Copy)]
pub enum FpGate {
    /// Red-swatch markers, literal `XXX`, and `[bracketed]` placeholders.
    Issue,
    /// Every `.starts-page`/`.starts-column` element is top-of-container.
    Column,
    /// No ASCII straight quotes in document text.
    Quote,
}

impl FpGate {
    fn flag(self) -> &'static str {
        match self {
            FpGate::Issue => "--issue-check",
            FpGate::Column => "--column-check",
            FpGate::Quote => "--quote-check",
        }
    }
}

/// Run one pre-issue gate against a rendered document.
///
/// Returns `Ok(stdout)` when the gate passes and the gate's own report as
/// [`FpRenderError::Failed`] when it does not. A failing `--issue-check` on a
/// manifest with unfilled narrative placeholders is the EXPECTED outcome, not a
/// bug: it is the mechanism that stops an incomplete proposal being issued.
pub fn run_gate(
    config: &FpRenderConfig,
    gate: FpGate,
    html_in: &Path,
) -> Result<String, FpRenderError> {
    config.validate()?;
    let mut cmd = Command::new(config.render_sh());
    cmd.arg(gate.flag()).arg(html_in);
    cmd.current_dir(&config.root);
    run_with_timeout(gate.flag(), cmd, config.timeout)
}

// ============================================================================
// RELEASE GATE
// ============================================================================

/// A rendered document that passed the release gate.
#[derive(Debug, Clone, Default)]
pub struct GateReport {
    /// `--issue-check`'s full report, kept whether it passed or not. It is
    /// advisory, not a verdict: see [`classify_gates`].
    pub issue_report: String,
    /// Hit count parsed from the issue-check report, 0 when it passed.
    pub issue_hits: usize,
}

/// Why a rendered document must not be handed to a caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateBlock {
    /// `--column-check` or `--quote-check` failed. These are unambiguous
    /// mechanical faults in the rendered geometry or typography.
    Mechanical { gate: String, output: String },
    /// The document still carries this exporter's own
    /// [`crate::export::fp_manifest::INCOMPLETE_MARKER`] sentinel, so a section
    /// that must be authored per project was never filled in.
    Incomplete { report: String },
}

impl std::fmt::Display for GateBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateBlock::Mechanical { gate, output } => write!(
                f,
                "fp-template {gate} failed on the rendered document:\n{}",
                output.trim()
            ),
            GateBlock::Incomplete { report } => write!(
                f,
                "the rendered proposal still contains unfilled placeholders and must not be \
                 issued. fp-template --issue-check reports:\n{}",
                report.trim()
            ),
        }
    }
}

impl std::error::Error for GateBlock {}

/// Parse the hit count from an issue-check report line like
/// `  24 hits (4 red-text, 4 XXX, 16 bracketed) across 8 page(s):`.
fn parse_issue_hits(report: &str) -> usize {
    for line in report.lines() {
        let t = line.trim();
        if let Some(rest) = t.split_whitespace().next() {
            if t.contains(" hits") || t.contains(" hit ") {
                if let Ok(n) = rest.parse::<usize>() {
                    return n;
                }
            }
        }
    }
    0
}

/// Decide whether a rendered document may be returned to a caller.
///
/// Pure over the three gates' already-collected outputs, so the policy is
/// testable without a Chromium render.
///
/// THE POLICY, and why it is not "any non-zero exit blocks":
///
/// - `--column-check` and `--quote-check` are unambiguous. A failure is a real
///   fault in the rendered document. They block.
/// - `--issue-check` is NOT a pass/fail verdict on a filled proposal, and
///   treating it as one would reject every proposal we ever render. Its own
///   documentation in `render.sh` says so: "A filled proposal's bracketed-token
///   count is NOT automatically expected to be zero either - the master's own
///   permanent legal boilerplate legitimately uses bracket-qualifier drafting
///   convention that a proposal correctly inherits verbatim; judge each hit."
///   Measured on a real render: 24 hits, of which 22 belong to the template's
///   own boilerplate (`[Client]`, `[60]`, `[RIBA]`, its `xxx` markers) and only
///   2 were ours. So the block condition is the sentinel THIS EXPORTER writes
///   and the template never does, found in the issue-check report. The full
///   report travels with the decision either way, so a human can judge the rest.
pub fn classify_gates(
    column: Result<String, FpRenderError>,
    quote: Result<String, FpRenderError>,
    issue: Result<String, FpRenderError>,
    incomplete_marker: &str,
) -> Result<GateReport, GateBlock> {
    for (name, result) in [("--column-check", column), ("--quote-check", quote)] {
        if let Err(e) = result {
            let output = match &e {
                FpRenderError::Failed { stdout, stderr, .. } => {
                    let combined = format!("{}\n{}", stdout.trim(), stderr.trim());
                    combined.trim().to_string()
                }
                other => other.to_string(),
            };
            return Err(GateBlock::Mechanical {
                gate: name.to_string(),
                output,
            });
        }
    }

    // issue-check reports on stdout and exits 1 whenever it finds ANY hit, so
    // the report is read in both branches - the exit code is not the signal.
    let report = match issue {
        Ok(stdout) => stdout,
        Err(FpRenderError::Failed { stdout, stderr, .. }) => {
            format!("{}\n{}", stdout.trim(), stderr.trim())
                .trim()
                .to_string()
        }
        Err(other) => {
            return Err(GateBlock::Mechanical {
                gate: "--issue-check".to_string(),
                output: other.to_string(),
            })
        }
    };

    if report.contains(incomplete_marker) {
        return Err(GateBlock::Incomplete { report });
    }

    Ok(GateReport {
        issue_hits: parse_issue_hits(&report),
        issue_report: report,
    })
}

/// Run all three pre-issue gates against a rendered document and apply
/// [`classify_gates`].
///
/// Every path that hands a rendered proposal to a client must go through this.
pub fn run_release_gates(config: &FpRenderConfig, html_in: &Path) -> Result<GateReport, GateBlock> {
    let column = run_gate(config, FpGate::Column, html_in);
    let quote = run_gate(config, FpGate::Quote, html_in);
    let issue = run_gate(config, FpGate::Issue, html_in);
    classify_gates(
        column,
        quote,
        issue,
        crate::export::fp_manifest::INCOMPLETE_MARKER,
    )
}

/// Fill then render, in one call.
///
/// NOTE: this does NOT run the release gate. Callers that hand the PDF to a
/// client must call [`run_release_gates`] on the returned `html_path` and honour
/// its verdict; [`build_proposal_pdf_gated`] does both.
pub fn build_proposal_pdf(
    config: &FpRenderConfig,
    manifest_path: &Path,
    html_out: &Path,
    pdf_out: &Path,
) -> Result<FpRenderOutput, FpRenderError> {
    build_proposal_pdf_with(
        config,
        select_renderer(config).as_ref(),
        manifest_path,
        html_out,
        pdf_out,
    )
}

/// Fill then render with an explicitly chosen backend.
pub fn build_proposal_pdf_with(
    config: &FpRenderConfig,
    renderer: &dyn ProposalPdfRenderer,
    manifest_path: &Path,
    html_out: &Path,
    pdf_out: &Path,
) -> Result<FpRenderOutput, FpRenderError> {
    let fill_stdout = fill_manifest(config, manifest_path, html_out)?;
    renderer.render(html_out, pdf_out)?;
    if !pdf_out.exists() {
        return Err(FpRenderError::MissingOutput {
            command: renderer.backend().to_string(),
            expected: pdf_out.to_path_buf(),
        });
    }
    Ok(FpRenderOutput {
        html_path: html_out.to_path_buf(),
        pdf_path: pdf_out.to_path_buf(),
        fill_stdout,
    })
}

/// Everything a gated build produced.
#[derive(Debug)]
pub struct GatedRender {
    pub output: FpRenderOutput,
    pub gates: GateReport,
    /// Which backend produced the PDF, for logs and response headers.
    pub backend: &'static str,
}

/// Why a gated build stopped.
#[derive(Debug)]
pub enum GatedRenderError {
    /// The fill or the render itself failed.
    Render(FpRenderError),
    /// The document rendered but must not be issued.
    Gate(GateBlock),
}

impl std::fmt::Display for GatedRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GatedRenderError::Render(e) => write!(f, "{e}"),
            GatedRenderError::Gate(b) => write!(f, "{b}"),
        }
    }
}

impl std::error::Error for GatedRenderError {}

/// Fill, gate, then render — the only entry point a client-facing caller
/// should use. A PDF comes back only when the release gate passed, and the
/// backend is whatever [`select_renderer`] chose.
pub fn build_proposal_pdf_gated(
    config: &FpRenderConfig,
    manifest_path: &Path,
    html_out: &Path,
    pdf_out: &Path,
) -> Result<GatedRender, GatedRenderError> {
    build_proposal_pdf_gated_with(
        config,
        select_renderer(config).as_ref(),
        manifest_path,
        html_out,
        pdf_out,
    )
}

/// Fill, gate, and render with an explicitly chosen backend.
///
/// ORDER MATTERS AND IS DELIBERATE: the gates run on the filled HTML BEFORE
/// the PDF is produced, so a document that must not be issued never reaches a
/// renderer at all — and, with gotenberg, never leaves this host.
pub fn build_proposal_pdf_gated_with(
    config: &FpRenderConfig,
    renderer: &dyn ProposalPdfRenderer,
    manifest_path: &Path,
    html_out: &Path,
    pdf_out: &Path,
) -> Result<GatedRender, GatedRenderError> {
    let fill_stdout =
        fill_manifest(config, manifest_path, html_out).map_err(GatedRenderError::Render)?;
    let gates = run_release_gates(config, html_out).map_err(GatedRenderError::Gate)?;
    renderer
        .render(html_out, pdf_out)
        .map_err(GatedRenderError::Render)?;
    if !pdf_out.exists() {
        return Err(GatedRenderError::Render(FpRenderError::MissingOutput {
            command: renderer.backend().to_string(),
            expected: pdf_out.to_path_buf(),
        }));
    }
    Ok(GatedRender {
        output: FpRenderOutput {
            html_path: html_out.to_path_buf(),
            pdf_path: pdf_out.to_path_buf(),
            fill_stdout,
        },
        gates,
        backend: renderer.backend(),
    })
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// A minimal one-shot HTTP server for exercising [`DocbuilderRenderer`]
    /// without a live service: binds an ephemeral port, accepts one
    /// connection, and replies with a fixed response.
    fn one_shot_server(response: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 8192];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(&response);
                let _ = stream.flush();
            }
        });
        format!("http://{addr}")
    }

    fn json_response(status_line: &str, body: &[u8]) -> Vec<u8> {
        let mut resp = format!(
            "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        resp.extend_from_slice(body);
        resp
    }

    #[test]
    fn docbuilder_renderer_maps_a_blocked_stage_to_proposal_build_error_blocked() {
        let addr = one_shot_server(json_response(
            "HTTP/1.1 422 Unprocessable Entity",
            br#"{"stage":"gate","blocked":true,"report":["quote-check failed"]}"#,
        ));
        let renderer = DocbuilderRenderer::new(DocbuilderConfig::new(addr));
        let err = renderer
            .build("# manifest", "{}", "vars.json", None)
            .unwrap_err();
        match err {
            ProposalBuildError::Blocked(msg) => {
                assert!(msg.contains("gate"), "{msg}");
                assert!(msg.contains("quote-check failed"), "{msg}");
            }
            other => panic!("expected Blocked, got {other}"),
        }
    }

    #[test]
    fn docbuilder_renderer_maps_a_render_failure_to_proposal_build_error_unavailable() {
        let addr = one_shot_server(json_response(
            "HTTP/1.1 503 Service Unavailable",
            br#"{"stage":"render","error":"chromium crashed"}"#,
        ));
        let renderer = DocbuilderRenderer::new(DocbuilderConfig::new(addr));
        let err = renderer
            .build("# manifest", "{}", "vars.json", None)
            .unwrap_err();
        match err {
            ProposalBuildError::Unavailable(msg) => assert!(msg.contains("chromium crashed")),
            other => panic!("expected Unavailable, got {other}"),
        }
    }

    #[test]
    fn docbuilder_renderer_build_propagates_pdf_backend_and_issue_hits() {
        let mut resp = b"HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\n\
            X-Docbuilder-Gates: advisory:2\r\nContent-Length: 9\r\n\r\n"
            .to_vec();
        resp.extend_from_slice(b"%PDF-fake");
        let addr = one_shot_server(resp);
        let renderer = DocbuilderRenderer::new(DocbuilderConfig::new(addr));
        let output = renderer
            .build("# manifest", "{}", "vars.json", Some("job-FP.pdf"))
            .unwrap();
        assert_eq!(output.pdf, b"%PDF-fake");
        assert_eq!(output.backend, "docbuilder");
        assert_eq!(output.issue_hits, 2);
        assert_eq!(renderer.backend(), "docbuilder");
    }

    #[test]
    fn missing_root_is_reported_not_guessed() {
        let config = FpRenderConfig::new("/nonexistent/fp-template");
        match config.validate() {
            Err(FpRenderError::TemplateRootInvalid { missing, .. }) => {
                assert!(missing.ends_with("fill.py"));
            }
            other => panic!("expected TemplateRootInvalid, got {other:?}"),
        }
    }

    #[test]
    fn fill_refuses_an_invalid_root_before_spawning_anything() {
        let config = FpRenderConfig::new("/nonexistent/fp-template");
        let err = fill_manifest(
            &config,
            Path::new("/tmp/manifest.md"),
            Path::new("/tmp/out.html"),
        )
        .unwrap_err();
        assert!(matches!(err, FpRenderError::TemplateRootInvalid { .. }));
    }

    #[test]
    fn timeout_kills_a_hung_command() {
        let mut cmd = Command::new("sleep");
        cmd.arg("30");
        let err = run_with_timeout("sleep", cmd, Duration::from_millis(200)).unwrap_err();
        match err {
            FpRenderError::TimedOut { timeout, .. } => {
                assert_eq!(timeout, Duration::from_millis(200))
            }
            other => panic!("expected TimedOut, got {other:?}"),
        }
    }

    #[test]
    fn nonzero_exit_carries_stderr_back_to_the_caller() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("echo boom >&2; exit 3");
        let err = run_with_timeout("sh", cmd, Duration::from_secs(5)).unwrap_err();
        match err {
            FpRenderError::Failed { code, stderr, .. } => {
                assert_eq!(code, Some(3));
                assert!(stderr.contains("boom"));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn successful_command_returns_stdout() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("echo hello");
        let out = run_with_timeout("sh", cmd, Duration::from_secs(5)).unwrap();
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn unset_env_yields_no_config() {
        // Not asserting on the live process env (other tests may set it);
        // asserting the filter rejects an empty value, which is the case that
        // would otherwise produce a config pointing at "".
        assert!(Some("".to_string())
            .filter(|v: &String| !v.trim().is_empty())
            .is_none());
    }

    // ---- release-gate policy ----

    const MARKER: &str = "TO BE COMPLETED";

    /// A real `--issue-check` report from a rendered proposal, abbreviated. Every
    /// one of these hits belongs to the TEMPLATE's own boilerplate - this is the
    /// shape a COMPLETE proposal produces, and it must not block.
    const TEMPLATE_ONLY_REPORT: &str = "issue-check: /tmp/proposal.html\n  \
        22 hits (4 red-text, 4 XXX, 14 bracketed) across 8 page(s):\n    \
        [PLACEHOLDER] page 5   line 280: [\u{201c}Consultant\u{201d}, \u{201c}We\u{201d}]\n    \
        [PLACEHOLDER] page 5   line 287: [60]\n    \
        [RED-TEXT   ] page 6   line 332: xxx\n    \
        [PLACEHOLDER] page 9   line 440: [RIBA]\n";

    /// The same report from an UNFINISHED proposal - two of our own sentinels.
    const INCOMPLETE_REPORT: &str = "issue-check: /tmp/proposal.html\n  \
        24 hits (4 red-text, 4 XXX, 16 bracketed) across 8 page(s):\n    \
        [PLACEHOLDER] page 5   line 287: [60]\n    \
        [PLACEHOLDER] page 5   line 290: [TO BE COMPLETED: list the documents this proposal responds to]\n    \
        [PLACEHOLDER] page 5   line 297: [TO BE COMPLETED: state the areas included in and excluded from the scope]\n";

    fn failed(command: &str, stdout: &str) -> FpRenderError {
        FpRenderError::Failed {
            command: command.to_string(),
            code: Some(1),
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    #[test]
    fn issue_check_hits_from_the_template_alone_do_not_block() {
        // The decisive case: issue-check exits 1 on ANY hit, and a complete
        // proposal always has hits from the template's own boilerplate. Treating
        // its exit code as a verdict would reject every proposal.
        let report = classify_gates(
            Ok(String::new()),
            Ok(String::new()),
            Err(failed("--issue-check", TEMPLATE_ONLY_REPORT)),
            MARKER,
        )
        .expect("template-only hits must not block");
        assert_eq!(report.issue_hits, 22);
        assert!(report.issue_report.contains("[RIBA]"));
    }

    #[test]
    fn our_own_unfilled_placeholder_blocks_with_the_report() {
        let block = classify_gates(
            Ok(String::new()),
            Ok(String::new()),
            Err(failed("--issue-check", INCOMPLETE_REPORT)),
            MARKER,
        )
        .unwrap_err();
        match &block {
            GateBlock::Incomplete { report } => {
                assert!(report.contains("list the documents"));
                assert!(report.contains("state the areas"));
            }
            other => panic!("expected Incomplete, got {other:?}"),
        }
        // The caller-facing message must carry the reason and the evidence.
        let msg = block.to_string();
        assert!(msg.contains("must not be issued"), "{msg}");
        assert!(msg.contains("TO BE COMPLETED"), "{msg}");
    }

    #[test]
    fn a_clean_issue_check_passes_with_zero_hits() {
        let report = classify_gates(
            Ok(String::new()),
            Ok(String::new()),
            Ok("issue-check: /tmp/proposal.html\n  clean\n".to_string()),
            MARKER,
        )
        .unwrap();
        assert_eq!(report.issue_hits, 0);
    }

    #[test]
    fn column_check_failure_blocks_mechanically() {
        let block = classify_gates(
            Err(failed("--column-check", "1 misplaced element on page 4")),
            Ok(String::new()),
            Ok(String::new()),
            MARKER,
        )
        .unwrap_err();
        match &block {
            GateBlock::Mechanical { gate, output } => {
                assert_eq!(gate, "--column-check");
                assert!(output.contains("misplaced element"));
            }
            other => panic!("expected Mechanical, got {other:?}"),
        }
        assert!(block.to_string().contains("--column-check failed"));
    }

    #[test]
    fn quote_check_failure_blocks_mechanically() {
        let block = classify_gates(
            Ok(String::new()),
            Err(failed("--quote-check", "straight apostrophe on page 7")),
            Ok(String::new()),
            MARKER,
        )
        .unwrap_err();
        assert!(matches!(
            block,
            GateBlock::Mechanical { ref gate, .. } if gate == "--quote-check"
        ));
    }

    #[test]
    fn a_mechanical_gate_failure_wins_over_a_clean_issue_check() {
        let block = classify_gates(
            Err(failed("--column-check", "bad")),
            Ok(String::new()),
            Ok(String::new()),
            MARKER,
        )
        .unwrap_err();
        assert!(matches!(block, GateBlock::Mechanical { .. }));
    }

    #[test]
    fn an_unrunnable_issue_check_blocks_rather_than_passing_silently() {
        // A gate we could not run is never treated as a pass.
        let block = classify_gates(
            Ok(String::new()),
            Ok(String::new()),
            Err(FpRenderError::TimedOut {
                command: "--issue-check".into(),
                timeout: Duration::from_secs(1),
            }),
            MARKER,
        )
        .unwrap_err();
        assert!(matches!(block, GateBlock::Mechanical { .. }));
    }

    #[test]
    fn issue_hit_count_parsing() {
        assert_eq!(parse_issue_hits(TEMPLATE_ONLY_REPORT), 22);
        assert_eq!(parse_issue_hits(INCOMPLETE_REPORT), 24);
        assert_eq!(parse_issue_hits("no counts here"), 0);
        assert_eq!(parse_issue_hits(""), 0);
    }

    #[test]
    fn gate_flags_match_render_sh() {
        assert_eq!(FpGate::Issue.flag(), "--issue-check");
        assert_eq!(FpGate::Column.flag(), "--column-check");
        assert_eq!(FpGate::Quote.flag(), "--quote-check");
    }

    // ---- select_proposal_backend: env precedence ----

    /// Guards the three env-mutating assertions below. No other test in this
    /// crate reads or writes `DOCBUILDER_URL`/`FP_TEMPLATE_ROOT`, but a mutex
    /// makes that an enforced invariant of THIS test rather than a
    /// coincidence that a future test could silently break under parallel
    /// execution.
    static BACKEND_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn select_proposal_backend_prefers_docbuilder_over_local() {
        use crate::export::docbuilder::DOCBUILDER_URL_ENV;

        let _guard = BACKEND_ENV_LOCK.lock().unwrap();
        // SAFETY: serialised by BACKEND_ENV_LOCK above; no other test touches
        // these two variables.
        unsafe {
            std::env::remove_var(DOCBUILDER_URL_ENV);
            std::env::remove_var(FP_TEMPLATE_ROOT_ENV);
        }
        assert!(
            select_proposal_backend().is_none(),
            "neither env var set must be None (\"not configured\")"
        );

        unsafe {
            std::env::set_var(FP_TEMPLATE_ROOT_ENV, "/tmp/efees-test-fp-template-root");
        }
        assert!(
            matches!(select_proposal_backend(), Some(ProposalBackend::Local(_))),
            "FP_TEMPLATE_ROOT alone must select the local pipeline"
        );

        unsafe {
            std::env::set_var(DOCBUILDER_URL_ENV, "http://127.0.0.1:1");
        }
        assert!(
            matches!(
                select_proposal_backend(),
                Some(ProposalBackend::Docbuilder(_))
            ),
            "DOCBUILDER_URL must take priority when both are set"
        );

        unsafe {
            std::env::remove_var(DOCBUILDER_URL_ENV);
        }
        assert!(
            matches!(select_proposal_backend(), Some(ProposalBackend::Local(_))),
            "clearing DOCBUILDER_URL must fall back to the still-set FP_TEMPLATE_ROOT"
        );

        unsafe {
            std::env::remove_var(FP_TEMPLATE_ROOT_ENV);
        }
    }
}
