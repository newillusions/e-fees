# IDW T5 (Reimbursable Costs) → .indd Linking — Scope

**Status:** scoped 2026-06-14, not yet executed (carryover). One-time manual InDesign step.
**Context:** the issued `.indd` proposal links its pricing tables to sheets in the generated IDW workbook. Tables T0–T4 are already linked; the **T5 Reimbursable Costs** table is the only missing link, so reimbursable / provisional-sum costs do not yet surface in the issued proposal. (This is the structural gap behind the Lulu Acoustics gotcha `obs:ve11z833mnxb9p8yiqsf` — but note the recommended workaround there is to model on-charged buy-ins as discipline lines, which flow into T2; T5 is for genuine reimbursables/provisional sums.)

## Already done (code) — no work needed
`crates/e-fees-core/src/export/indesign_workbook.rs` already emits the T5 sheet (verified 2026-06-14):
- Sheet name: **`T5 Reimbursable Costs`**
- Columns A–E: **Stage | Description | Base Cost | Markup | Cost to Client**
- One row per cost; a final **Total** row (label in col A, grand total in col E "Cost to Client"; cols B–D blank).
- Source: `fee.reimbursable_costs` (preferred) → fallback `pricing.costs`. Model `ReimbursableCost { description, stage_id, base_cost, markup_percent, cost_to_client, … }`.
- Produced by the `export_indesign_workbook` Tauri command → `<proposal_dir>/<number>-IDW Pricing.xlsx`.

## Linking mechanism (per design spec)
`docs/superpowers/specs/2026-03-29-indesign-table-export-design.md` §"Layer 2: InDesign Template Linking (manual, one-time)":
- Each `.indd` table is a **Placed (Cmd+D) link** to one Excel sheet/range — NOT data-merge, NOT UXP (UXP automation is explicitly out of scope/future).
- Place with "Create Static Captions" off, "Replace Selected Item" on; refresh later via **Links panel → Update Link**.
- Design-only proposals: sheets still exist with header-only rows; the user deletes the unused linked table + surrounding text manually in InDesign.

## The one-time task
1. Open the canonical proposal `.indd` template. **(Confirm the exact template file/path at execute time — the existing T0–T4 tables live in story 25.)**
2. In story 25, after the **T4 Payment Schedule** table, add a section heading (e.g. "Reimbursable Costs / Provisional Sums") + intro line matching the other sections' style.
3. **Place (Cmd+D)** the project's `…-IDW Pricing.xlsx`, choose the **`T5 Reimbursable Costs`** sheet, import range covering header + data + Total (`A1:E{n+1}`), Show Import Options → linked table.
4. Apply the existing pricing-table cell/paragraph styles so T5 matches T0–T4. The **Total** row mirrors T2/T3/T4: "Total" label spanning Stage/Description/Base Cost/Markup (colSpan 4) with the grand total in the Cost to Client column.
5. Save the template. Future exports + "Update Links" repopulate it automatically.

## Open details to confirm at execute time
- Canonical `.indd` template path (the one with the T0–T4 tables in story 25).
- Whether reimbursables are currently used on any live proposal (T5 may be header-only / empty until a fee actually carries `reimbursable_costs`).
- Could be assisted via the InDesign UXP MCP (`place_file_on_spread` / table tools) with InDesign open on the template, instead of a fully manual Place.

## References
- Code: `crates/e-fees-core/src/export/indesign_workbook.rs` (T5 sheet, lines ~236–290), `src-tauri/src/commands/export.rs` (`export_indesign_workbook`).
- Spec/plan: `docs/superpowers/specs/2026-03-29-indesign-table-export-design.md`, `docs/superpowers/plans/2026-03-29-indesign-workbook-export.md`.
- Gotcha: `obs:ve11z833mnxb9p8yiqsf` (buy-ins as discipline lines vs reimbursables).
