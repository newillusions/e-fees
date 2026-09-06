//! Gotenberg render backend — HTML to PDF over HTTP, no local browser.
//!
//! [`GotenbergRenderer`] replaces fp-template's `render.sh` (which drives a
//! local headless Chrome) with a POST to a gotenberg service. Everything else
//! in the pipeline is unchanged: the fill engine still produces the HTML, and
//! the three pre-issue gates still run on that HTML before anything is
//! rendered.
//!
//! # Why the document has to be flattened first
//!
//! Gotenberg stores every uploaded file in a single flat directory and its own
//! documentation is explicit about the consequence: "Reference assets by
//! filename only: no absolute paths (`/img.png`) or subdirectories
//! (`./assets/img.png`)." fp-template's output does the opposite — the fill
//! engine rebases every stylesheet and image against the output file's
//! location, so a filled proposal carries references like
//! `../../tokens.css` and `../../assets/logo white 1024 513.svg`, and
//! `template.css` in turn carries `url("font/Ubuntu-Regular.ttf")`.
//!
//! [`flatten_document`] therefore walks the document, collects every local
//! asset it transitively references (HTML to CSS to fonts and images),
//! rewrites each reference to a bare filename, and returns the rewritten text
//! files alongside the untouched binaries as one [`FlatBundle`] ready to
//! upload.
//!
//! # Containment, and why it is not optional
//!
//! Flattening reads whatever the document references and sends it to a service
//! on the network. Without a boundary, a reference like
//! `../../../../etc/hosts` would be read and uploaded — any file the process
//! can open, leaving the host, driven by whatever influenced the HTML. So
//! every reference must resolve inside an allowed root: the document's own
//! directory (the per-request work directory) plus the fp-template checkout.
//! The check runs on the CANONICALISED path, so a symlink whose target escapes
//! is caught too, and it runs before the file is read.
//!
//! Two smaller guards travel with it. Error text names files relative to a
//! root, or by bare filename, never by absolute path — those strings reach
//! both the API's 422 body and its logs. And [`GotenbergConfig`] caps the
//! bundle size, checked before the request body is built, so an asset tree
//! that has gone wrong costs one comparison rather than a large upload.
//!
//! # Verified against the live service (2026-09-06, gotenberg 8.36.0)
//!
//! - `generateTaggedPdf=true` produces `/Marked true` plus a `StructTreeRoot`,
//!   matching what local Chromium's `--print-to-pdf` emits.
//! - A bundle without an `index.html` member answers `400` with
//!   `Invalid form data: form file 'index.html' is required`, and a body that
//!   does not match its own boundary answers `400 Malformed body: ...`. That
//!   is why 400 maps to the caller's 422 (our request was wrong) while every
//!   other non-2xx maps to 503 (the service is wrong or unavailable).
//! - Unknown form fields are IGNORED, not rejected — a misspelt field name
//!   fails silently and produces a subtly different PDF. Never invent one.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Base URL of the gotenberg service, e.g. `http://10.0.23.31:3000`.
///
/// There is deliberately NO default: an unset value means "no render backend
/// configured", which the caller reports rather than guessing a host.
pub const GOTENBERG_URL_ENV: &str = "GOTENBERG_URL";

/// Per-request wall-clock budget, in seconds.
pub const GOTENBERG_TIMEOUT_ENV: &str = "GOTENBERG_TIMEOUT_SECS";

/// Whether to ask Chromium for a tagged (accessible) PDF. Default on.
pub const GOTENBERG_TAGGED_PDF_ENV: &str = "GOTENBERG_TAGGED_PDF";

/// Upper bound on the flattened bundle, in bytes.
pub const GOTENBERG_MAX_BUNDLE_BYTES_ENV: &str = "GOTENBERG_MAX_BUNDLE_BYTES";

/// Default per-request budget. The measured render of a 19-page proposal with
/// ~650 KB of assets answered in low single-digit seconds; 60s absorbs a cold
/// Chromium start and font loading without hanging a request forever.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Default bundle cap. The measured 19-page proposal with every font embedded
/// is well under 1 MB, so 25 MiB is generous for a real document while still
/// refusing an asset tree that has gone wrong, before anything is read into
/// memory and pushed over the network.
pub const DEFAULT_MAX_BUNDLE_BYTES: usize = 25 * 1024 * 1024;

/// The route this client posts to. Chromium module, HTML entry point.
pub const CONVERT_PATH: &str = "/forms/chromium/convert/html";

/// The filename gotenberg requires as the document entry point.
pub const INDEX_FILENAME: &str = "index.html";

// ============================================================================
// CONFIG
// ============================================================================

/// Where gotenberg lives and how it should render.
#[derive(Debug, Clone)]
pub struct GotenbergConfig {
    /// Base URL, without a trailing slash.
    pub base_url: String,
    /// Wall-clock budget for one conversion request.
    pub timeout: Duration,
    /// Emit a tagged PDF (`generateTaggedPdf`), matching local Chromium.
    pub tagged_pdf: bool,
    /// Refuse to upload a bundle larger than this many bytes.
    pub max_bundle_bytes: usize,
}

impl GotenbergConfig {
    /// Build a config for an explicit base URL, with the defaults applied.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout: DEFAULT_TIMEOUT,
            tagged_pdf: true,
            max_bundle_bytes: DEFAULT_MAX_BUNDLE_BYTES,
        }
    }

    /// Read the gotenberg settings from the environment.
    ///
    /// Returns `None` when [`GOTENBERG_URL_ENV`] is unset or blank — the
    /// signal that this deployment renders locally instead.
    pub fn from_env() -> Option<Self> {
        let url = std::env::var(GOTENBERG_URL_ENV)
            .ok()
            .filter(|v| !v.trim().is_empty())?;
        let mut config = Self::new(url.trim());
        if let Some(secs) = std::env::var(GOTENBERG_TIMEOUT_ENV)
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|s| *s > 0)
        {
            config.timeout = Duration::from_secs(secs);
        }
        if let Ok(raw) = std::env::var(GOTENBERG_TAGGED_PDF_ENV) {
            config.tagged_pdf = parse_bool(&raw).unwrap_or(config.tagged_pdf);
        }
        if let Some(cap) = std::env::var(GOTENBERG_MAX_BUNDLE_BYTES_ENV)
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|c| *c > 0)
        {
            config.max_bundle_bytes = cap;
        }
        Some(config)
    }

    /// Full URL of the HTML conversion route.
    pub fn convert_url(&self) -> String {
        format!("{}{}", self.base_url, CONVERT_PATH)
    }

    /// The form fields sent with every conversion.
    ///
    /// `preferCssPageSize` + zero margins hand page geometry entirely to
    /// template.css's `@page` rule (1920pt x 1080pt), and `printBackground`
    /// keeps the template's black ground. These three are what made the
    /// measured render match local Chromium to antialiasing noise; changing
    /// any of them changes the output.
    pub fn form_fields(&self) -> Vec<(String, String)> {
        let mut fields = vec![
            ("preferCssPageSize".to_string(), "true".to_string()),
            ("printBackground".to_string(), "true".to_string()),
            ("marginTop".to_string(), "0".to_string()),
            ("marginBottom".to_string(), "0".to_string()),
            ("marginLeft".to_string(), "0".to_string()),
            ("marginRight".to_string(), "0".to_string()),
        ];
        if self.tagged_pdf {
            fields.push(("generateTaggedPdf".to_string(), "true".to_string()));
        }
        fields
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

/// Why a gotenberg render did not produce a PDF.
///
/// The split matters to the caller: [`GotenbergError::BadRequest`] and the two
/// bundle failures are OUR fault and map to 422, while transport failures and
/// any other non-2xx are the service's and map to 503.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GotenbergError {
    /// A reference in the document names a file that is not on disk.
    AssetMissing { referrer: String, reference: String },
    /// A reference resolves OUTSIDE every allowed root. Either a traversal
    /// (`../../../../etc/hostname`) or a symlink pointing out of the tree.
    /// Refused before the file is read, so it never enters a bundle.
    EscapesRoot { referrer: String, reference: String },
    /// Two different files would land on the same flat filename. Gotenberg has
    /// no subdirectories, so one would silently overwrite the other.
    DuplicateBasename {
        filename: String,
        first: String,
        second: String,
    },
    /// The bundle exceeds the configured upload cap.
    BundleTooLarge { bytes: usize, limit: usize },
    /// The document or one of its assets could not be read.
    Io { path: String, message: String },
    /// The request never got an answer (connection refused, DNS, timeout).
    Transport { url: String, message: String },
    /// Gotenberg rejected the request itself — a malformed bundle. Our bug.
    BadRequest { status: u16, body: String },
    /// Any other non-2xx answer: the service is unhealthy or unavailable.
    Status { status: u16, body: String },
    /// A 2xx answer that was not a PDF.
    NotAPdf { bytes: usize },
}

impl fmt::Display for GotenbergError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GotenbergError::AssetMissing {
                referrer,
                reference,
            } => write!(
                f,
                "{referrer} references {reference}, which does not exist on disk"
            ),
            GotenbergError::EscapesRoot {
                referrer,
                reference,
            } => write!(
                f,
                "{referrer} references {reference}, which resolves outside the \
                 template and working directories; refused"
            ),
            GotenbergError::DuplicateBasename {
                filename,
                first,
                second,
            } => write!(
                f,
                "two different assets would both upload as {filename} ({first} and \
                 {second}); gotenberg has no subdirectories, so one would overwrite \
                 the other"
            ),
            GotenbergError::BundleTooLarge { bytes, limit } => write!(
                f,
                "the flattened document is {bytes} bytes, over the {limit}-byte \
                 upload cap; nothing was sent"
            ),
            GotenbergError::Io { path, message } => {
                write!(f, "could not read {path}: {message}")
            }
            GotenbergError::Transport { url, message } => {
                write!(f, "gotenberg at {url} did not answer: {message}")
            }
            GotenbergError::BadRequest { status, body } => write!(
                f,
                "gotenberg rejected the render request ({status}): {}",
                body.trim()
            ),
            GotenbergError::Status { status, body } => {
                write!(f, "gotenberg failed to render ({status}): {}", body.trim())
            }
            GotenbergError::NotAPdf { bytes } => write!(
                f,
                "gotenberg answered 200 with {bytes} bytes that are not a PDF"
            ),
        }
    }
}

impl std::error::Error for GotenbergError {}

impl GotenbergError {
    /// True when the failure is this side's (a malformed bundle), so the
    /// caller answers 422 rather than 503.
    pub fn is_caller_fault(&self) -> bool {
        matches!(
            self,
            GotenbergError::AssetMissing { .. }
                | GotenbergError::EscapesRoot { .. }
                | GotenbergError::DuplicateBasename { .. }
                | GotenbergError::BundleTooLarge { .. }
                | GotenbergError::BadRequest { .. }
        )
    }
}

// ============================================================================
// FLATTENING
// ============================================================================

/// One member of the flat upload directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledFile {
    /// The flat name gotenberg will store it under.
    pub filename: String,
    /// File content — rewritten for HTML/CSS, verbatim for everything else.
    pub bytes: Vec<u8>,
}

/// A document plus every local asset it transitively references, with all
/// references rewritten to flat filenames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatBundle {
    /// `files[0]` is always `index.html`; the rest are sorted by filename so
    /// the request body is byte-stable for a given document.
    pub files: Vec<BundledFile>,
}

impl FlatBundle {
    /// Names of every member, in upload order.
    pub fn filenames(&self) -> Vec<&str> {
        self.files.iter().map(|f| f.filename.as_str()).collect()
    }

    /// Total size of every member, the figure checked against the upload cap.
    pub fn total_bytes(&self) -> usize {
        self.files.iter().map(|f| f.bytes.len()).sum()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextKind {
    Html,
    Css,
}

fn text_kind_for(path: &Path) -> Option<TextKind> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("html") | Some("htm") => Some(TextKind::Html),
        Some("css") => Some(TextKind::Css),
        _ => None,
    }
}

/// Collect `html_path` and every local asset it references into a flat bundle.
///
/// The entry document is always renamed to `index.html` (gotenberg requires
/// that name); every other file keeps its own basename.
///
/// CONTAINMENT (security, not tidiness): every reference must resolve inside
/// an allowed root - the entry document's own directory (the per-request work
/// directory) plus whatever `extra_roots` names (in practice the fp-template
/// checkout). Paths are canonicalised BEFORE the check, so a traversal like
/// `../../../../etc/hostname` and a symlink pointing out of the tree are both
/// caught. Without this, any reference the document can express would be read
/// from disk and uploaded to the render service.
///
/// Fails rather than guessing when a reference points at a file that is not
/// there, escapes the roots, or when two different files would collide on one
/// flat name.
pub fn flatten_document(
    html_path: &Path,
    extra_roots: &[PathBuf],
) -> Result<FlatBundle, GotenbergError> {
    let entry = canonical(html_path)?;

    // The entry's own directory is always a root; a missing or unreadable
    // extra root is dropped rather than failing the render, since the entry
    // root alone is still a valid containment boundary.
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(parent) = entry.parent() {
        roots.push(parent.to_path_buf());
    }
    for extra in extra_roots {
        if let Ok(canonical_root) = extra.canonicalize() {
            if !roots.contains(&canonical_root) {
                roots.push(canonical_root);
            }
        }
    }

    // flat filename -> the one canonical source path allowed to claim it.
    let mut claimed: BTreeMap<String, PathBuf> = BTreeMap::new();
    claimed.insert(INDEX_FILENAME.to_string(), entry.clone());

    // canonical path -> flat filename, for files already queued or emitted.
    let mut assigned: BTreeMap<PathBuf, String> = BTreeMap::new();
    assigned.insert(entry.clone(), INDEX_FILENAME.to_string());

    let mut queue: Vec<(PathBuf, String)> = vec![(entry, INDEX_FILENAME.to_string())];
    let mut out: Vec<BundledFile> = Vec::new();

    while let Some((path, filename)) = queue.pop() {
        let bytes = std::fs::read(&path).map_err(|e| GotenbergError::Io {
            path: redact(&path, &roots),
            message: e.to_string(),
        })?;

        match text_kind_for(&path) {
            Some(kind) => {
                let text = String::from_utf8_lossy(&bytes).into_owned();
                let dir = path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| PathBuf::from("."));
                let mut discovered: Vec<(PathBuf, String)> = Vec::new();
                let rewritten = rewrite_references(&text, kind, |reference| {
                    let resolved = resolve_reference(&dir, reference, &roots)?;
                    let (flat, is_new) =
                        claim_name(&resolved, &mut claimed, &mut assigned, reference, &roots)?;
                    if is_new {
                        discovered.push((resolved, flat.clone()));
                    }
                    Ok(flat)
                })
                .map_err(|e| annotate(e, &path, &roots))?;
                queue.extend(discovered);
                out.push(BundledFile {
                    filename,
                    bytes: rewritten.into_bytes(),
                });
            }
            None => out.push(BundledFile { filename, bytes }),
        }
    }

    // index.html first, then the rest in a stable order.
    out.sort_by(|a, b| {
        let rank = |f: &BundledFile| u8::from(f.filename != INDEX_FILENAME);
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.filename.cmp(&b.filename))
    });
    Ok(FlatBundle { files: out })
}

/// Render a path for a caller-facing message without exposing the container's
/// absolute layout: relative to whichever allowed root contains it, else the
/// bare file name. The 422 body and the warn! line both go through this.
fn redact(path: &Path, roots: &[PathBuf]) -> String {
    for root in roots {
        if let Ok(relative) = path.strip_prefix(root) {
            let shown = relative.display().to_string();
            if !shown.is_empty() {
                return shown;
            }
        }
    }
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "<path>".to_string())
}

/// Name the referring FILE (not its absolute path) on the errors that carry a
/// reference, so the caller can find the offending link without learning the
/// container's directory layout.
fn annotate(e: GotenbergError, referrer: &Path, roots: &[PathBuf]) -> GotenbergError {
    match e {
        GotenbergError::AssetMissing { reference, .. } => GotenbergError::AssetMissing {
            referrer: redact(referrer, roots),
            reference,
        },
        GotenbergError::EscapesRoot { reference, .. } => GotenbergError::EscapesRoot {
            referrer: redact(referrer, roots),
            reference,
        },
        other => other,
    }
}

fn canonical(path: &Path) -> Result<PathBuf, GotenbergError> {
    path.canonicalize().map_err(|e| GotenbergError::Io {
        path: redact(path, &[]),
        message: e.to_string(),
    })
}

/// Resolve one reference against its referrer's directory, then require the
/// result to sit inside an allowed root. Absolute and remote references never
/// reach here.
///
/// The containment check runs on the CANONICALISED path, so it also rejects a
/// symlink whose target lies outside the roots. `referrer` is filled in by
/// [`annotate`] once the caller knows which file the reference came from.
fn resolve_reference(
    dir: &Path,
    reference: &str,
    roots: &[PathBuf],
) -> Result<PathBuf, GotenbergError> {
    let decoded = percent_decode(reference);
    let candidate = dir.join(&decoded);
    let resolved = candidate
        .canonicalize()
        .map_err(|_| GotenbergError::AssetMissing {
            referrer: String::new(),
            reference: reference.to_string(),
        })?;

    if !roots.iter().any(|root| resolved.starts_with(root)) {
        return Err(GotenbergError::EscapesRoot {
            referrer: String::new(),
            reference: reference.to_string(),
        });
    }
    Ok(resolved)
}

/// Give a resolved asset its flat filename, refusing a collision between two
/// different files and reusing the name when the same file is referenced twice.
///
/// The returned flag is true only the first time a given file is claimed, so
/// the caller enqueues it for processing exactly once.
fn claim_name(
    resolved: &Path,
    claimed: &mut BTreeMap<String, PathBuf>,
    assigned: &mut BTreeMap<PathBuf, String>,
    reference: &str,
    roots: &[PathBuf],
) -> Result<(String, bool), GotenbergError> {
    if let Some(existing) = assigned.get(resolved) {
        return Ok((existing.clone(), false));
    }
    let filename = resolved
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| GotenbergError::AssetMissing {
            referrer: String::new(),
            reference: reference.to_string(),
        })?
        .to_string();

    if let Some(first) = claimed.get(&filename) {
        if first != resolved {
            return Err(GotenbergError::DuplicateBasename {
                filename,
                first: redact(first, roots),
                second: redact(resolved, roots),
            });
        }
    }
    claimed.insert(filename.clone(), resolved.to_path_buf());
    assigned.insert(resolved.to_path_buf(), filename.clone());
    Ok((filename, true))
}

/// True for references that are not local files: remote URLs, data URIs,
/// in-page fragments, and anything rooted at `/` (which gotenberg's flat
/// directory could not serve anyway).
fn is_external(reference: &str) -> bool {
    let r = reference.trim();
    r.is_empty()
        || r.starts_with('#')
        || r.starts_with('/')
        || r.starts_with("//")
        || r.contains("://")
        || {
            let lower = r.to_ascii_lowercase();
            lower.starts_with("data:")
                || lower.starts_with("mailto:")
                || lower.starts_with("tel:")
                || lower.starts_with("javascript:")
        }
}

/// Decode `%XX` escapes. Browsers do; the template's own asset names contain
/// spaces, so a hand-written `%20` must resolve to the same file.
fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(v) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Rewrite every local reference in `text`, calling `resolve` for each one and
/// substituting what it returns.
///
/// HTML is scanned for `href=`/`src=` attributes AND for CSS `url(...)`
/// (inline `<style>` blocks and `style=` attributes use the latter). Content
/// inside HTML comments is skipped: the fill engine emits long provenance
/// comments, and a stray `href=` in one is prose, not a reference.
fn rewrite_references<F>(
    text: &str,
    kind: TextKind,
    mut resolve: F,
) -> Result<String, GotenbergError>
where
    F: FnMut(&str) -> Result<String, GotenbergError>,
{
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;

    while i < bytes.len() {
        if kind == TextKind::Html && text[i..].starts_with("<!--") {
            let end = text[i..]
                .find("-->")
                .map(|p| i + p + 3)
                .unwrap_or(bytes.len());
            out.push_str(&text[i..end]);
            i = end;
            continue;
        }

        let marker = next_marker(text, i, kind);
        let Some((start, len, style)) = marker else {
            out.push_str(&text[i..]);
            break;
        };

        // Everything up to and including the marker is copied verbatim.
        out.push_str(&text[i..start + len]);
        let mut cursor = start + len;

        // Skip whitespace between the marker and the value.
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            out.push(bytes[cursor] as char);
            cursor += 1;
        }
        if cursor >= bytes.len() {
            i = cursor;
            continue;
        }

        let quote = match bytes[cursor] {
            b'"' => Some('"'),
            b'\'' => Some('\''),
            _ => None,
        };

        // `@import url("x")` is two markers deep: leave the bare form to the
        // `url(` marker that follows rather than swallowing `url(...)` whole.
        if style == MarkerStyle::Import && quote.is_none() {
            i = cursor;
            continue;
        }
        let value_start = cursor + usize::from(quote.is_some());
        let terminators: &[u8] = match (quote, style) {
            (Some('"'), _) => b"\"",
            (Some('\''), _) => b"'",
            (_, MarkerStyle::Url) => b")",
            _ => b" \t\r\n>",
        };
        let mut value_end = value_start;
        while value_end < bytes.len() && !terminators.contains(&bytes[value_end]) {
            value_end += 1;
        }
        let reference = &text[value_start..value_end];

        if let Some(q) = quote {
            out.push(q);
        }
        if is_external(reference) {
            out.push_str(reference);
        } else {
            out.push_str(&resolve(reference)?);
        }
        i = value_end;
    }

    Ok(out)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkerStyle {
    Attribute,
    Url,
    Import,
}

/// Find the next reference-introducing token at or after `from`.
fn next_marker(text: &str, from: usize, kind: TextKind) -> Option<(usize, usize, MarkerStyle)> {
    let candidates: &[(&str, MarkerStyle)] = match kind {
        TextKind::Html => &[
            ("href=", MarkerStyle::Attribute),
            ("src=", MarkerStyle::Attribute),
            ("url(", MarkerStyle::Url),
            ("@import ", MarkerStyle::Import),
        ],
        TextKind::Css => &[
            ("url(", MarkerStyle::Url),
            ("@import ", MarkerStyle::Import),
        ],
    };
    candidates
        .iter()
        .filter_map(|(needle, style)| {
            text[from..]
                .find(needle)
                .map(|p| (from + p, needle.len(), *style))
        })
        .min_by_key(|(pos, _, _)| *pos)
}

// ============================================================================
// MULTIPART
// ============================================================================

/// Pick a boundary that appears in none of the parts, so no file content can
/// terminate the body early.
pub fn choose_boundary(files: &[BundledFile]) -> String {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    for salt in 0u32..1024 {
        let candidate = format!("efeesGotenberg{seed:x}{salt:04x}");
        let needle = candidate.as_bytes();
        if !files.iter().any(|f| contains(&f.bytes, needle)) {
            return candidate;
        }
    }
    format!("efeesGotenberg{seed:x}fallback")
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// The MIME type gotenberg should see for a given filename.
///
/// Gotenberg keys its handling off the FILENAME, not this header, so it is
/// informational — but sending `text/css` for a stylesheet keeps the request
/// honest and makes a captured body readable.
fn content_type_for(filename: &str) -> &'static str {
    let lower = filename.to_ascii_lowercase();
    match lower.rsplit('.').next() {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("js") => "text/javascript",
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

/// Build the `multipart/form-data` body: the plain fields first, then every
/// file under the field name `files`, then the closing boundary.
pub fn build_multipart_body(
    boundary: &str,
    fields: &[(String, String)],
    files: &[BundledFile],
) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    for (name, value) in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }
    for file in files {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"files\"; filename=\"{}\"\r\n",
                escape_filename(&file.filename)
            )
            .as_bytes(),
        );
        body.extend_from_slice(
            format!("Content-Type: {}\r\n\r\n", content_type_for(&file.filename)).as_bytes(),
        );
        body.extend_from_slice(&file.bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    body
}

// ============================================================================
// CLIENT
// ============================================================================

/// Renders fp-template HTML through a gotenberg service.
#[derive(Debug, Clone)]
pub struct GotenbergRenderer {
    config: GotenbergConfig,
    roots: Vec<PathBuf>,
}

impl GotenbergRenderer {
    pub fn new(config: GotenbergConfig) -> Self {
        Self {
            config,
            roots: Vec::new(),
        }
    }

    /// Additional directories an asset reference may resolve into, beyond the
    /// document's own. In practice the fp-template checkout, whose stylesheets,
    /// fonts and images a filled proposal legitimately points at.
    pub fn with_roots(mut self, roots: Vec<PathBuf>) -> Self {
        self.roots = roots;
        self
    }

    pub fn config(&self) -> &GotenbergConfig {
        &self.config
    }

    /// Flatten `html_in`, POST it, and write the returned PDF to `pdf_out`.
    pub fn render_to_file(&self, html_in: &Path, pdf_out: &Path) -> Result<(), GotenbergError> {
        let pdf = self.render_to_bytes(html_in)?;
        if let Some(parent) = pdf_out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| GotenbergError::Io {
                path: redact(parent, &[]),
                message: e.to_string(),
            })?;
        }
        std::fs::write(pdf_out, &pdf).map_err(|e| GotenbergError::Io {
            path: redact(pdf_out, &[]),
            message: e.to_string(),
        })
    }

    /// Flatten `html_in` and return the rendered PDF bytes.
    pub fn render_to_bytes(&self, html_in: &Path) -> Result<Vec<u8>, GotenbergError> {
        let bundle = flatten_document(html_in, &self.roots)?;
        self.convert(&bundle)
    }

    /// POST an already-flattened bundle.
    ///
    /// Blocking. Call it from a blocking context (a CLI, or `spawn_blocking`
    /// under an async runtime).
    pub fn convert(&self, bundle: &FlatBundle) -> Result<Vec<u8>, GotenbergError> {
        // Checked before the body is built, so an oversized tree costs one
        // comparison rather than a copy plus an upload.
        let total = bundle.total_bytes();
        if total > self.config.max_bundle_bytes {
            return Err(GotenbergError::BundleTooLarge {
                bytes: total,
                limit: self.config.max_bundle_bytes,
            });
        }
        let boundary = choose_boundary(&bundle.files);
        let body = build_multipart_body(&boundary, &self.config.form_fields(), &bundle.files);
        let url = self.config.convert_url();

        let client = reqwest::blocking::Client::builder()
            .timeout(self.config.timeout)
            .build()
            .map_err(|e| GotenbergError::Transport {
                url: url.clone(),
                message: e.to_string(),
            })?;

        let response = client
            .post(&url)
            .header(
                reqwest::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .map_err(|e| GotenbergError::Transport {
                url: url.clone(),
                message: if e.is_timeout() {
                    format!("timed out after {}s", self.config.timeout.as_secs())
                } else {
                    e.to_string()
                },
            })?;

        let status = response.status().as_u16();
        let bytes = response.bytes().map_err(|e| GotenbergError::Transport {
            url: url.clone(),
            message: e.to_string(),
        })?;

        if !(200..300).contains(&status) {
            // Gotenberg's error body is short plain text ("Invalid form data:
            // form file 'index.html' is required"). The document HTML is never
            // echoed back and must never be logged.
            let detail = error_body(&bytes);
            log::warn!("gotenberg {url} answered {status}: {detail}");
            return Err(if status == 400 {
                GotenbergError::BadRequest {
                    status,
                    body: detail,
                }
            } else {
                GotenbergError::Status {
                    status,
                    body: detail,
                }
            });
        }

        if !bytes.starts_with(b"%PDF") {
            return Err(GotenbergError::NotAPdf { bytes: bytes.len() });
        }
        Ok(bytes.to_vec())
    }
}

/// Gotenberg's error bodies are plain text; keep them short and single-line.
fn error_body(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let flat = text.trim().replace(['\n', '\r'], " ");
    match flat.char_indices().nth(500) {
        Some((cut, _)) => format!("{}...", &flat[..cut]),
        None => flat,
    }
}

/// Count the pages in a PDF by its page objects.
///
/// A deliberately dependency-free heuristic used by tests and logging, not a
/// PDF parser: it counts `/Type /Page` objects while ignoring the `/Type
/// /Pages` tree node. `file`'s own page count is known-unreliable on these
/// documents; this agrees with `pdfinfo` on Chromium output.
pub fn pdf_page_count(bytes: &[u8]) -> usize {
    let mut count = 0usize;
    for needle in [b"/Type /Page".as_slice(), b"/Type/Page".as_slice()] {
        let mut from = 0usize;
        while from + needle.len() <= bytes.len() {
            let Some(pos) = bytes[from..]
                .windows(needle.len())
                .position(|w| w == needle)
            else {
                break;
            };
            let at = from + pos;
            let next = bytes.get(at + needle.len()).copied();
            if next != Some(b's') {
                count += 1;
            }
            from = at + needle.len();
        }
    }
    count
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct TempTree(PathBuf);

    impl TempTree {
        fn new(tag: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("efees-gotenberg-{tag}-{nanos}"));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn write(&self, rel: &str, body: &str) -> PathBuf {
            let path = self.0.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, body).unwrap();
            path
        }

        fn path(&self, rel: &str) -> PathBuf {
            self.0.join(rel)
        }

        #[cfg(unix)]
        fn symlink(&self, rel: &str, target: &Path) -> PathBuf {
            let path = self.0.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(target, &path).unwrap();
            path
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn body_text(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }

    // ---- config ----

    #[test]
    fn convert_url_is_the_chromium_html_route() {
        let config = GotenbergConfig::new("http://10.0.23.31:3000/");
        assert_eq!(
            config.convert_url(),
            "http://10.0.23.31:3000/forms/chromium/convert/html"
        );
    }

    #[test]
    fn form_fields_carry_the_measured_parity_set() {
        let fields = GotenbergConfig::new("http://x:3000").form_fields();
        let names: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
        assert!(names.contains(&"preferCssPageSize"));
        assert!(names.contains(&"printBackground"));
        assert!(names.contains(&"generateTaggedPdf"));
        assert_eq!(
            fields
                .iter()
                .filter(|(k, _)| k.starts_with("margin"))
                .count(),
            4
        );
        for (_, value) in fields.iter().filter(|(k, _)| k.starts_with("margin")) {
            assert_eq!(value, "0");
        }
    }

    #[test]
    fn tagged_pdf_can_be_switched_off() {
        let mut config = GotenbergConfig::new("http://x:3000");
        config.tagged_pdf = false;
        assert!(!config
            .form_fields()
            .iter()
            .any(|(k, _)| k == "generateTaggedPdf"));
    }

    #[test]
    fn bool_env_parsing_accepts_the_usual_spellings() {
        assert_eq!(parse_bool("TRUE"), Some(true));
        assert_eq!(parse_bool(" no "), Some(false));
        assert_eq!(parse_bool("maybe"), None);
    }

    // ---- flattening ----

    #[test]
    fn nested_references_are_flattened_and_followed_transitively() {
        let tree = TempTree::new("nested");
        tree.write("fp/tokens.css", ":root{--x:1}");
        tree.write(
            "fp/template.css",
            "@font-face{src: url(\"font/Ubuntu-Regular.ttf\");}",
        );
        tree.write("fp/font/Ubuntu-Regular.ttf", "TTF-BYTES");
        tree.write("fp/assets/logo white.svg", "<svg/>");
        let html = tree.write(
            "fp/proposals/26-97109/index.html",
            "<html><head><link rel=\"stylesheet\" href=\"../../tokens.css\">\
             <link rel=\"stylesheet\" href=\"../../template.css\"></head>\
             <body><img src=\"../../assets/logo white.svg\"></body></html>",
        );

        // The template root must be passed, exactly as `select_renderer` does:
        // a filled proposal sits below it and points back up at its assets.
        let bundle = flatten_document(&html, &[tree.path("fp")]).unwrap();
        assert_eq!(
            bundle.filenames(),
            vec![
                "index.html",
                "Ubuntu-Regular.ttf",
                "logo white.svg",
                "template.css",
                "tokens.css",
            ]
        );

        let index = body_text(&bundle.files[0].bytes);
        assert!(index.contains("href=\"tokens.css\""), "{index}");
        assert!(index.contains("href=\"template.css\""), "{index}");
        assert!(index.contains("src=\"logo white.svg\""), "{index}");
        assert!(
            !index.contains("../.."),
            "no path segments may survive: {index}"
        );

        let css = bundle
            .files
            .iter()
            .find(|f| f.filename == "template.css")
            .map(|f| body_text(&f.bytes))
            .unwrap();
        assert!(
            css.contains("url(\"Ubuntu-Regular.ttf\")"),
            "the font reference inside CSS must be flattened too: {css}"
        );
    }

    #[test]
    fn the_entry_document_is_always_renamed_to_index_html() {
        let tree = TempTree::new("entry-name");
        let html = tree.write("proposal.html", "<html><body>x</body></html>");
        let bundle = flatten_document(&html, &[]).unwrap();
        assert_eq!(bundle.files[0].filename, INDEX_FILENAME);
    }

    #[test]
    fn the_same_asset_referenced_twice_is_uploaded_once() {
        let tree = TempTree::new("dedup");
        tree.write("logo.svg", "<svg/>");
        let html = tree.write(
            "index.html",
            "<img src=\"logo.svg\"><img src=\"./logo.svg\">",
        );
        let bundle = flatten_document(&html, &[]).unwrap();
        assert_eq!(bundle.filenames(), vec!["index.html", "logo.svg"]);
    }

    #[test]
    fn two_different_files_sharing_a_basename_are_refused_not_silently_merged() {
        let tree = TempTree::new("collide");
        tree.write("a/logo.svg", "<svg id=\"a\"/>");
        tree.write("b/logo.svg", "<svg id=\"b\"/>");
        let html = tree.write(
            "index.html",
            "<img src=\"a/logo.svg\"><img src=\"b/logo.svg\">",
        );
        match flatten_document(&html, &[]).unwrap_err() {
            GotenbergError::DuplicateBasename { filename, .. } => {
                assert_eq!(filename, "logo.svg")
            }
            other => panic!("expected DuplicateBasename, got {other:?}"),
        }
    }

    #[test]
    fn a_missing_asset_is_named_not_dropped() {
        let tree = TempTree::new("missing");
        let html = tree.write("index.html", "<img src=\"assets/gone.svg\">");
        match flatten_document(&html, &[]).unwrap_err() {
            GotenbergError::AssetMissing { reference, .. } => {
                assert_eq!(reference, "assets/gone.svg")
            }
            other => panic!("expected AssetMissing, got {other:?}"),
        }
    }

    #[test]
    fn remote_and_inline_references_are_left_alone() {
        let tree = TempTree::new("external");
        let html = tree.write(
            "index.html",
            "<a href=\"https://example.com/x.css\">l</a>\
             <a href=\"#top\">t</a>\
             <a href=\"mailto:a@b.c\">m</a>\
             <img src=\"data:image/png;base64,AAAA\">",
        );
        let bundle = flatten_document(&html, &[]).unwrap();
        assert_eq!(bundle.filenames(), vec!["index.html"]);
        let index = body_text(&bundle.files[0].bytes);
        assert!(index.contains("https://example.com/x.css"));
        assert!(index.contains("#top"));
        assert!(index.contains("data:image/png;base64,AAAA"));
    }

    #[test]
    fn href_inside_an_html_comment_is_prose_not_a_reference() {
        // The fill engine emits long provenance comments; one containing a
        // path must not be resolved (and must not fail the render).
        let tree = TempTree::new("comment");
        let html = tree.write(
            "index.html",
            "<!-- was: <link href=\"../../old/gone.css\"> -->\n<body>x</body>",
        );
        let bundle = flatten_document(&html, &[]).unwrap();
        assert_eq!(bundle.filenames(), vec!["index.html"]);
        assert!(body_text(&bundle.files[0].bytes).contains("../../old/gone.css"));
    }

    #[test]
    fn percent_escaped_names_resolve_to_the_same_file() {
        let tree = TempTree::new("percent");
        tree.write("assets/logo white.svg", "<svg/>");
        let html = tree.write("index.html", "<img src=\"assets/logo%20white.svg\">");
        let bundle = flatten_document(&html, &[]).unwrap();
        assert_eq!(bundle.filenames(), vec!["index.html", "logo white.svg"]);
        assert!(body_text(&bundle.files[0].bytes).contains("src=\"logo white.svg\""));
    }

    #[test]
    fn inline_style_url_references_are_flattened() {
        let tree = TempTree::new("inline-style");
        tree.write("bg.png", "PNG");
        let html = tree.write(
            "index.html",
            "<style>.a{background:url('bg.png')}</style><body></body>",
        );
        let bundle = flatten_document(&html, &[]).unwrap();
        assert_eq!(bundle.filenames(), vec!["index.html", "bg.png"]);
        assert!(body_text(&bundle.files[0].bytes).contains("url('bg.png')"));
    }

    #[test]
    fn file_order_is_deterministic() {
        let tree = TempTree::new("order");
        tree.write("z.css", "");
        tree.write("a.css", "");
        let html = tree.write("index.html", "<link href=\"z.css\"><link href=\"a.css\">");
        let first = flatten_document(&html, &[]).unwrap();
        let second = flatten_document(&html, &[]).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.filenames(), vec!["index.html", "a.css", "z.css"]);
    }

    // ---- containment ----

    #[test]
    fn a_traversal_reference_is_refused_not_bundled() {
        // The decisive case: without containment this reads a file outside the
        // document's tree and uploads it to the render service.
        let tree = TempTree::new("traversal");
        tree.write("outside/secret.txt", "SENSITIVE");
        let html = tree.write(
            "doc/index.html",
            "<link rel=\"stylesheet\" href=\"../outside/secret.txt\">",
        );
        match flatten_document(&html, &[]).unwrap_err() {
            GotenbergError::EscapesRoot { reference, .. } => {
                assert_eq!(reference, "../outside/secret.txt")
            }
            other => panic!("expected EscapesRoot, got {other:?}"),
        }
    }

    #[test]
    fn a_deep_traversal_to_a_system_file_is_refused() {
        let tree = TempTree::new("deep-traversal");
        // Enough levels to reach `/` from any temp path (`/..` is `/`), then a
        // file that exists on macOS and Linux alike - so this proves the
        // containment check fired, not that the target happened to be absent.
        let up = "../".repeat(40);
        let html = tree.write("doc/index.html", &format!("<img src=\"{up}etc/hosts\">"));
        assert!(matches!(
            flatten_document(&html, &[]).unwrap_err(),
            GotenbergError::EscapesRoot { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_pointing_out_of_the_tree_is_refused() {
        // Containment runs on the CANONICALISED path, so a link whose target
        // escapes is caught even though the link itself sits inside the root.
        let tree = TempTree::new("symlink-escape");
        tree.write("outside/secret.css", "body{}");
        tree.write("doc/placeholder", "");
        tree.symlink("doc/theme.css", &tree.path("outside/secret.css"));
        let html = tree.write("doc/index.html", "<link href=\"theme.css\">");
        assert!(matches!(
            flatten_document(&html, &[]).unwrap_err(),
            GotenbergError::EscapesRoot { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_staying_inside_an_allowed_root_is_accepted() {
        // The guard must not break the legitimate case: fp-template is a second
        // root, and a link into it resolves inside that root.
        let tree = TempTree::new("symlink-ok");
        tree.write("template/tokens.css", "body{}");
        tree.write("doc/placeholder", "");
        tree.symlink("doc/tokens.css", &tree.path("template/tokens.css"));
        let html = tree.write("doc/index.html", "<link href=\"tokens.css\">");
        let bundle = flatten_document(&html, &[tree.path("template")]).unwrap();
        assert_eq!(bundle.filenames(), vec!["index.html", "tokens.css"]);
    }

    #[test]
    fn an_extra_root_widens_containment_to_the_template_tree() {
        let tree = TempTree::new("extra-root");
        tree.write("template/tokens.css", "body{}");
        let html = tree.write("doc/index.html", "<link href=\"../template/tokens.css\">");

        // Without the template root, the reference escapes.
        assert!(matches!(
            flatten_document(&html, &[]).unwrap_err(),
            GotenbergError::EscapesRoot { .. }
        ));
        // With it, the same document flattens.
        let bundle = flatten_document(&html, &[tree.path("template")]).unwrap();
        assert_eq!(bundle.filenames(), vec!["index.html", "tokens.css"]);
    }

    #[test]
    fn error_messages_name_the_file_not_the_absolute_host_path() {
        // These strings reach the 422 body and the warn! line, so they must not
        // expose the container's directory layout.
        let tree = TempTree::new("no-abs-paths");
        let html = tree.write("index.html", "<img src=\"assets/gone.svg\">");
        let message = flatten_document(&html, &[]).unwrap_err().to_string();
        assert!(message.contains("index.html"), "{message}");
        assert!(
            !message.contains(&tree.0.display().to_string()),
            "absolute host path leaked: {message}"
        );
    }

    #[test]
    fn a_collision_message_names_both_files_relatively() {
        let tree = TempTree::new("collide-msg");
        tree.write("a/logo.svg", "<svg id=\"a\"/>");
        tree.write("b/logo.svg", "<svg id=\"b\"/>");
        let html = tree.write(
            "index.html",
            "<img src=\"a/logo.svg\"><img src=\"b/logo.svg\">",
        );
        let message = flatten_document(&html, &[]).unwrap_err().to_string();
        assert!(
            message.contains("a/logo.svg") && message.contains("b/logo.svg"),
            "{message}"
        );
        assert!(
            !message.contains(&tree.0.display().to_string()),
            "absolute host path leaked: {message}"
        );
    }

    // ---- size cap ----

    #[test]
    fn an_oversized_bundle_is_refused_before_anything_is_sent() {
        let mut config = GotenbergConfig::new("http://127.0.0.1:1");
        config.max_bundle_bytes = 10;
        let bundle = FlatBundle {
            files: vec![BundledFile {
                filename: "index.html".into(),
                bytes: vec![b'x'; 64],
            }],
        };
        match GotenbergRenderer::new(config).convert(&bundle).unwrap_err() {
            GotenbergError::BundleTooLarge { bytes, limit } => {
                assert_eq!(bytes, 64);
                assert_eq!(limit, 10);
            }
            other => panic!("expected BundleTooLarge, got {other:?}"),
        }
    }

    #[test]
    fn total_bytes_sums_every_member() {
        let bundle = FlatBundle {
            files: vec![
                BundledFile {
                    filename: "index.html".into(),
                    bytes: vec![b'a'; 3],
                },
                BundledFile {
                    filename: "x.css".into(),
                    bytes: vec![b'b'; 4],
                },
            ],
        };
        assert_eq!(bundle.total_bytes(), 7);
    }

    // ---- multipart ----

    #[test]
    fn multipart_body_has_fields_then_files_then_a_closing_boundary() {
        let files = vec![
            BundledFile {
                filename: "index.html".into(),
                bytes: b"<html></html>".to_vec(),
            },
            BundledFile {
                filename: "logo white.svg".into(),
                bytes: b"<svg/>".to_vec(),
            },
        ];
        let fields = vec![("printBackground".to_string(), "true".to_string())];
        let body = build_multipart_body("BOUND", &fields, &files);
        let text = body_text(&body);

        assert!(text.starts_with(
            "--BOUND\r\nContent-Disposition: form-data; name=\"printBackground\"\r\n\r\ntrue\r\n"
        ));
        assert!(text.contains(
            "Content-Disposition: form-data; name=\"files\"; filename=\"index.html\"\r\n\
             Content-Type: text/html; charset=utf-8\r\n\r\n<html></html>\r\n"
        ));
        assert!(text.contains("filename=\"logo white.svg\""));
        assert!(text.contains("Content-Type: image/svg+xml"));
        assert!(text.ends_with("--BOUND--\r\n"));
        assert_eq!(text.matches("--BOUND\r\n").count(), 3);
    }

    #[test]
    fn every_file_uses_the_field_name_gotenberg_requires() {
        let files = vec![BundledFile {
            filename: "tokens.css".into(),
            bytes: b"x".to_vec(),
        }];
        let text = body_text(&build_multipart_body("B", &[], &files));
        assert_eq!(text.matches("name=\"files\"").count(), 1);
        assert!(text.contains("Content-Type: text/css; charset=utf-8"));
    }

    #[test]
    fn the_boundary_never_appears_inside_a_part() {
        let files = vec![BundledFile {
            filename: "index.html".into(),
            bytes: b"efeesGotenberg".to_vec(),
        }];
        let boundary = choose_boundary(&files);
        assert!(!contains(&files[0].bytes, boundary.as_bytes()));
    }

    #[test]
    fn quotes_in_a_filename_cannot_break_the_header() {
        assert_eq!(escape_filename("a\"b\\c"), "a\\\"b\\\\c");
        assert_eq!(escape_filename("a\r\nb"), "ab");
    }

    // ---- error classification ----

    #[test]
    fn a_malformed_bundle_is_our_fault_and_a_dead_service_is_not() {
        assert!(GotenbergError::BadRequest {
            status: 400,
            body: "form file 'index.html' is required".into()
        }
        .is_caller_fault());
        assert!(GotenbergError::AssetMissing {
            referrer: "x".into(),
            reference: "y".into()
        }
        .is_caller_fault());
        assert!(!GotenbergError::Status {
            status: 503,
            body: String::new()
        }
        .is_caller_fault());
        assert!(!GotenbergError::Transport {
            url: "http://x".into(),
            message: "refused".into()
        }
        .is_caller_fault());
    }

    #[test]
    fn error_bodies_are_flattened_and_capped() {
        assert_eq!(error_body(b"  boom\nagain  "), "boom again");
        assert_eq!(error_body(&vec![b'x'; 900]).len(), 503);
    }

    // ---- pdf helpers ----

    #[test]
    fn page_counting_ignores_the_pages_tree_node() {
        let pdf = b"%PDF-1.4 /Type /Pages /Count 2 /Type /Page x /Type /Page y";
        assert_eq!(pdf_page_count(pdf), 2);
    }
}
