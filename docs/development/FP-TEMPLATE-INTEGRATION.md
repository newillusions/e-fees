# fp-template integration (fee record to proposal PDF)

*(as of 2026-09-04)*

How a fee record becomes a rendered proposal, without InDesign.

## The pipeline

```
fee + project + company + contact
  -> {number}-var Default Values.json   (export::build_fee_json, pre-existing)
  -> manifest.md                        (export::fp_manifest, new)
  -> python3 fp-template/fill.py manifest.md proposal.html
  -> fp-template/render.sh proposal.pdf proposal.html
```

The two files must sit in the same directory: fp-template resolves the
manifest's `vars_source` relative to the manifest itself.

## Where the pieces live

| Piece | Where |
|---|---|
| Manifest builder (pure, tested) | `crates/e-fees-core/src/export/fp_manifest.rs` |
| Render driver (subprocess + timeout) | `crates/e-fees-core/src/export/fp_render.rs` |
| CLI (read-only, DB to PDF) | `crates/e-fees-core/src/bin/fp_export.rs` |
| API routes | `e-fees-api/src/routes/fp_proposal.rs` |
| Template + fill engine | gtm repo, `fp-template/` (also packaged as `emittiv/fp-template`) |
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
| `200 application/pdf` | rendered AND passed the release gate |
| `422` | the gate blocked it, or the manifest failed fp-template's own check. Body carries the gate's report verbatim. Never a PDF. |
| `404` | fee or a linked project/company/contact missing |
| `503` | `FP_TEMPLATE_ROOT` unset, or not an fp-template checkout |

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

## What is proven, and what is not

Proven end to end on 2026-09-04 against the dev database (10.0.23.12), fee
`25_96501_1`, both gate outcomes:

- **no narrative supplied** — release gate BLOCKS, CLI exits 1, naming both of
  our own placeholders in its report. The API returns 422, never the PDF.
- **narrative supplied** (`--reference-doc`, `--areas`) — release gate PASSES
  with 22 advisory hits from the template's own boilerplate; 14-page HTML,
  1920x1080pt PDF, and zero occurrences of the incomplete sentinel in the
  rendered document.

NOT proven: rendering from inside the `e-fees-api` container. The render path
needs python3 (pyyaml, beautifulsoup4, playwright), a headless Chrome, and an
fp-template checkout on the same host, and the API image carries none of them.
The manifest route works regardless. Packaging the renderer, or splitting it
into its own container, is an open deployment decision.

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
