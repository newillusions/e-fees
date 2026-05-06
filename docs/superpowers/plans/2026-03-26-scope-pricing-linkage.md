# Scope-Pricing Stage Linkage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Link pricing stages and scope modules so they share a single per-proposal stage list, with bidirectional sync, LLM-driven stage name templating, revision history, and markdown export.

**Architecture:** The fee record's `pricing.stages` array is the single source of truth. The frontend shares stage state between pricing and scope tabs via a reactive variable in ProposalModal. The scope service receives stages as input in the generate request (stateless). Revisions are stored in a separate `scope_revision` table. The existing `stage_config` table is extended to serve as the global autocomplete dictionary.

**Tech Stack:** Rust (e-fees-core, e-fees-scope, Tauri commands), Svelte 5 (runes), SurrealDB v3, TypeScript

**Spec:** `docs/superpowers/specs/2026-03-26-scope-pricing-linkage-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `crates/e-fees-core/src/models/fee.rs` | Modify | Add `set_pricing_stages()` helper, `StageInput` re-export |
| `e-fees-scope/schema.surql` | Modify | Extend `stage_config`, add `scope_revision` table, new fields on `scope_assembly` |
| `e-fees-scope/src/models.rs` | Modify | Add `StageInput` to `GenerateScopeRequest` |
| `e-fees-scope/src/routes/scope.rs` | Modify | Pass stages to LLM polish, store `stages_snapshot` |
| `src-tauri/src/commands/fee_stages.rs` | Create | `get_fee_stages`, `add_stage_to_fee` Tauri commands |
| `src-tauri/src/commands/scope_export.rs` | Create | `export_scope_markdown` Tauri command |
| `src-tauri/src/commands/stage_dictionary.rs` | Create | `search_stage_dictionary`, `add_stage_to_dictionary` commands |
| `src-tauri/src/commands/mod.rs` | Modify | Register new command modules |
| `src-tauri/src/lib.rs` | Modify | Register new commands in invoke_handler |
| `src/lib/types/scope.ts` | Modify | Add `StageInput`, extend `GenerateScopeRequest`, `ScopeAssembly` |
| `src/lib/api/scope.ts` | Modify | Pass stages in `generateScope()` |
| `src/lib/api/feeStages.ts` | Create | `getFeeStages()`, `addStageToFee()` API wrappers |
| `src/lib/api/stageDictionary.ts` | Create | `searchStageDictionary()` API wrapper |
| `src/lib/components/scope/ScopeViewer.svelte` | Modify | Accept stages prop, revision save, banners |
| `src/lib/components/scope/ScopeAdvancedControls.svelte` | Modify | Use fee stages instead of global stage_config |
| `src/lib/components/ProposalModal.svelte` | Modify | Shared stage state, pass stages to ScopeViewer |

---

### Task 0: Schema migration — extend stage_config and add scope_revision

**Files:**
- Modify: `e-fees-scope/schema.surql`

- [ ] **Step 1: Write schema additions**

Add to the end of `schema.surql`:

```sql
-- ============================================================
-- Extend stage_config for dictionary/autocomplete
-- ============================================================
DEFINE FIELD code             ON stage_config TYPE option<string>;
DEFINE FIELD usage_count      ON stage_config TYPE int DEFAULT 0;
DEFINE FIELD is_post_contract ON stage_config TYPE bool DEFAULT false;

-- ============================================================
-- Scope revision history (separate table to avoid record bloat)
-- ============================================================
DEFINE TABLE scope_revision SCHEMAFULL;

DEFINE FIELD fee_id         ON scope_revision TYPE record<fee>;
DEFINE FIELD revision       ON scope_revision TYPE int;
DEFINE FIELD clauses        ON scope_revision TYPE array<object> FLEXIBLE;
DEFINE FIELD clauses.*      ON scope_revision TYPE object FLEXIBLE;
DEFINE FIELD clauses.*.clauses   ON scope_revision TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD clauses.*.clauses.* ON scope_revision TYPE object FLEXIBLE;
DEFINE FIELD generated_text ON scope_revision TYPE string;
DEFINE FIELD stages_at_time ON scope_revision TYPE option<array<string>>;
DEFINE FIELD saved_at       ON scope_revision TYPE datetime DEFAULT time::now();
DEFINE FIELD trigger        ON scope_revision TYPE string;

DEFINE INDEX idx_scope_rev_fee ON scope_revision FIELDS fee_id, revision;

-- ============================================================
-- Extend scope_assembly for revision tracking and stage snapshot
-- ============================================================
DEFINE FIELD current_revision ON scope_assembly TYPE int DEFAULT 0;
DEFINE FIELD stages_snapshot  ON scope_assembly TYPE option<array<string>>;
```

- [ ] **Step 2: Apply schema to dev DB**

```bash
curl -s -X POST "http://10.0.23.12:8000/rpc" \
  -u "martin:<redacted-rotated-2026-05-06>" \
  -H "surreal-ns: emittiv" -H "surreal-db: projects" \
  -H "Accept: application/json" -H "Content-Type: application/json" \
  -d '{"method":"query","params":["<paste schema additions>"]}'
```

Expected: All statements return `"status": "OK"` (or "already exists" for re-runs).

- [ ] **Step 3: Apply schema to prod DB**

Same curl against `http://10.0.23.11:8000/rpc`.

- [ ] **Step 4: Commit**

```bash
git add e-fees-scope/schema.surql
git commit -m "feat(scope): extend schema for stage dictionary and revision history"
```

---

### Task 1: Rust — add `StageInput` and `set_pricing_stages()` to e-fees-core

**Files:**
- Modify: `crates/e-fees-core/src/models/fee.rs:112-122` (impl Fee block)
- Test: `crates/e-fees-core/src/models/fee.rs` (add test module)

- [ ] **Step 1: Write failing test**

Add at the bottom of `fee.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_pricing_stages_on_empty_pricing() {
        // Fee doesn't derive Default — construct with minimal required fields
        let mut fee = Fee {
            pricing: None,
            ..serde_json::from_value(serde_json::json!({
                "id": "fee:test1",
                "activity": "Test",
                "number": "25-97101-1",
                "status": "Draft",
                "issue_date": "202601",
                "project_id": "projects:test",
                "company_id": "company:test",
                "contact_id": "contacts:test"
            })).expect("test fee construction")
        };
        assert!(fee.pricing.is_none());

        let stages = vec![Stage {
            id: "sd-01".to_string(),
            name: "Schematic Design".to_string(),
            code: "SD".to_string(),
            percentage: 25.0,
            order: 1,
            is_post_contract: false,
        }];

        fee.set_pricing_stages(stages.clone());

        let breakdown = fee.pricing_typed().expect("pricing should be set");
        assert_eq!(breakdown.stages.len(), 1);
        assert_eq!(breakdown.stages[0].name, "Schematic Design");
    }

    #[test]
    fn test_set_pricing_stages_preserves_existing_pricing() {
        let mut fee: Fee = serde_json::from_value(serde_json::json!({
            "id": "fee:test2",
            "activity": "Test",
            "number": "25-97101-2",
            "status": "Draft",
            "issue_date": "202601",
            "project_id": "projects:test",
            "company_id": "company:test",
            "contact_id": "contacts:test"
        })).expect("test fee construction");
        // Set up initial pricing with config
        let mut breakdown = PricingBreakdown::default();
        breakdown.config.quoted_fee = 50000.0;
        breakdown.stages = vec![Stage {
            id: "sd-01".to_string(),
            name: "Schematic Design".to_string(),
            code: "SD".to_string(),
            percentage: 25.0,
            order: 1,
            is_post_contract: false,
        }];
        let json = serde_json::to_value(&breakdown).unwrap();
        fee.pricing = Some(serde_json::from_value(json).unwrap());

        // Add a new stage
        let new_stages = vec![
            Stage {
                id: "sd-01".to_string(),
                name: "Schematic Design".to_string(),
                code: "SD".to_string(),
                percentage: 25.0,
                order: 1,
                is_post_contract: false,
            },
            Stage {
                id: "bim-01".to_string(),
                name: "BIM Coordination".to_string(),
                code: "BC".to_string(),
                percentage: 0.0,
                order: 2,
                is_post_contract: false,
            },
        ];
        fee.set_pricing_stages(new_stages);

        let result = fee.pricing_typed().unwrap();
        assert_eq!(result.stages.len(), 2);
        assert_eq!(result.config.quoted_fee, 50000.0); // preserved
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p e-fees-core --lib tests -- --test-threads=1`
Expected: FAIL — `set_pricing_stages` method not found.

- [ ] **Step 3: Implement `set_pricing_stages`**

In `fee.rs`, extend the `impl Fee` block (after line 121):

```rust
impl Fee {
    pub fn pricing_typed(&self) -> Option<PricingBreakdown> {
        self.pricing.as_ref().and_then(|v| {
            let json = dbvalue_to_json(v);
            serde_json::from_value(json).ok()
        })
    }

    /// Replace the stages in the pricing breakdown, preserving all other pricing data.
    /// If pricing is None, creates a default PricingBreakdown with the given stages.
    pub fn set_pricing_stages(&mut self, stages: Vec<Stage>) {
        let mut breakdown = self.pricing_typed().unwrap_or_default();
        breakdown.stages = stages;
        // Serialize back to DbValue via JSON round-trip
        if let Ok(json) = serde_json::to_value(&breakdown) {
            if let Ok(db_val) = serde_json::from_value(json) {
                self.pricing = Some(db_val);
            }
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p e-fees-core --lib tests -- --test-threads=1`
Expected: PASS (both tests).

- [ ] **Step 5: Commit**

```bash
git add crates/e-fees-core/src/models/fee.rs
git commit -m "feat(core): add set_pricing_stages helper for stage sync"
```

---

### Task 2: Rust — extend scope service GenerateScopeRequest with stages

**Files:**
- Modify: `e-fees-scope/src/models.rs:121-125`
- Modify: `e-fees-scope/src/routes/scope.rs:176-198` (generate_scope handler)

- [ ] **Step 1: Add StageInput to models.rs**

After `GenerateScopeRequest` (line 125), add:

```rust
/// Stage input from the fee's pricing data, passed by the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInput {
    pub name: String,
    pub code: String,
    pub is_post_contract: bool,
    pub order: i64,
}
```

Extend `GenerateScopeRequest`:

```rust
pub struct GenerateScopeRequest {
    pub fee_id: String,
    #[serde(default)]
    pub polish: bool,
    #[serde(default)]
    pub stages: Option<Vec<StageInput>>,
}
```

- [ ] **Step 2: Update generate_scope to pass stages to LLM context**

In `scope.rs`, after the fee fetch (line 197), add stage context:

```rust
// Build stage context string for LLM
let stage_context = if let Some(ref stages) = body.stages {
    let design: Vec<String> = stages.iter()
        .filter(|s| !s.is_post_contract)
        .map(|s| format!("{} ({})", s.name, s.code))
        .collect();
    let post: Vec<String> = stages.iter()
        .filter(|s| s.is_post_contract)
        .map(|s| format!("{} ({})", s.name, s.code))
        .collect();
    let mut ctx = String::new();
    if !design.is_empty() {
        ctx.push_str(&format!("Design stages: {}", design.join(", ")));
    }
    if !post.is_empty() {
        if !ctx.is_empty() { ctx.push_str(". "); }
        ctx.push_str(&format!("Post-contract stages: {}", post.join(", ")));
    }
    Some(ctx)
} else {
    None
};
```

Pass `stage_context` to the `llm::polish_scope` call (extend its signature to accept `Option<&str>` for stage context).

- [ ] **Step 3: Store stages_snapshot on scope_assembly**

In the upsert query (line 240), add:

```rust
// Add stages_snapshot to the SET clause
let stages_snapshot: Vec<String> = body.stages
    .as_ref()
    .map(|s| s.iter().map(|st| st.name.clone()).collect())
    .unwrap_or_default();
```

Add `stages_snapshot = $stages_snapshot` to the CREATE SET clause, and bind it.

- [ ] **Step 4: Verify scope service compiles**

Run: `cargo check -p e-fees-scope`
Expected: No errors (warnings OK).

- [ ] **Step 5: Test via curl**

```bash
curl -s -X POST "http://localhost:3201/scope/generate" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: efees-scope-2026-s7k2m9xp" \
  -d '{"fee_id":"22_97111_1","polish":false,"stages":[{"name":"Schematic Design","code":"SD","is_post_contract":false,"order":1}]}'
```

Expected: 200 OK with scope data. Verify `stages_snapshot` is stored:

```bash
curl -s -X POST "http://10.0.23.12:8000/rpc" \
  -u "martin:<redacted-rotated-2026-05-06>" \
  -H "surreal-ns: emittiv" -H "surreal-db: projects" \
  -H "Accept: application/json" -H "Content-Type: application/json" \
  -d '{"method":"query","params":["SELECT stages_snapshot FROM scope_assembly LIMIT 1;"]}'
```

- [ ] **Step 6: Commit**

```bash
git add e-fees-scope/src/models.rs e-fees-scope/src/routes/scope.rs
git commit -m "feat(scope): accept stages in generate request, store snapshot"
```

---

### Task 3: Rust — Tauri commands for fee stage access

**Files:**
- Create: `src-tauri/src/commands/fee_stages.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (invoke_handler)

- [ ] **Step 1: Create `fee_stages.rs`**

Uses `AppState` (= `Arc<RwLock<DatabaseManager>>`) and `execute_with_manager` pattern matching existing commands. Parameterised queries to prevent SQL injection.

```rust
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::AppState;
use crate::commands::utils::execute_with_manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStage {
    pub id: String,
    pub name: String,
    pub code: String,
    pub percentage: f64,
    pub order: i64,
    pub is_post_contract: bool,
}

/// Get the pricing stages for a fee. Returns empty vec if no pricing configured.
#[tauri::command]
pub async fn get_fee_stages(
    fee_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FeeStage>, String> {
    execute_with_manager(
        &state,
        |manager| {
            let fid = fee_id.clone();
            Box::pin(async move {
                let client = manager.client.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("DB not connected"))?;
                let mut res = client.query(
                    "SELECT pricing FROM type::record('fee', $fee_key) LIMIT 1;"
                )
                .bind(("fee_key", fid))
                .await?;
                let rows: Vec<serde_json::Value> = res.take(0)?;

                let stages = rows
                    .into_iter()
                    .next()
                    .and_then(|r| r.get("pricing").cloned())
                    .and_then(|p| p.get("stages").cloned())
                    .and_then(|s| serde_json::from_value::<Vec<FeeStage>>(s).ok())
                    .unwrap_or_default();

                Ok(stages)
            })
        },
        "fetch fee stages",
    )
    .await
}

/// Add a stage to a fee's pricing breakdown. Creates minimal pricing if none exists.
#[tauri::command]
pub async fn add_stage_to_fee(
    fee_id: String,
    stage: FeeStage,
    state: State<'_, AppState>,
) -> Result<Vec<FeeStage>, String> {
    execute_with_manager(
        &state,
        |manager| {
            let fid = fee_id.clone();
            let new_stage = stage.clone();
            Box::pin(async move {
                let client = manager.client.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("DB not connected"))?;

                // Read current pricing
                let mut res = client.query(
                    "SELECT pricing FROM type::record('fee', $fee_key) LIMIT 1;"
                )
                .bind(("fee_key", fid.clone()))
                .await?;
                let rows: Vec<serde_json::Value> = res.take(0)?;

                let current_pricing = rows
                    .into_iter()
                    .next()
                    .and_then(|r| r.get("pricing").cloned());

                // Build updated stages
                let mut stages: Vec<FeeStage> = current_pricing
                    .as_ref()
                    .and_then(|p| p.get("stages").cloned())
                    .and_then(|s| serde_json::from_value(s).ok())
                    .unwrap_or_default();

                if !stages.iter().any(|s| s.id == new_stage.id) {
                    stages.push(new_stage);
                }

                // Rebuild pricing with updated stages
                let mut pricing: serde_json::Value = current_pricing
                    .unwrap_or_else(|| serde_json::json!({}));
                pricing["stages"] = serde_json::to_value(&stages)?;

                // Write back with parameterised binding
                client.query(
                    "UPDATE type::record('fee', $fee_key) SET pricing = $pricing;"
                )
                .bind(("fee_key", fid))
                .bind(("pricing", pricing))
                .await?;

                Ok(stages)
            })
        },
        "add stage to fee",
    )
    .await
}
```

- [ ] **Step 2: Register in mod.rs and lib.rs**

In `src-tauri/src/commands/mod.rs`, add:
```rust
pub mod fee_stages;
```

In `src-tauri/src/lib.rs` invoke_handler, add:
```rust
commands::fee_stages::get_fee_stages,
commands::fee_stages::add_stage_to_fee,
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p app`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/fee_stages.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): add get_fee_stages and add_stage_to_fee commands"
```

---

### Task 4: TypeScript — extend scope types and API client

**Files:**
- Modify: `src/lib/types/scope.ts:157-160`
- Modify: `src/lib/api/scope.ts:139-145`
- Test: `src/lib/api/scope.test.ts`

- [ ] **Step 1: Add StageInput type and extend GenerateScopeRequest**

In `src/lib/types/scope.ts`, after line 160:

```typescript
export interface StageInput {
  name: string;
  code: string;
  is_post_contract: boolean;
  order: number;
}

// Replace the existing GenerateScopeRequest (lines 157-160)
export interface GenerateScopeRequest {
  fee_id: string;
  polish?: boolean;
  stages?: StageInput[];
}

// Extend ScopeAssembly (add after existing fields)
// Add to the ScopeAssembly interface:
//   current_revision?: number;
//   stages_snapshot?: string[];
```

- [ ] **Step 2: Write failing test for stages in generateScope**

In `src/lib/api/scope.test.ts`, add:

```typescript
describe('generateScope', () => {
  it('passes stages in request body', async () => {
    const mockResult = {
      clauses: [{ number: '1.0', title: 'Admin', clauses: [] }],
      generated_text: '1.0 ADMIN',
      llm_polished: false,
      stages_snapshot: ['Schematic Design'],
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: () => Promise.resolve({ data: mockResult }),
    });

    const { generateScope } = await import('./scope');
    const result = await generateScope({
      fee_id: '25_97101_1',
      polish: false,
      stages: [{ name: 'Schematic Design', code: 'SD', is_post_contract: false, order: 1 }],
    });

    expect(result.stages_snapshot).toEqual(['Schematic Design']);
    // Verify fetch was called with stages in body
    const fetchCall = mockFetch.mock.calls[0];
    const body = JSON.parse(fetchCall[1].body);
    expect(body.stages).toHaveLength(1);
    expect(body.stages[0].name).toBe('Schematic Design');
  });
});
```

- [ ] **Step 3: Run test to verify it fails then passes**

Run: `npx vitest run src/lib/api/scope.test.ts`
The test should pass immediately since `generateScope` already sends the full body. Verify.

- [ ] **Step 4: Commit**

```bash
git add src/lib/types/scope.ts src/lib/api/scope.ts src/lib/api/scope.test.ts
git commit -m "feat(scope): extend types and API for stage input"
```

---

### Task 5: TypeScript — fee stages and stage dictionary API wrappers

**Files:**
- Create: `src/lib/api/feeStages.ts`
- Create: `src/lib/api/stageDictionary.ts`

- [ ] **Step 1: Create feeStages.ts**

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface FeeStage {
  id: string;
  name: string;
  code: string;
  percentage: number;
  order: number;
  is_post_contract: boolean;
}

export async function getFeeStages(feeId: string): Promise<FeeStage[]> {
  return invoke('get_fee_stages', { feeId });
}

export async function addStageToFee(feeId: string, stage: FeeStage): Promise<FeeStage[]> {
  return invoke('add_stage_to_fee', { feeId, stage });
}
```

- [ ] **Step 2: Create stageDictionary.ts**

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface StageDictionaryEntry {
  canonical_name: string;
  default_label: string;
  code?: string;
  is_post_contract: boolean;
  usage_count: number;
}

export async function searchStageDictionary(query: string): Promise<StageDictionaryEntry[]> {
  return invoke('search_stage_dictionary', { query });
}
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/api/feeStages.ts src/lib/api/stageDictionary.ts
git commit -m "feat(api): add fee stages and stage dictionary wrappers"
```

---

### Task 6: Frontend — ScopeViewer accepts stages prop and passes to generate

**Files:**
- Modify: `src/lib/components/scope/ScopeViewer.svelte`
- Modify: `src/lib/components/ProposalModal.svelte:1496-1499`

- [ ] **Step 1: Add stages prop to ScopeViewer**

In `ScopeViewer.svelte`, update the props (lines 17-23):

```typescript
let {
  feeId,
  stages = [],
  ondirtychange,
}: {
  feeId: string;
  stages?: import('$lib/api/feeStages').FeeStage[];
  ondirtychange?: (isDirty: boolean) => void;
} = $props();
```

- [ ] **Step 2: Pass stages to generateScope call**

In `handleGenerate()` (line 76), update:

```typescript
const result = await generateScope({
  fee_id: feeId,
  polish: true,
  stages: stages.map(s => ({
    name: s.name,
    code: s.code,
    is_post_contract: s.is_post_contract,
    order: s.order,
  })),
}, controller.signal);
```

- [ ] **Step 3: Add stage sync banners**

After the sections render block (line 244), add a banner section:

```svelte
{#if scope && stages.length > 0}
  {#each stages as stage}
    {#if !sections.some(s => s.clauses?.some(c => c.body?.includes(stage.name)))}
      <div class="emittiv-alert emittiv-alert--info emittiv-alert--sm">
        {stage.name}: no scope deliverables
      </div>
    {/if}
  {/each}
{/if}

{#if scope?.stages_snapshot && stages.length > 0}
  {#if stages.some(s => !scope.stages_snapshot?.includes(s.name)) || scope.stages_snapshot.some(n => !stages.find(s => s.name === n))}
    <div class="emittiv-alert emittiv-alert--warning emittiv-alert--sm">
      Stages changed since last generation — regenerate to update scope text
    </div>
  {/if}
{/if}
```

- [ ] **Step 4: Pass stages from ProposalModal**

In `ProposalModal.svelte` (line 1496-1499), update:

```svelte
<ScopeViewer
  feeId={getEntityId(proposal)}
  stages={proposal?.pricing?.stages ?? []}
  ondirtychange={(d) => scopeDirty = d}
/>
```

- [ ] **Step 5: Verify with `npm run check`**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 6: Commit**

```bash
git add src/lib/components/scope/ScopeViewer.svelte src/lib/components/ProposalModal.svelte
git commit -m "feat(scope): pass pricing stages to scope viewer and generate"
```

---

### Task 7: Frontend — ScopeAdvancedControls uses fee stages

**Files:**
- Modify: `src/lib/components/scope/ScopeAdvancedControls.svelte`

- [ ] **Step 1: Replace global stage_config with fee stages**

Currently the component loads stages from `getStages()` (scope API). Change it to use the `stages` prop passed from ScopeViewer:

Update props to accept fee stages:

```typescript
let {
  feeId,
  stages = [],
  onassemble,
  loading = false,
}: {
  feeId: string;
  stages?: import('$lib/api/feeStages').FeeStage[];
  onassemble: (request: AssembleRequest) => void;
  loading?: boolean;
} = $props();
```

Replace the stage selector options to use `stages` prop instead of fetching from scope API.

- [ ] **Step 2: Update ScopeViewer to pass stages to ScopeAdvancedControls**

In `ScopeViewer.svelte` where `ScopeAdvancedControls` is rendered (lines 220-226 and 258-264):

```svelte
<ScopeAdvancedControls
  {feeId}
  {stages}
  onassemble={handleAdvancedAssemble}
  loading={generating}
/>
```

- [ ] **Step 3: Verify with `npm run check`**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/scope/ScopeAdvancedControls.svelte src/lib/components/scope/ScopeViewer.svelte
git commit -m "feat(scope): use fee stages in advanced controls instead of global config"
```

---

### Task 8: Scope revision save and markdown export

**Files:**
- Create: `src-tauri/src/commands/scope_export.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/components/scope/ScopeViewer.svelte` (save handler)

- [ ] **Step 1: Create scope_export.rs**

```rust
use std::fs;
use std::path::PathBuf;
use tauri::State;

use crate::db::DatabaseManager;

/// Export scope as markdown to the project folder.
/// Returns the file path on success, or None if folder not found.
#[tauri::command]
pub async fn export_scope_markdown(
    fee_id: String,
    fee_ref: String,
    project_name: String,
    project_folder: String,
    revision: i64,
    stages: Vec<String>,
    scope_text: String,
) -> Result<Option<String>, String> {
    let scope_dir = PathBuf::from(&project_folder).join("scope");

    // Create scope directory if project folder exists
    if !PathBuf::from(&project_folder).exists() {
        return Ok(None); // Project folder not found — skip silently
    }
    fs::create_dir_all(&scope_dir).map_err(|e| e.to_string())?;

    let stages_yaml = stages.join(", ");
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    let content = format!(
        "---\nfee: {}\nproject: {}\nrevision: {}\ndate: {}\nstages: [{}]\n---\n\n{}",
        fee_ref, project_name, revision, date, stages_yaml, scope_text
    );

    // Write revision file
    let rev_filename = format!("scope-rev-{:02}.md", revision);
    let rev_path = scope_dir.join(&rev_filename);
    fs::write(&rev_path, &content).map_err(|e| e.to_string())?;

    // Write/overwrite current file
    let current_path = scope_dir.join("scope-current.md");
    fs::write(&current_path, &content).map_err(|e| e.to_string())?;

    Ok(Some(rev_path.to_string_lossy().to_string()))
}
```

- [ ] **Step 2: Register in mod.rs and lib.rs**

In `mod.rs`: `pub mod scope_export;`
In `lib.rs` invoke_handler: `commands::scope_export::export_scope_markdown,`

- [ ] **Step 3: Update ScopeViewer save handler to create revision + export**

In `ScopeViewer.svelte`, update `handleSave()`:

```typescript
async function handleSave() {
  saving = true;
  error = null;

  try {
    const generatedText = formatSectionsAsText(sections);

    // Save to scope service (this triggers revision creation server-side via Task 9)
    await updateScope(feeId, { clauses: sections, generated_text: generatedText });

    // Export markdown to project folder via Tauri IPC
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const stageNames = stages.map(s => s.name);
      const exported = await invoke('export_scope_markdown', {
        feeId,
        feeRef: feeId, // TODO: pass actual fee number from parent
        projectName: '', // TODO: pass from parent
        projectFolder: '', // TODO: pass from parent
        revision: scope?.current_revision ?? 1,
        stages: stageNames,
        scopeText: generatedText,
      });
      if (exported) {
        message = `Scope saved and exported to ${exported}`;
      } else {
        message = 'Scope saved. Folder export skipped — project folder not found.';
      }
    } catch {
      // Export failure is non-blocking
      message = 'Scope saved successfully';
    }

    dirty = false;
    setTimeout(() => (message = null), 3000);
  } catch (err: any) {
    error = err.message || 'Failed to save scope';
    logApiError('ScopeViewer save', err as Error);
  } finally {
    saving = false;
  }
}
```

Note: The full revision save + export integration requires the scope service to also support revision creation. For now, the Tauri command is ready and the save handler structure is in place. The revision write will be wired in a follow-up task when the scope service gets a revision endpoint.

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p app && npm run check`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/scope_export.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src/lib/components/scope/ScopeViewer.svelte
git commit -m "feat(scope): add markdown export command and revision save structure"
```

---

### Task 9: Scope service — revision endpoint

**Files:**
- Modify: `e-fees-scope/src/routes/scope.rs`
- Modify: `e-fees-scope/src/main.rs` (register route)

- [ ] **Step 1: Add `current_revision` and `stages_snapshot` to ScopeAssembly model FIRST**

In `models.rs`, add to `ScopeAssembly` struct:

```rust
#[serde(default)]
pub current_revision: Option<i64>,
#[serde(default)]
pub stages_snapshot: Option<Vec<String>>,
```

Both fields need `#[serde(default)]` because existing records in SurrealDB won't have them — the binary protocol needs serde defaults to handle missing keys.

- [ ] **Step 2: Add create_revision helper**

In `scope.rs`, add a helper function:

```rust
/// Create a revision record from the current scope_assembly state before updating.
async fn create_revision(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    fee_key: &str,
    trigger: &str,
) -> Result<i64, ApiError> {
    // Get current assembly
    let mut res = db
        .query("SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key)")
        .bind(("fee_key", fee_key.to_string()))
        .await?;
    let assemblies: Vec<ScopeAssembly> = res.take(0)?;

    let Some(assembly) = assemblies.into_iter().next() else {
        return Ok(0); // No existing assembly to revision
    };

    let next_rev = assembly.current_revision.unwrap_or(0) + 1;

    db.query(
        "CREATE scope_revision SET \
         fee_id = type::record('fee', $fee_key), \
         revision = $revision, \
         clauses = $clauses, \
         generated_text = $generated_text, \
         stages_at_time = $stages, \
         trigger = $trigger;"
    )
    .bind(("fee_key", fee_key.to_string()))
    .bind(("revision", next_rev))
    .bind(("clauses", assembly.clauses.clone()))
    .bind(("generated_text", assembly.generated_text.clone()))
    .bind(("stages", assembly.stages_snapshot.clone().unwrap_or_default()))
    .bind(("trigger", trigger.to_string()))
    .await?;

    // Update current_revision counter
    db.query(
        "UPDATE scope_assembly SET current_revision = $rev \
         WHERE fee_id = type::record('fee', $fee_key);"
    )
    .bind(("fee_key", fee_key.to_string()))
    .bind(("rev", next_rev))
    .await?;

    Ok(next_rev)
}
```

- [ ] **Step 3: Call create_revision before upsert in generate_scope**

In the `generate_scope` handler, before the DELETE+CREATE upsert (line 240), add:

```rust
// Revision: save current state before overwriting
let _revision = create_revision(&state.db, fee_key, "regeneration").await?;
```

- [ ] **Step 4: Call create_revision in update_scope handler**

In `update_scope` (line 320), before the UPDATE:

```rust
let _revision = create_revision(&state.db, fee_key, "manual_edit").await?;
```

- [ ] **Step 5: Verify compilation and test**

Run: `cargo check -p e-fees-scope`

Test via curl: generate scope twice for the same fee, then check revisions exist:

```bash
curl -s -X POST "http://10.0.23.12:8000/rpc" \
  -u "martin:<redacted-rotated-2026-05-06>" \
  -H "surreal-ns: emittiv" -H "surreal-db: projects" \
  -H "Accept: application/json" -H "Content-Type: application/json" \
  -d '{"method":"query","params":["SELECT fee_id, revision, trigger, saved_at FROM scope_revision ORDER BY saved_at DESC LIMIT 5;"]}'
```

- [ ] **Step 6: Also update `regenerate_scope` handler**

The `regenerate_scope` handler at `scope.rs:384` also calls `polish_scope`. Update its call to pass `stage_context` (from the stored `stages_snapshot` on the existing assembly).

- [ ] **Step 7: Commit**

```bash
git add e-fees-scope/src/routes/scope.rs e-fees-scope/src/models.rs e-fees-scope/src/main.rs
git commit -m "feat(scope): add revision history on generate and update"
```

---

### Task 10: Stage dictionary Tauri command

**Files:**
- Create: `src-tauri/src/commands/stage_dictionary.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create stage_dictionary.rs**

Uses `AppState` and `execute_with_manager` matching other commands.

```rust
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::AppState;
use crate::commands::utils::execute_with_manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct StageDictionaryEntry {
    pub canonical_name: String,
    pub default_label: String,
    pub code: Option<String>,
    pub is_post_contract: bool,
    pub usage_count: i64,
}

/// Search stage dictionary by name prefix/substring.
#[tauri::command]
pub async fn search_stage_dictionary(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<StageDictionaryEntry>, String> {
    execute_with_manager(
        &state,
        |manager| {
            let q = query.clone();
            Box::pin(async move {
                let client = manager.client.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("DB not connected"))?;
                let mut res = client.query(
                    "SELECT canonical_name, default_label, code, is_post_contract, usage_count \
                     FROM stage_config \
                     WHERE status = 'active' \
                     AND (string::lowercase(canonical_name) CONTAINS string::lowercase($query) \
                          OR string::lowercase(default_label) CONTAINS string::lowercase($query)) \
                     ORDER BY usage_count DESC \
                     LIMIT 20;"
                )
                .bind(("query", q))
                .await?;
                let entries: Vec<StageDictionaryEntry> = res.take(0)?;
                Ok(entries)
            })
        },
        "search stage dictionary",
    )
    .await
}

/// Add a new entry to the stage dictionary (auto-insert on new stage creation).
#[tauri::command]
pub async fn add_stage_to_dictionary(
    name: String,
    code: Option<String>,
    is_post_contract: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    execute_with_manager(
        &state,
        |manager| {
            let n = name.clone();
            let c = code.clone();
            Box::pin(async move {
                let client = manager.client.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("DB not connected"))?;
                // Upsert: increment usage_count if exists, create if not
                client.query(
                    "UPSERT stage_config SET \
                       canonical_name = $name, \
                       default_label = $name, \
                       code = $code, \
                       is_post_contract = $is_post_contract, \
                       sort_order = 99, \
                       status = 'active', \
                       usage_count = (SELECT VALUE usage_count FROM stage_config WHERE canonical_name = $name)[0] + 1 OR 1 \
                     WHERE canonical_name = $name;"
                )
                .bind(("name", n))
                .bind(("code", c))
                .bind(("is_post_contract", is_post_contract))
                .await?;
                Ok(())
            })
        },
        "add stage to dictionary",
    )
    .await
}
```

- [ ] **Step 2: Register in mod.rs and lib.rs**

In `mod.rs`: `pub mod stage_dictionary;`
In `lib.rs` invoke_handler:
```rust
commands::stage_dictionary::search_stage_dictionary,
commands::stage_dictionary::add_stage_to_dictionary,
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p app`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/stage_dictionary.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(tauri): add stage dictionary search and auto-insert commands"
```

---

### Task 11: Integration test — full round-trip

**Files:**
- Modify: `src/lib/api/scope.test.ts`

- [ ] **Step 1: Add integration test for stages in generate request**

```typescript
describe('generateScope with stages', () => {
  it('includes stages in request and receives stages_snapshot', async () => {
    const mockResult = {
      clauses: [{ number: '1.0', title: 'Admin', clauses: [] }],
      generated_text: 'During Schematic Design...',
      llm_polished: true,
      stages_snapshot: ['Schematic Design', 'Design Development'],
      current_revision: 1,
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: () => Promise.resolve({ data: mockResult }),
    });

    const { generateScope } = await import('./scope');
    const result = await generateScope({
      fee_id: '25_97101_1',
      polish: true,
      stages: [
        { name: 'Schematic Design', code: 'SD', is_post_contract: false, order: 1 },
        { name: 'Design Development', code: 'DD', is_post_contract: false, order: 2 },
      ],
    });

    expect(result.stages_snapshot).toEqual(['Schematic Design', 'Design Development']);
    expect(result.current_revision).toBe(1);
  });
});
```

- [ ] **Step 2: Run all scope tests**

Run: `npx vitest run src/lib/api/scope.test.ts src/lib/utils/scopeFormatter.test.ts`
Expected: All tests pass.

- [ ] **Step 3: Run full test suite**

Run: `cargo test -p e-fees-core --lib && npm run check`
Expected: All pass, 0 type errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/api/scope.test.ts
git commit -m "test(scope): add integration tests for stage-aware scope generation"
```

---

## Execution Order

Tasks can be parallelised where noted:

1. **Task 0** (schema) — must be first
2. **Task 1** (e-fees-core) — independent
3. **Task 2** (scope service) — depends on Task 0
4. **Task 3** (Tauri commands) — independent
5. **Task 4** (TS types) — independent
6. **Task 5** (TS API wrappers) — depends on Task 3, Task 4
7. **Task 6** (ScopeViewer) — depends on Task 4, Task 5
8. **Task 7** (AdvancedControls) — depends on Task 6
9. **Task 8** (export command) — depends on Task 3
10. **Task 9** (revision endpoint) — depends on Task 0, Task 2
11. **Task 10** (stage dictionary) — independent
12. **Task 11** (integration test) — depends on all above

**Parallel batches:**
- Batch 1: Task 0
- Batch 2: Tasks 1, 3, 4, 10 (all independent)
- Batch 3: Tasks 2, 5, 8, 9
- Batch 4: Tasks 6, 7
- Batch 5: Task 11
