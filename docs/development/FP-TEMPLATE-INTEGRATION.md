# fp-template integration (fee record to proposal PDF)

*(as of 2026-09-06)*

How a fee record becomes a rendered proposal, without InDesign.

## Two proposal backends: the remote service, and the local pipeline

As of 2026-09-06, `POST /fees/{id}/fp-proposal` builds through ONE of two
backends, chosen once by `export::fp_render::select_proposal_backend`:

| `DOCBUILDER_URL` | Backend | This host needs |
|---|---|---|
| set | the remote **fp-docbuilder service** | a reachable docbuilder service only — no `FP_TEMPLATE_ROOT`, no python3, no browser |
| unset | the **local pipeline** below (`FP_TEMPLATE_ROOT` required) | `FP_TEMPLATE_ROOT`, python3, and (unless `GOTENBERG_URL` is also set) a local headless Chrome |

`DOCBUILDER_URL` takes priority and is the **production path**: the
fp-docbuilder service (gtm repo) owns fill, the three pre-issue gates, and
rendering end to end, so `e-fees-api` builds only the manifest
(`export::fp_manifest`, pure Rust — no I/O) and POSTs it. The `e-fees-api`
image no longer carries python3 or a vendored fp-template checkout at all
(dropped with the docbuilder client) — see "Building the image" below.

The **local pipeline** (this file's original subject, `GOTENBERG_URL` set or
not) remains in the code, still compiled and tested, as the **development-only**
fallback for a host that has `FP_TEMPLATE_ROOT` (and, without `GOTENBERG_URL`,
a browser). It is documented in full below.

## The local pipeline

```
fee + project + company + contact
  -> {number}-var Default Values.json   (export::build_fee_json)
  -> manifest.md                        (export::fp_manifest)
  -> python3 fp-template/fill.py manifest.md proposal.html
  -> the three pre-issue gates, on the HTML
  -> PDF, via one of two render backends
```

The first two files must sit in the same directory: fp-template resolves the
manifest's `vars_source` relative to the manifest itself.

The gates run BEFORE the render, not after. A document that must not be issued
therefore never reaches a renderer, and with the remote gotenberg backend it
never leaves this host. (The docbuilder path above gates on the SERVICE side,
before it ever answers — see the docbuilder contract further down.)

## Render backends (local pipeline)

| `GOTENBERG_URL` | Backend | Needs |
|---|---|---|
| set | POST to gotenberg | a reachable gotenberg service |
| unset | fp-template's `render.sh` | a local headless Chrome |

`export::fp_render::select_renderer` applies that rule, so the CLI and the API
agree without either passing a flag. There is no host default for
`GOTENBERG_URL`: unset means "this deployment renders locally", never "try
some address".

### Environment variables

| Variable | Default | Meaning |
|---|---|---|
| `DOCBUILDER_URL` | none | Base URL of the remote fp-docbuilder service. Set = production path, everything below this row is unused. **Production value: `http://10.0.21.85:8080`** (once deployed). |
| `DOCBUILDER_TOKEN` | none | Bearer token sent with every docbuilder request, when set. |
| `DOCBUILDER_TIMEOUT_SECS` | `120` | Per-request budget for the docbuilder client. |
| `DOCBUILDER_TAGGED_PDF` | `true` | Sent as the `tagged` query parameter on every request, matching the local pipeline's `GOTENBERG_TAGGED_PDF` default. |
| `FP_TEMPLATE_ROOT` | none | fp-template checkout, LOCAL PIPELINE ONLY (ignored when `DOCBUILDER_URL` is set). Required by both local backends: `fill.py` and the gates live there. Unset (with `DOCBUILDER_URL` also unset) gives 503. |
| `GOTENBERG_URL` | none | Base URL, LOCAL PIPELINE ONLY. Set = use gotenberg. **Value: `http://10.0.21.83:3000`** (gotenberg-ai, on the AI host). |
| `GOTENBERG_TIMEOUT_SECS` | `60` | Per-request budget. |
| `GOTENBERG_TAGGED_PDF` | `true` | Send `generateTaggedPdf`, matching local Chromium's tagged output. |
| `GOTENBERG_MAX_BUNDLE_BYTES` | `26214400` (25 MiB) | Refuse to upload a flattened bundle larger than this. |

### What the gotenberg backend does

Gotenberg stores every uploaded file in one flat directory and cannot follow
subdirectories. A filled proposal is the opposite shape: the fill engine
rebases stylesheets and images against the output file's own location, so it
carries `../../tokens.css` and `../../assets/logo white 1024 513.svg`, and
`template.css` in turn carries `url("font/Montserrat-Regular.ttf")`.

`export::gotenberg::flatten_document` therefore walks the document, follows
HTML to CSS to fonts and images, rewrites every reference to a bare filename,
and uploads the result as one bundle. It refuses rather than guessing when a
reference names a file that is not there, and when two different files would
collide on one flat name (gotenberg would silently keep one of them).

**Containment.** Flattening reads whatever the document references and sends it
to a service on the network, so every reference must resolve inside an allowed
root: the per-request working directory plus the fp-template checkout. A
traversal such as `../../../../etc/hosts` is refused, and because the check
runs on the canonicalised path, so is a symlink whose target escapes. The check
happens before the file is read, so an out-of-tree file never enters a bundle.

Two smaller guards travel with it. Error text names files relative to a root,
or by bare filename, never by absolute path, because those strings reach both
the 422 body and the logs. And the bundle has a size cap
(`GOTENBERG_MAX_BUNDLE_BYTES`, default 25 MiB) checked before the request body
is built.

Form fields sent with every conversion: `preferCssPageSize=true`,
`printBackground=true`, all four margins `0`, and `generateTaggedPdf=true`.
Those first three are what made the measured render match local Chromium to
antialiasing noise. Unknown field names are IGNORED by gotenberg rather than
rejected, so a misspelt one fails silently and produces a subtly different
PDF — never invent one.

### Status mapping

Measured against gotenberg 8.36.0 on 2026-09-06, not assumed:

| Gotenberg outcome | API status | Why |
|---|---|---|
| `400 Invalid form data: ...` | 422 | our bundle was malformed; the caller's document cannot be rendered as written |
| a missing or colliding asset (before any request) | 422 | same class, caught locally |
| a reference resolving outside the allowed roots | 422 | the document asked for a file that is not ours |
| a bundle over the size cap | 422 | our request is too big to send |
| connection refused, DNS failure, timeout | 503 | the service is unavailable |
| any other non-2xx | 503 | the service failed |
| 200 with a non-PDF body | 503 | the service misbehaved |

A successful response carries `x-fp-render-backend: gotenberg` (or
`local-chromium`), alongside the existing `x-fp-residuals` and
`x-fp-issue-hits`.

## Where the pieces live

| Piece | Where |
|---|---|
| Manifest builder (pure, tested) | `crates/e-fees-core/src/export/fp_manifest.rs` |
| Docbuilder client (production backend) | `crates/e-fees-core/src/export/docbuilder.rs` |
| Live end-to-end test against docbuilder (env gated) | `crates/e-fees-core/tests/fp_docbuilder_live.rs` |
| Render driver, backend selection, gates (local pipeline) | `crates/e-fees-core/src/export/fp_render.rs` |
| Gotenberg client + asset flattening (local pipeline) | `crates/e-fees-core/src/export/gotenberg.rs` |
| Live end-to-end test against gotenberg (env gated) | `crates/e-fees-core/tests/fp_gotenberg_live.rs` |
| CLI (read-only, DB to PDF) | `crates/e-fees-core/src/bin/fp_export.rs` |
| API routes | `e-fees-api/src/routes/fp_proposal.rs` |
| fp-docbuilder service | gtm repo, `fp-docbuilder-service/` |
| Template + fill engine (local pipeline only) | gtm repo, `fp-template/` (also packaged as `emittiv/fp-template`) |
| Local-pipeline vendoring script + pinned commit (dev-only, no longer used by the image build) | `scripts/vendor-fp-template.sh`, `e-fees-api/fp-template.ref` |
| Vendored section-ID contract | `crates/e-fees-core/src/resources/fp_section_inventory.json` |

## Using it

```bash
# manifest only, no render toolchain needed
cargo run -p e-fees-core --bin fp_export -- \
  --target dev --fee 25_96501_1 --out-dir /tmp/25-96501

# full render, plus fp-template's three pre-issue gates
cargo run -p e-fees-core --bin fp_export -- \
  --target dev --fee 25_96501_1 --out-dir /tmp/25-96501 \
  --render --gates --fp-template-root /path/to/gtm/fp-template
```

API: `GET /fees/{id}/fp-manifest` (always available) and
`POST /fees/{id}/fp-proposal`. The optional POST body supplies per-project prose
(`reference_documents`, `areas`, `assumptions`, `project_details`). Statuses:

| Status | When |
|---|---|
| `200 application/pdf` | rendered AND passed the release gate (either backend) |
| `422` | the gate blocked it, or the manifest failed fp-template's own check. Body carries the gate's (or docbuilder's) report verbatim. Never a PDF. |
| `404` | fee or a linked project/company/contact missing |
| `503` | neither `DOCBUILDER_URL` nor `FP_TEMPLATE_ROOT` is set; `FP_TEMPLATE_ROOT` is set but not an fp-template checkout; or the selected backend (docbuilder or gotenberg) is unreachable or failing |

To render through gotenberg from the CLI, set the same variables:

```bash
FP_TEMPLATE_ROOT=/path/to/gtm/fp-template GOTENBERG_URL=http://10.0.21.83:3000 \
  cargo run -p e-fees-core --bin fp_export -- \
    --target dev --fee 25_96501_1 --out-dir /tmp/25-96501 --render --gates
```

## Building the image

As of 2026-09-06 the `e-fees-api` image carries **no** python3, no vendored
fp-template checkout, and no browser — production rendering goes to the
remote fp-docbuilder service via `DOCBUILDER_URL`. Building the image is now
plain `docker build`, with no vendoring step and no `emittiv/gtm` access
needed:

```bash
docker build -f e-fees-api/Dockerfile -t e-fees-api .
```

CI (`.forgejo/workflows/build-containers.yml`) dropped its "Vendor
fp-template" step accordingly. The `GTM_READ_TOKEN` secret that step used is
therefore **unused by this workflow now**; it is left registered in case a
future local/gotenberg-mode image build needs it again.

`scripts/vendor-fp-template.sh` and `e-fees-api/fp-template.ref` are unchanged
and still work exactly as before — they are just no longer part of the image
build. They remain useful for exercising the LOCAL pipeline: populate
`.vendor/fp-template` and point `FP_TEMPLATE_ROOT` at it to run
`fp_gotenberg_live.rs`, the `fp_export` CLI with `--render --gates`, or a
locally-built image that still needs the old toolchain baked in (not the
default `Dockerfile` any more — build a dev variant if this is needed).

## The release gate

Every path that hands a rendered proposal to a client runs all three of
fp-template's pre-issue gates through `fp_render::run_release_gates`.

`--column-check` and `--quote-check` are unambiguous: a failure is a real fault
in the rendered document and blocks.

`--issue-check` is **not** a pass/fail verdict, and treating its exit code as
one would reject every proposal we ever render. `render.sh`'s own documentation
says so: a filled proposal's bracketed-token count is not expected to be zero,
because the template's permanent legal boilerplate uses bracket-qualifier
drafting (`[Client]`, `[60]`, `[RIBA]`) that a proposal correctly inherits.
Measured on a real render: 22 hits, all of them the template's own.

So the block condition is the sentinel this exporter writes and the template
never does, `INCOMPLETE_MARKER` ("TO BE COMPLETED"), found in the issue-check
report. The full report travels with the decision either way, so a human can
judge the remaining hits as `render.sh` intends. A gate we could not run counts
as a block, never a pass.

## The docbuilder client (production path)

### The request is multipart, not a bare markdown body

The manifest's YAML frontmatter carries `vars_source: {number}-var Default
Values.json`, and both the local pipeline's `fill.py` and the docbuilder
service resolve that path **relative to the manifest file itself** — a
manifest sent alone can never resolve it, so a bare `text/markdown` body
always 422s at stage `"manifest"`. The request is therefore
`multipart/form-data` with exactly two file parts:

| Part | Filename | Content |
|---|---|---|
| `manifest` | any (`manifest.md`) | the manifest markdown (`export::fp_manifest`) |
| `vars` | MUST match the manifest frontmatter's `vars_source` exactly | the variables JSON (`export::build_fee_json`) |

The service writes both into one temporary directory under their given
filenames, so `vars_source` resolves the same way it does in the local
pipeline's own scratch directory. Query params (`tagged`, `filename`),
headers, and response codes are otherwise unchanged from the original design.
`crates/e-fees-core/src/export/docbuilder.rs`'s module docs carry the full,
current contract — this section is a summary, not the source of truth.

### What is proven, and what is not

Proven this session (2026-09-06), against a mock HTTP server, not a live
docbuilder deployment (`fp-docbuilder-service` was being built in the gtm
repo concurrently and was not yet reachable): the client sends the multipart
request above with the right field names and filenames, `Authorization:
Bearer` only when a token is configured, parses a `200` into PDF bytes plus
its three headers, passes a `422` body's `stage` and `report` through
verbatim, maps a `503` body's `error` through, times out on its own budget
rather than hanging, and the same route-level wiring
(`ProposalBuildError::Blocked` → 422, `::Unavailable` → 503) that the existing
gotenberg path uses. `crates/e-fees-core/tests/fp_docbuilder_live.rs` exists
for the real end-to-end check and is skipped (not failed) until
`DOCBUILDER_URL` names a running instance — run it once the service is
deployed, the same way `fp_gotenberg_live.rs` was proven on 2026-09-06.

## What is proven, and what is not

Proven end to end on 2026-09-04 against the dev database (10.0.23.12), fee
`25_96501_1`, both gate outcomes:

- **no narrative supplied** — release gate BLOCKS, CLI exits 1, naming both of
  our own placeholders in its report. The API returns 422, never the PDF.
- **narrative supplied** (`--reference-doc`, `--areas`) — release gate PASSES
  with 22 advisory hits from the template's own boilerplate; 14-page HTML,
  1920x1080pt PDF, and zero occurrences of the incomplete sentinel in the
  rendered document.

Proven end to end through gotenberg on 2026-09-06, by
`crates/e-fees-core/tests/fp_gotenberg_live.rs` against gotenberg 8.36.0 at
both gotenberg instances, using fixture records rather than the database, and
against the vendored tree at the pinned commit:

- `fill.py` built a 14-page HTML document; all three gates passed with 22
  advisory issue-check hits from the template's own boilerplate.
- The flattened bundle was exactly the nine files the document needs:
  `index.html`, `template.css`, `tokens.css`, both Montserrat faces, both
  Ubuntu faces, and the two SVGs. No member carried a path separator.
- Gotenberg returned a 193,944-byte PDF of **14 pages** — the same count
  `fill.py` reported, which is the check that catches a silently dropped
  stylesheet (a lost `@page` rule collapses many pages into few).
- Run against the production instance (`http://10.0.21.83:3000`) and against
  the other one (`http://10.0.23.31:3000`), the output was byte-identical at
  193,944 bytes and 14 pages, so the two are interchangeable for this document.

This closes the gap the 2026-09-05 parity measurement named: that test rendered
a hand-filled proposal already committed in fp-template, never the fill
engine's own output.

### JavaScript is disabled on these instances, and that is fine here

Both deployments run Chromium with JavaScript disabled by policy. It does not
affect this pipeline: the fill engine emits no `<script>` tags, so nothing in a
generated proposal depends on script execution, and the measured renders above
are complete. The one document that WOULD be affected is fp-template's own
hand-filled reference proposal, which sets its issue date from an inline
script — a hand-editing convenience, not part of the programmatic build path.
If a future template change introduces script-dependent content, it will render
blank here rather than fail loudly, so add the content through the fill engine
instead.

### The browser that is still required, and where

Gotenberg removes the browser from the RENDER step only. `fill.py` still needs
one, because fp-template's `engine/measure.py` measures every block's height in
a real Playwright Chromium render before deciding page breaks — it has no
remote or cached mode and raises rather than guessing a height.

So the `e-fees-api` image, which carries python3 and fp-template but no
browser, can serve the manifest route and run the three gates, but `fill.py`
will fail there and the render route answers 422 with that failure. The
render path is proven from a host that has a browser (a developer machine, the
CLI). Closing this needs a change in fp-template — a measurement backend that
talks to a remote browser, or a cached measurement pass — and is tracked
against gtm, not here.

## Known gaps, by design not by accident

1. **Narrative sections.** `reference-documents` and `areas` are
   `override-required` in fp-template and have no e-fees field. Absent caller
   input they render as `[TO BE COMPLETED ...]`, and the release gate then
   refuses to release the document (422 from the API, exit 1 from the CLI) — an
   incomplete proposal cannot be issued, silently or otherwise. The Phase 4
   scope-assembly UI is the intended source; the API and the CLI accept them
   meanwhile.
2. **Discipline vocabulary.** e-fees has five standard disciplines plus `Sub`;
   fp-template's document has sections for three (lighting, video, sound).
   `Audio` maps to `sound`; anything else (`Control`, `SFX`) produces a named
   residual. Its money still appears in the fee totals; only the descriptive
   sections are missing.
3. **Stage names.** e-fees stage names are free text; the template's Services
   section describes four fixed stages. A stage that matches none of them
   (e.g. `Construction Documents`) is reported as a residual.
4. **D2 client-visibility rule.** `kind=Sub` and `sub_company` must never reach
   a client document. The check is case-insensitive on whole alphanumeric
   tokens, so it also catches a hyphenated `sub-consultant`; that is deliberate
   fail-closed behaviour, and the fix is to rephrase the prose rather than
   weaken the guard. A discipline literally named `Sub` is REFUSED, because the
   fee's discipline rows carry no `sub_discipline` label to substitute (the D2
   shape landed on `projects.disciplines[]` in migration 006, not on
   `fee.pricing.disciplines[]`). When those fields reach the fee model, relabel
   instead of refusing and populate `sub_company_names()` so the second half of
   the guard goes live.
5. **Template commit pin.** `DEFAULT_TEMPLATE_COMMIT` and the vendored
   `fp_section_inventory.json` are pinned to fp-template commit `932b4cd`. A
   test fails if the constants drift from the fixture; re-copy the fixture when
   fp-template's inventory changes.
