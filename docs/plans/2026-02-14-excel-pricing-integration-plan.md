# Excel Pricing Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Import historical pricing data from ~60 Excel files into SurrealDB, export pricing back to Excel template format, and add fee version management UI.

**Architecture:** Three independent workstreams. Workstream A (import) uses Excel MCP + SurrealDB MCP as a standalone task agent — no app dependency. Workstream B (export) adds a Rust function to generate template-format Excel. Workstream C (versioning UI) adds revision creation and history display in Svelte.

**Tech Stack:** Rust (rust_xlsxwriter), Svelte 5, SurrealDB, Excel MCP, TypeScript

---

## Workstream A: Historical Import Agent

### Task 1: Add `import_source` field to Fee struct

**Files:**
- Modify: `src-tauri/src/db/types.rs:115-150` (Fee struct)
- Modify: `src-tauri/src/db/types.rs:153-187` (FeeCreate struct)
- Modify: `src/types/database.ts:82-111` (TypeScript Fee interface)

**Step 1: Add field to Rust Fee struct**

In `src-tauri/src/db/types.rs`, add after line 149 (`current_release_number`):

```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_source: Option<serde_json::Value>,
```

**Step 2: Add field to Rust FeeCreate struct**

In the same file, add after line 186:

```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_source: Option<serde_json::Value>,
```

**Step 3: Add field to TypeScript Fee interface**

In `src/types/database.ts`, add after line 110:

```typescript
  import_source?: {
    file_path: string;
    file_modified: string;
    file_size: number;
    imported_at: string;
    import_version: string;
    checksum: string;
    project_folder_status: string;
    filename_pattern: string;
  };
```

**Step 4: Update test helper in excel_export.rs**

In `src-tauri/src/excel_export.rs`, add `import_source: None,` to both `minimal_fee()` (after line 443) and `full_fee()` (inherits from minimal_fee, no change needed).

**Step 5: Verify compilation**

Run: `cd /Volumes/base/dev/app/e-fees && cargo check`
Expected: Compilation succeeds (may warn about unused field)

**Step 6: Run existing tests**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test`
Expected: All existing tests pass

**Step 7: Commit**

```bash
git add src-tauri/src/db/types.rs src/types/database.ts src-tauri/src/excel_export.rs
git commit -m "feat(import): add import_source field to Fee struct for source tracking"
```

---

### Task 2: Create import agent script

This is a standalone TypeScript script that uses Excel MCP and SurrealDB MCP tools. It runs as a Claude Code task agent — not part of the app.

**Files:**
- Create: `src/lib/api/ailx.ts` (import agent utilities — already tracked in git status)
- Create: `src/lib/api/ailx.test.ts` (tests — already tracked)

**Wait**: This task is executed as a Claude Code task agent, not compiled code. The agent uses MCP tools directly. See Task 3 for the agent prompt.

---

### Task 3: Run the import agent

This task is a Claude Code task agent invocation. The agent:

1. Scans project folders at `/Volumes/svrroot/user/emittiv/nc/__groupfolders/1/01 Projects/`
2. Reads each pricing Excel file using `mcp__excel__read_data_from_excel`
3. Extracts pricing data per the mapping in the design doc
4. Writes fee records to SurrealDB using `mcp__SurrealDB__create` or `mcp__SurrealDB__upsert`
5. Generates validation report

**Agent prompt template:**

```
You are a data import agent for E-Fees. Your job is to scan project folders and extract pricing data from Excel files into SurrealDB.

## Connection
- SurrealDB: ws://10.0.21.8:8000, namespace: emittiv, database: projects, auth: martin/th38ret3ch

## Source Folders
Base: /Volumes/svrroot/user/emittiv/nc/__groupfolders/1/01 Projects/
Scan: 00 Inactive/, 01 RFPs/, 11 Current/, 99 Completed/
Target files: */02 Proposal/*.xlsx matching pricing patterns

## Filename Patterns (in priority order)
1. yy-cccnn-FP-NN Pricing.xlsx (standard)
2. e-yy-cccnn-FP-NN Pricing.xlsx (older e- prefix)
3. Pricing.xlsx (bare)
4. yy-cccnn Pricing.xlsx or yy-cccnn Pricing rN.xlsx
5. e-yy-cccnn-XX-FP-NN Pricing.xlsx (sub-packages)

## Skip Rules
- Skip if B2 AND all of C2:H2 are 0/null/empty (unfilled template)
- Skip vendor quotes, client schedules, deployment/expense files

## Excel Template Structure
- B2: Project target (base fee)
- C1:H1: Discipline names (Lighting, Video, Audio, SFX, Show Control, Sub)
- C2:H2: Discipline allocation multipliers
- P3: VAT rate
- P4: Mobilisation percentage
- Stage count N = COUNT(A:A) from P2
- Rows 9 to 8+N: Design stage rows (A=number, B=name, C-H=amounts per discipline)
- M9:M{8+N}: Stage percentages
- R9:R{8+N}: Cost values per stage
- Subtotal row = 9+N, VAT = 10+N, Grand = 11+N
- Post-contract starts at row 14+N

## Data Mapping
Map to E-Fees fee record with pricing field structured as PricingBreakdown.

## Source Tracking
Every imported record gets import_source: { file_path, file_modified, file_size, imported_at, import_version: "1.0", checksum: sha256, project_folder_status, filename_pattern }

## Idempotency
Match by project_id + rev. If exists, compare checksum. Skip if unchanged, update if changed.

## Output
Generate docs/import-validation-report.md with per-file validation tables.
```

**This task is run manually by the user when ready.** The import agent is a prompt for a future Claude Code session, not automated code.

---

### Task 4: Write agent prompt to file

**Files:**
- Create: `docs/agents/excel-import-agent.md`

**Step 1: Write the agent prompt**

Write the complete agent prompt (from Task 3) to `docs/agents/excel-import-agent.md` with full instructions, connection details, mapping rules, and validation report format.

**Step 2: Commit**

```bash
git add docs/agents/excel-import-agent.md
git commit -m "docs(import): add Excel import agent prompt and instructions"
```

---

## Workstream B: Excel Template Export

### Task 5: Write failing test for template export function

**Files:**
- Modify: `src-tauri/src/excel_export.rs:403-582` (add new tests)

**Step 1: Write the failing test**

Add to the `tests` module in `excel_export.rs`:

```rust
    #[test]
    fn test_generate_fee_template_creates_valid_xlsx() {
        let fee = full_fee();
        let path = std::env::temp_dir().join("delete_me_test_template.xlsx");
        let result = generate_fee_template(&fee, &path, 3);
        assert!(result.is_ok(), "generate_fee_template failed: {:?}", result.err());

        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "Generated file is empty");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_template_row_calculation_3_stages() {
        assert_eq!(template_subtotal_row(3), 12);
        assert_eq!(template_vat_row(3), 13);
        assert_eq!(template_grand_row(3), 14);
        assert_eq!(template_post_contract_start(3), 17);
    }

    #[test]
    fn test_template_row_calculation_4_stages() {
        assert_eq!(template_subtotal_row(4), 13);
        assert_eq!(template_vat_row(4), 14);
        assert_eq!(template_grand_row(4), 15);
        assert_eq!(template_post_contract_start(4), 18);
    }
```

**Step 2: Run test to verify it fails**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test test_generate_fee_template -- --nocapture`
Expected: FAIL — `generate_fee_template` not found

---

### Task 6: Implement template row helpers

**Files:**
- Modify: `src-tauri/src/excel_export.rs`

**Step 1: Add row calculation helpers**

Add after the format helpers section (~line 123):

```rust
// ============================================================================
// TEMPLATE ROW CALCULATIONS
// ============================================================================

/// Subtotal row in the pricing template.
/// Row 9+N where N = stage count (1-indexed, 0-indexed in xlsx = 8+N).
fn template_subtotal_row(stage_count: usize) -> u32 {
    (8 + stage_count + 1) as u32
}

fn template_vat_row(stage_count: usize) -> u32 {
    template_subtotal_row(stage_count) + 1
}

fn template_grand_row(stage_count: usize) -> u32 {
    template_vat_row(stage_count) + 1
}

fn template_post_contract_start(stage_count: usize) -> u32 {
    template_grand_row(stage_count) + 3
}
```

**Step 2: Run row calculation tests**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test test_template_row_calculation -- --nocapture`
Expected: PASS

**Step 3: Commit**

```bash
git add src-tauri/src/excel_export.rs
git commit -m "feat(export): add template row calculation helpers"
```

---

### Task 7: Implement `generate_fee_template` function

**Files:**
- Modify: `src-tauri/src/excel_export.rs`

**Step 1: Write the template generator**

Add after the row calculation helpers:

```rust
// ============================================================================
// TEMPLATE EXPORT FUNCTION
// ============================================================================

/// Generate a working pricing template Excel file from a Fee record.
///
/// This produces the discipline × stage matrix format used in project folders,
/// complete with formulas. Stage count determines row positions for subtotals,
/// VAT, and post-contract sections.
///
/// Returns the canonical path to the written file.
pub fn generate_fee_template(fee: &Fee, output_path: &Path, stage_count: usize) -> Result<String, String> {
    let pricing = fee.pricing.as_ref()
        .ok_or_else(|| "Fee has no pricing data".to_string())?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Pricing").map_err(|e| e.to_string())?;

    // Column widths: A=4, B=20, C-H=14 each, I=14 (Total), J-K=gap, L=14, M=8, N=gap, O=12, P=10, Q=gap, R=10
    worksheet.set_column_width(0, 4).map_err(|e| e.to_string())?;   // A
    worksheet.set_column_width(1, 20).map_err(|e| e.to_string())?;  // B
    for col in 2..=8 { // C-I
        worksheet.set_column_width(col, 14).map_err(|e| e.to_string())?;
    }
    worksheet.set_column_width(11, 14).map_err(|e| e.to_string())?; // L
    worksheet.set_column_width(12, 8).map_err(|e| e.to_string())?;  // M
    worksheet.set_column_width(14, 12).map_err(|e| e.to_string())?; // O
    worksheet.set_column_width(15, 10).map_err(|e| e.to_string())?; // P
    worksheet.set_column_width(17, 10).map_err(|e| e.to_string())?; // R

    let design_stages: Vec<&Stage> = pricing.stages.iter()
        .filter(|s| !s.is_post_contract)
        .collect();
    let actual_stage_count = design_stages.len().max(stage_count);

    // --- Row 1: Header ---
    // B1 = "Project Target", C1-H1 = discipline names, I1 = "Total"
    worksheet.write_string(0, 1, "Project Target").map_err(|e| e.to_string())?;
    for (i, disc) in pricing.disciplines.iter().enumerate() {
        if i < 6 {
            worksheet.write_string(0, (2 + i) as u16, &disc.name).map_err(|e| e.to_string())?;
        }
    }
    worksheet.write_string(0, 8, "Total").map_err(|e| e.to_string())?;

    // --- Row 2: Target + multipliers ---
    worksheet.write_number(1, 1, pricing.config.target_fee).map_err(|e| e.to_string())?;
    for (i, disc) in pricing.disciplines.iter().enumerate() {
        if i < 6 {
            worksheet.write_number(1, (2 + i) as u16, disc.percentage / 100.0).map_err(|e| e.to_string())?;
        }
    }
    // I2 = SUM formula
    worksheet.write_formula(1, 8, "=SUM(C2:H2)").map_err(|e| e.to_string())?;

    // --- Row 3: Target Price = B2 * C2 etc ---
    worksheet.write_string(2, 1, "Target Price").map_err(|e| e.to_string())?;
    for i in 0..6 {
        let col_letter = (b'C' + i as u8) as char;
        worksheet.write_formula(2, (2 + i) as u16, &format!("=$B$2*{}2", col_letter))
            .map_err(|e| e.to_string())?;
    }
    worksheet.write_formula(2, 8, "=SUM(C3:H3)").map_err(|e| e.to_string())?;

    // --- Row 6: Remaining = target - subtotal ---
    let sub_row_1indexed = template_subtotal_row(actual_stage_count);
    worksheet.write_string(5, 1, "Remaining").map_err(|e| e.to_string())?;
    for i in 0..6 {
        let col_letter = (b'C' + i as u8) as char;
        worksheet.write_formula(5, (2 + i) as u16,
            &format!("={col}3-{col}{sub}", col = col_letter, sub = sub_row_1indexed))
            .map_err(|e| e.to_string())?;
    }
    worksheet.write_formula(5, 8, "=SUM(C6:H6)").map_err(|e| e.to_string())?;

    // --- Row 8: Section header ---
    worksheet.write_string(7, 1, "Design Stages").map_err(|e| e.to_string())?;
    worksheet.write_string(7, 11, "Rec Fee").map_err(|e| e.to_string())?; // L8
    worksheet.write_string(7, 12, "%").map_err(|e| e.to_string())?;       // M8
    worksheet.write_string(7, 14, "Mobilisation").map_err(|e| e.to_string())?; // O8
    worksheet.write_string(7, 17, "Costs").map_err(|e| e.to_string())?;   // R8

    // --- Config section (O2:P4) ---
    worksheet.write_string(1, 14, "Stages").map_err(|e| e.to_string())?;
    worksheet.write_number(1, 15, actual_stage_count as f64).map_err(|e| e.to_string())?;
    worksheet.write_string(2, 14, "VAT").map_err(|e| e.to_string())?;
    worksheet.write_number(2, 15, pricing.config.vat_percent / 100.0).map_err(|e| e.to_string())?;
    worksheet.write_string(3, 14, "Mobilisation").map_err(|e| e.to_string())?;
    worksheet.write_number(3, 15, pricing.config.mobilisation_percent / 100.0).map_err(|e| e.to_string())?;

    // --- P8: Mobilisation calculation ---
    worksheet.write_formula(7, 15,
        &format!("=I{}*P4", sub_row_1indexed))
        .map_err(|e| e.to_string())?;

    // --- Rows 9 to 8+N: Design stage rows ---
    for (idx, stage) in design_stages.iter().enumerate() {
        let row = (8 + idx) as u32;
        // A: stage number
        worksheet.write_number(row, 0, (idx + 1) as f64).map_err(|e| e.to_string())?;
        // B: stage name
        worksheet.write_string(row, 1, &stage.name).map_err(|e| e.to_string())?;
        // C-H: discipline amounts
        for (di, disc) in pricing.disciplines.iter().enumerate() {
            if di < 6 {
                let amount = pricing.cells.iter()
                    .find(|c| c.discipline_id == disc.id && c.stage_id == stage.id)
                    .map(|c| c.override_amount.unwrap_or(c.amount))
                    .unwrap_or(0.0);
                worksheet.write_number(row, (2 + di) as u16, amount).map_err(|e| e.to_string())?;
            }
        }
        // I: Total formula
        worksheet.write_formula(row, 8, &format!("=SUM(C{}:H{})", row + 1, row + 1))
            .map_err(|e| e.to_string())?;
        // L: Rec fee formula
        worksheet.write_formula(row, 11, &format!("=$L$6*M{}", row + 1))
            .map_err(|e| e.to_string())?;
        // M: Stage percentage
        worksheet.write_number(row, 12, stage.percentage / 100.0).map_err(|e| e.to_string())?;
        // P: Net fee after mobilisation
        worksheet.write_formula(row, 15, &format!("=I{}-($P$8/$P$2)", row + 1))
            .map_err(|e| e.to_string())?;
        // R: Costs (from reimbursable costs matching this stage)
        let stage_costs: f64 = pricing.costs.iter()
            .filter(|c| c.stage_id == stage.id)
            .map(|c| c.cost_to_client)
            .sum();
        worksheet.write_number(row, 17, stage_costs).map_err(|e| e.to_string())?;
    }

    // --- Subtotal row ---
    let sub_row = sub_row_1indexed - 1; // 0-indexed
    worksheet.write_string(sub_row, 1, "Sub-Total").map_err(|e| e.to_string())?;
    for i in 0..7 { // C-I
        let col_letter = if i < 6 { (b'C' + i as u8) as char } else { 'I' };
        worksheet.write_formula(sub_row, (2 + i) as u16,
            &format!("=SUM({col}9:{col}{end})", col = col_letter, end = sub_row_1indexed - 1))
            .map_err(|e| e.to_string())?;
    }

    // --- VAT row ---
    let vat_row = template_vat_row(actual_stage_count) - 1;
    worksheet.write_string(vat_row, 1, "VAT").map_err(|e| e.to_string())?;
    worksheet.write_formula(vat_row, 8, &format!("=I{}*P3", sub_row_1indexed))
        .map_err(|e| e.to_string())?;

    // --- Grand Total row ---
    let grand_row = template_grand_row(actual_stage_count) - 1;
    worksheet.write_string(grand_row, 1, "Grand Total").map_err(|e| e.to_string())?;
    worksheet.write_formula(grand_row, 8,
        &format!("=SUM(I{}:I{})", sub_row_1indexed, template_vat_row(actual_stage_count)))
        .map_err(|e| e.to_string())?;

    // --- Post-contract items ---
    if let Some(ref items) = fee.post_contract_items {
        if !items.is_empty() {
            let pc_start = template_post_contract_start(actual_stage_count) - 1;
            worksheet.write_string(pc_start, 1, "Post Contract").map_err(|e| e.to_string())?;

            for (idx, item) in items.iter().enumerate() {
                let row = pc_start + 1 + idx as u32;
                worksheet.write_string(row, 1, &item.description).map_err(|e| e.to_string())?;
                worksheet.write_number(row, 2, item.quantity).map_err(|e| e.to_string())?;
                worksheet.write_number(row, 3, item.rate).map_err(|e| e.to_string())?;
                worksheet.write_formula(row, 4, &format!("=C{}*D{}", row + 1, row + 1))
                    .map_err(|e| e.to_string())?;
            }

            // Post-contract subtotal, VAT, grand total
            let pc_sub_row = pc_start + 1 + items.len() as u32;
            worksheet.write_string(pc_sub_row, 1, "Sub-Total").map_err(|e| e.to_string())?;
            worksheet.write_formula(pc_sub_row, 4,
                &format!("=SUM(E{}:E{})", pc_start + 2, pc_sub_row))
                .map_err(|e| e.to_string())?;

            let pc_vat_row = pc_sub_row + 1;
            worksheet.write_string(pc_vat_row, 1, "VAT").map_err(|e| e.to_string())?;
            worksheet.write_formula(pc_vat_row, 4, &format!("=E{}*P3", pc_sub_row + 1))
                .map_err(|e| e.to_string())?;

            let pc_grand_row = pc_vat_row + 1;
            worksheet.write_string(pc_grand_row, 1, "Grand Total").map_err(|e| e.to_string())?;
            worksheet.write_formula(pc_grand_row, 4,
                &format!("=SUM(E{}:E{})", pc_sub_row + 1, pc_vat_row + 1))
                .map_err(|e| e.to_string())?;
        }
    }

    // L6: Recommended fee (total from pricing config)
    worksheet.write_number(5, 11, pricing.config.quoted_fee).map_err(|e| e.to_string())?;

    workbook.save(output_path).map_err(|e| format!("Failed to save template: {}", e))?;
    Ok(output_path.to_string_lossy().to_string())
}
```

**Step 2: Run all template tests**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test test_generate_fee_template -- --nocapture`
Expected: PASS

Run: `cd /Volumes/base/dev/app/e-fees && cargo test`
Expected: All tests pass

**Step 3: Commit**

```bash
git add src-tauri/src/excel_export.rs
git commit -m "feat(export): add generate_fee_template for project folder Excel format"
```

---

### Task 8: Add Tauri command for template export

**Files:**
- Modify: `src-tauri/src/commands/export.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (command registration)

**Step 1: Read current export.rs**

Read `src-tauri/src/commands/export.rs` to understand existing patterns.

**Step 2: Add the export command**

Add a new Tauri command `export_fee_template` that:
1. Fetches the fee by ID
2. Reads project_folder_path from settings
3. Finds the project folder on the SMB share
4. Calculates the output filename: `{project_number}-FP-{rev:02} Pricing.xlsx`
5. Calls `generate_fee_template`
6. Returns the path where the file was saved

**Step 3: Register the command**

Add `commands::export::export_fee_template` to the invoke handler in `src-tauri/src/lib.rs`.

**Step 4: Run compilation check**

Run: `cd /Volumes/base/dev/app/e-fees && cargo check`
Expected: Compiles

**Step 5: Commit**

```bash
git add src-tauri/src/commands/export.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(export): add export_fee_template Tauri command for project folder export"
```

---

### Task 9: Add export button to proposal detail UI

**Files:**
- Modify: `src/lib/components/ProposalDetail.svelte` (add export button)
- Create: `src/lib/api/templateExport.ts` (frontend wrapper)

**Step 1: Create the TypeScript API wrapper**

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface ExportResult {
  path: string;
}

export async function exportFeeTemplate(feeId: string): Promise<ExportResult> {
  const path = await invoke<string>('export_fee_template', { feeId });
  return { path };
}
```

**Step 2: Add export button to ProposalDetail**

Add a "Export to Project Folder" button next to the existing export controls. On click, call `exportFeeTemplate(fee.id)` and show a success/error toast.

**Step 3: Verify in dev mode**

Run: `npm run tauri:dev`
Verify the button appears and clicking it generates a file (requires SMB share access).

**Step 4: Commit**

```bash
git add src/lib/api/templateExport.ts src/lib/components/ProposalDetail.svelte
git commit -m "feat(export): add 'Export to Project Folder' button on proposal detail"
```

---

## Workstream C: Fee Version Management

### Task 10: Add "New Revision" Tauri command

**Files:**
- Modify: `src-tauri/src/commands/fees.rs`
- Modify: `src-tauri/src/db/operations.rs`
- Modify: `src-tauri/src/lib.rs` (command registration)

**Step 1: Add clone_fee_as_revision to operations.rs**

```rust
    pub async fn clone_fee_as_revision(&self, source_fee_id: &str) -> Result<Fee, Error> {
        let client = self.get_client()?;
        // 1. Fetch the source fee
        let source: Option<Fee> = self.get_by_id("fee", source_fee_id).await?;
        let source = source.ok_or_else(|| self.not_found_error("source fee"))?;

        // 2. Find max rev for this project
        let max_rev: Option<i32> = client.query(
            "SELECT math::max(rev) AS max_rev FROM fee WHERE project_id = $pid"
        )
        .bind(("pid", source.project_id.clone()))
        .await?
        .take::<Option<i32>>("max_rev")?;

        let new_rev = max_rev.unwrap_or(0) + 1;

        // 3. Create new fee with incremented rev
        let new_fee = FeeCreate {
            name: source.name.clone(),
            number: source.number.clone(),
            rev: new_rev,
            status: "Draft".to_string(),
            issue_date: chrono::Utc::now().format("%y%m%d").to_string(),
            activity: source.activity.clone(),
            package: source.package.clone(),
            project_id: source.project_id.id.to_string(),
            company_id: source.company_id.id.to_string(),
            contact_id: source.contact_id.id.to_string(),
            staff_name: source.staff_name.clone(),
            staff_email: source.staff_email.clone(),
            staff_phone: source.staff_phone.clone(),
            staff_position: source.staff_position.clone(),
            strap_line: source.strap_line.clone(),
            revisions: vec![],
            pricing: source.pricing.clone(),
            post_contract_items: source.post_contract_items.clone(),
            reimbursable_costs: source.reimbursable_costs.clone(),
            payment_schedule: None, // New revision starts with clean payment schedule
            pricing_revisions: None,
            current_revision_number: None,
            current_release_number: None,
            import_source: None,
        };

        self.create_fee(new_fee).await
    }
```

**Step 2: Add Tauri command in fees.rs**

```rust
#[tauri::command]
pub async fn clone_fee_revision(fee_id: String, state: State<'_, AppState>) -> Result<Fee, String> {
    execute_with_manager(
        &state,
        |manager| {
            let id = fee_id.clone();
            Box::pin(async move {
                manager.clone_fee_as_revision(&id).await
            })
        },
        "clone revision",
        &format!("fee '{}'", fee_id)
    ).await
}
```

**Step 3: Register the command in lib.rs**

Add `commands::fees::clone_fee_revision` to the invoke handler.

**Step 4: Verify compilation**

Run: `cd /Volumes/base/dev/app/e-fees && cargo check`
Expected: Compiles

**Step 5: Commit**

```bash
git add src-tauri/src/commands/fees.rs src-tauri/src/db/operations.rs src-tauri/src/lib.rs
git commit -m "feat(versioning): add clone_fee_revision Tauri command"
```

---

### Task 11: Add "get revisions for project" query

**Files:**
- Modify: `src-tauri/src/db/operations.rs`
- Modify: `src-tauri/src/commands/fees.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add query to operations.rs**

```rust
    pub async fn get_fees_for_project(&self, project_id: &str) -> Result<Vec<Fee>, Error> {
        let client = self.get_client()?;
        let fees: Vec<Fee> = client.query(
            "SELECT * FROM fee WHERE project_id = type::thing('projects', $pid) ORDER BY rev DESC"
        )
        .bind(("pid", project_id))
        .await?
        .take(0)?;
        Ok(fees)
    }
```

**Step 2: Add Tauri command**

```rust
#[tauri::command]
pub async fn get_fees_for_project(project_id: String, state: State<'_, AppState>) -> Result<Vec<Fee>, String> {
    execute_with_manager(
        &state,
        |manager| {
            let pid = project_id.clone();
            Box::pin(async move {
                manager.get_fees_for_project(&pid).await
            })
        },
        "fetch",
        "fees for project"
    ).await
}
```

**Step 3: Register in lib.rs**

**Step 4: Verify compilation and commit**

```bash
cargo check
git add src-tauri/src/commands/fees.rs src-tauri/src/db/operations.rs src-tauri/src/lib.rs
git commit -m "feat(versioning): add get_fees_for_project query for revision history"
```

---

### Task 12: Add revision UI to ProposalDetail

**Files:**
- Modify: `src/lib/components/ProposalDetail.svelte`
- Create: `src/lib/api/revisions.ts`

**Step 1: Create revisions API wrapper**

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Fee } from '../../types';

export async function cloneFeeRevision(feeId: string): Promise<Fee> {
  return await invoke<Fee>('clone_fee_revision', { feeId });
}

export async function getFeesForProject(projectId: string): Promise<Fee[]> {
  return await invoke<Fee[]>('get_fees_for_project', { projectId });
}
```

**Step 2: Add revision dropdown to ProposalDetail**

Add a revision selector showing all revisions for the same project:
- Current revision highlighted
- Click to navigate to different revision
- "New Revision" button that calls `cloneFeeRevision` and navigates to the new fee

**Step 3: Test in dev mode**

Run: `npm run tauri:dev`
Navigate to a proposal, verify revision dropdown appears, create a new revision.

**Step 4: Commit**

```bash
git add src/lib/api/revisions.ts src/lib/components/ProposalDetail.svelte
git commit -m "feat(versioning): add revision dropdown and 'New Revision' button to proposal detail"
```

---

### Task 13: Add "latest only" filter to Proposals list

**Files:**
- Modify: `src/routes/Proposals.svelte`

**Step 1: Add toggle state**

Add a `showAllRevisions` toggle (default: false). When false, filter the fees list to show only the latest revision per project (highest `rev` value per `project_id`).

**Step 2: Add filter logic**

```typescript
function filterToLatestRevisions(fees: Fee[]): Fee[] {
  const latestByProject = new Map<string, Fee>();
  for (const fee of fees) {
    const pid = extractId(fee.project_id);
    const existing = latestByProject.get(pid);
    if (!existing || (fee.rev ?? 0) > (existing.rev ?? 0)) {
      latestByProject.set(pid, fee);
    }
  }
  return Array.from(latestByProject.values());
}
```

**Step 3: Add toggle UI**

Add a small toggle/checkbox near the filter controls: "Show all revisions" with a count badge showing how many projects have multiple revisions.

**Step 4: Test and commit**

```bash
git add src/routes/Proposals.svelte
git commit -m "feat(versioning): add 'Show all revisions' toggle to proposals list"
```

---

## Final Integration

### Task 14: Run full test suite and verify

**Step 1: Run Rust tests**

Run: `cd /Volumes/base/dev/app/e-fees && cargo test`
Expected: All tests pass

**Step 2: Run frontend checks**

Run: `cd /Volumes/base/dev/app/e-fees && npm run check`
Expected: No type errors

**Step 3: Build production**

Run: `cd /Volumes/base/dev/app/e-fees && npm run tauri:build`
Expected: Build succeeds

**Step 4: Commit any remaining changes**

```bash
git add -A
git commit -m "chore: integration verification for excel pricing and versioning features"
```

---

## Dependency Graph

```
Task 1 (import_source field) → Task 4 (agent prompt)
                              → Task 5-7 (template export)
                              → Task 10-11 (versioning backend)

Task 5-7 (template export) → Task 8 (Tauri command) → Task 9 (export button)

Task 10-11 (versioning backend) → Task 12 (revision UI) → Task 13 (list filter)

All → Task 14 (integration verification)
```

Tasks 1, 5-7, and 10-11 can run in parallel after Task 1.
Workstream A (Tasks 2-4) is fully independent.
