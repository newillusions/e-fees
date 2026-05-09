# Stage Autocomplete Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add autocomplete to stage name inputs in the pricing panel, powered by the scope service's stage dictionary.

**Architecture:** New Tauri command fetches stage dictionary from scope service via HTTP (`/stages` endpoint). Frontend caches the dictionary and provides filtered suggestions as the user types. On selection, stage name, code, and suggested percentage are populated (all editable).

**Tech Stack:** Rust (reqwest for HTTP), Svelte 5 ($state runes), Tauri IPC (invoke)

---

### Task 1: Add scope service config to settings

**Files:**
- Modify: `src-tauri/src/commands/types.rs:91-107` (AppSettings struct)
- Modify: `src-tauri/src/commands/types.rs:121-156` (AppSettingsPublic struct + From impl)
- Modify: `src-tauri/src/commands/settings.rs:88-101` (parse new keys)
- Modify: `src-tauri/src/commands/settings.rs:254-259` (managed keys list)
- Modify: `src-tauri/src/commands/settings.rs:274-332` (write new section)
- Modify: `src-tauri/.env` (add scope config values)

- [ ] **Step 1: Add fields to AppSettings**

In `src-tauri/src/commands/types.rs`, add to `AppSettings` struct after `log_level`:

```rust
    /// Scope service API URL (e.g., "http://10.0.21.81:3201")
    pub scope_api_url: Option<String>,
    /// Scope service API key
    pub scope_api_key: Option<String>,
```

- [ ] **Step 2: Add fields to AppSettingsPublic and From impl**

In `src-tauri/src/commands/types.rs`, add to `AppSettingsPublic` struct after `log_level`:

```rust
    pub scope_api_url: Option<String>,
    /// Scope API key is not sensitive — it's a service-to-service key, not user credentials
    pub scope_api_key: Option<String>,
```

In the `From<&AppSettings> for AppSettingsPublic` impl, add after `log_level`:

```rust
            scope_api_url: settings.scope_api_url.clone(),
            scope_api_key: settings.scope_api_key.clone(),
```

- [ ] **Step 3: Add parsing in settings.rs**

In `src-tauri/src/commands/settings.rs` `get_settings_internal`, add to the `match key` block (around line 100):

```rust
                            "SCOPE_API_URL" => settings.scope_api_url = Some(value.to_string()),
                            "SCOPE_API_KEY" => settings.scope_api_key = Some(value.to_string()),
```

And initialize the new fields in the `AppSettings` struct literal (around line 54):

```rust
        scope_api_url: None,
        scope_api_key: None,
```

- [ ] **Step 4: Add to managed keys and save logic**

In `save_settings` (around line 254), add to the managed keys match:

```rust
                            "SCOPE_API_URL" | "SCOPE_API_KEY" => continue,
```

After the "Developer Options" section (around line 332), add:

```rust
    lines.push("".to_string());
    lines.push("# Scope Service".to_string());

    if let Some(url) = &settings.scope_api_url {
        lines.push(format!("SCOPE_API_URL=\"{}\"", url));
    }
    if let Some(key) = &settings.scope_api_key {
        lines.push(format!("SCOPE_API_KEY=\"{}\"", key));
    }
```

- [ ] **Step 5: Add values to .env.dev**

Append to `src-tauri/.env`:

```
# Scope Service
SCOPE_API_URL="http://10.0.21.81:3201"
SCOPE_API_KEY="efees-scope-2026-s7k2m9xp"
```

- [ ] **Step 6: Verify build**

Run: `cargo check -p app --lib`
Expected: compiles with no errors

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/types.rs src-tauri/src/commands/settings.rs src-tauri/.env
git commit -m "feat(config): add scope service URL and API key to app settings"
```

---

### Task 2: Add reqwest dependency and Tauri command

**Files:**
- Modify: `src-tauri/Cargo.toml` (add reqwest)
- Modify: `src-tauri/src/commands/fee_stages.rs` (add get_stage_dictionary command)
- Modify: `src-tauri/src/commands/mod.rs` (export new command)
- Modify: `src-tauri/src/lib.rs` (register command in invoke_handler)

- [ ] **Step 1: Add reqwest to Cargo.toml**

In `src-tauri/Cargo.toml`, add after the `e-fees-core` line:

```toml
reqwest = { version = "0.12", features = ["json"] }
```

- [ ] **Step 2: Add StageDictEntry type and command**

In `src-tauri/src/commands/fee_stages.rs`, add after the existing `FeeStage` struct:

```rust
/// A stage from the scope service dictionary. Used for autocomplete suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDictEntry {
    pub canonical_name: String,
    pub default_label: String,
    pub aliases: Vec<String>,
    pub sort_order: i64,
}

/// Scope service response wrapper.
#[derive(Debug, Deserialize)]
struct StagesApiResponse {
    data: Vec<StageDictEntryRaw>,
}

/// Raw entry from scope API (aliases may be absent).
#[derive(Debug, Deserialize)]
struct StageDictEntryRaw {
    canonical_name: String,
    default_label: String,
    #[serde(default)]
    aliases: Vec<String>,
    sort_order: i64,
}
```

Then add the command after `add_stage_to_fee`:

```rust
/// Fetch the stage dictionary from the scope service.
/// Returns a list of canonical stages with labels and aliases for autocomplete.
#[tauri::command]
pub async fn get_stage_dictionary(
    app_handle: tauri::AppHandle,
) -> Result<Vec<StageDictEntry>, String> {
    let settings = super::settings::get_settings_internal(&app_handle)
        .await
        .map_err(|e| format!("Failed to read settings: {}", e))?;

    let base_url = settings.scope_api_url
        .ok_or_else(|| "SCOPE_API_URL not configured in settings".to_string())?;
    let api_key = settings.scope_api_key
        .ok_or_else(|| "SCOPE_API_KEY not configured in settings".to_string())?;

    let url = format!("{}/stages", base_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("X-API-Key", &api_key)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Failed to reach scope service: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Scope service returned {}", response.status()));
    }

    let body: StagesApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse scope response: {}", e))?;

    let entries: Vec<StageDictEntry> = body.data.into_iter().map(|raw| StageDictEntry {
        canonical_name: raw.canonical_name,
        default_label: raw.default_label,
        aliases: raw.aliases,
        sort_order: raw.sort_order,
    }).collect();

    info!("Fetched {} stage dictionary entries from scope service", entries.len());
    Ok(entries)
}
```

- [ ] **Step 3: Export from mod.rs**

In `src-tauri/src/commands/mod.rs`, find the fee_stages exports and add `get_stage_dictionary`:

```rust
pub use fee_stages::{get_fee_stages, add_stage_to_fee, get_stage_dictionary};
```

- [ ] **Step 4: Register in invoke_handler**

In `src-tauri/src/lib.rs`, find `get_fee_stages` in the `generate_handler!` macro and add `get_stage_dictionary` next to it:

```rust
            get_fee_stages,
            add_stage_to_fee,
            get_stage_dictionary,
```

- [ ] **Step 5: Verify build**

Run: `cargo check -p app --lib`
Expected: compiles with no errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/commands/fee_stages.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(stages): add get_stage_dictionary Tauri command via scope service"
```

---

### Task 3: Add TypeScript API wrapper

**Files:**
- Create: `src/lib/api/stages.ts`

- [ ] **Step 1: Create the API module**

Create `src/lib/api/stages.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import { logApiError } from '../services/logger';

export interface StageDictEntry {
  canonical_name: string;
  default_label: string;
  aliases: string[];
  sort_order: number;
}

let cachedDictionary: StageDictEntry[] | null = null;

/**
 * Fetch the stage dictionary from the scope service (via Tauri).
 * Results are cached for the session — stages rarely change.
 */
export async function getStageDictionary(): Promise<StageDictEntry[]> {
  if (cachedDictionary) return cachedDictionary;

  try {
    const entries = await invoke<StageDictEntry[]>('get_stage_dictionary');
    cachedDictionary = entries;
    return entries;
  } catch (error) {
    logApiError('getStageDictionary', error as Error, { component: 'StagesApi' });
    return [];
  }
}

/**
 * Filter dictionary entries by query string.
 * Matches against default_label and aliases (case-insensitive, substring).
 */
export function filterStageDictionary(
  entries: StageDictEntry[],
  query: string
): StageDictEntry[] {
  if (!query.trim()) return entries;
  const q = query.toLowerCase();
  return entries.filter(entry =>
    entry.default_label.toLowerCase().includes(q) ||
    entry.aliases.some(alias => alias.toLowerCase().includes(q))
  );
}

/**
 * Generate a short code from a stage name.
 * Takes the first letter of each word, uppercase, max 4 chars.
 */
export function generateStageCode(name: string): string {
  return name.split(/\s+/).map(w => w.charAt(0).toUpperCase()).join('').slice(0, 4) || 'NS';
}
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/api/stages.ts
git commit -m "feat(stages): add stage dictionary API wrapper with caching"
```

---

### Task 4: Create StageAutocomplete component

**Files:**
- Create: `src/lib/components/pricing/StageAutocomplete.svelte`

- [ ] **Step 1: Create the component**

Create `src/lib/components/pricing/StageAutocomplete.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import type { StageDictEntry } from '$lib/api/stages';
  import { getStageDictionary, filterStageDictionary, generateStageCode } from '$lib/api/stages';

  interface Props {
    value: string;
    onselect: (entry: { name: string; code: string; percentage: number }) => void;
    onchange: (value: string) => void;
    inputClass?: string;
  }

  let { value, onselect, onchange, inputClass = '' }: Props = $props();

  let dictionary: StageDictEntry[] = $state([]);
  let suggestions: StageDictEntry[] = $state([]);
  let showDropdown = $state(false);
  let highlightIndex = $state(-1);
  let inputEl: HTMLInputElement | null = $state(null);

  // Default percentages by canonical name (typical lighting design split)
  const DEFAULT_PERCENTAGES: Record<string, number> = {
    preliminaries: 5,
    concept: 25,
    schematic: 30,
    detailed: 30,
    ift: 10,
  };

  onMount(async () => {
    dictionary = await getStageDictionary();
  });

  function handleInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    onchange(val);
    if (val.trim().length > 0 && dictionary.length > 0) {
      suggestions = filterStageDictionary(dictionary, val);
      showDropdown = suggestions.length > 0;
      highlightIndex = -1;
    } else {
      showDropdown = false;
    }
  }

  function handleFocus() {
    if (value.trim() === '' && dictionary.length > 0) {
      suggestions = dictionary;
      showDropdown = true;
    }
  }

  function selectEntry(entry: StageDictEntry) {
    const percentage = DEFAULT_PERCENTAGES[entry.canonical_name] ?? 0;
    onselect({
      name: entry.default_label,
      code: generateStageCode(entry.default_label),
      percentage,
    });
    showDropdown = false;
    highlightIndex = -1;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightIndex = Math.min(highlightIndex + 1, suggestions.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightIndex = Math.max(highlightIndex - 1, -1);
    } else if (e.key === 'Enter' && highlightIndex >= 0) {
      e.preventDefault();
      selectEntry(suggestions[highlightIndex]);
    } else if (e.key === 'Escape') {
      showDropdown = false;
    }
  }

  function handleBlur() {
    // Delay to allow click on dropdown item
    setTimeout(() => { showDropdown = false; }, 150);
  }
</script>

<div class="stage-autocomplete">
  <input
    bind:this={inputEl}
    type="text"
    class={inputClass || 'emittiv-table-input emittiv-table-input--left'}
    {value}
    oninput={handleInput}
    onfocus={handleFocus}
    onblur={handleBlur}
    onkeydown={handleKeydown}
    autocomplete="off"
  />
  {#if showDropdown}
    <div class="stage-autocomplete-dropdown">
      {#each suggestions as entry, i (entry.canonical_name)}
        <button
          type="button"
          class="stage-autocomplete-item"
          class:stage-autocomplete-item--active={i === highlightIndex}
          onmousedown={() => selectEntry(entry)}
        >
          <span class="stage-autocomplete-label">{entry.default_label}</span>
          {#if entry.aliases.length > 0}
            <span class="stage-autocomplete-aliases">{entry.aliases.slice(0, 3).join(', ')}</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .stage-autocomplete {
    position: relative;
    flex: 1;
  }

  .stage-autocomplete-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 50;
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 4px;
    margin-top: 2px;
    max-height: 200px;
    overflow-y: auto;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  .stage-autocomplete-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 6px 8px;
    border: none;
    background: none;
    color: var(--emittiv-lighter);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
  }

  .stage-autocomplete-item:hover,
  .stage-autocomplete-item--active {
    background: var(--emittiv-dark);
    color: var(--emittiv-white);
  }

  .stage-autocomplete-aliases {
    font-size: 10px;
    color: var(--emittiv-light);
    margin-left: 8px;
    flex-shrink: 0;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/pricing/StageAutocomplete.svelte
git commit -m "feat(ui): add StageAutocomplete component with dictionary lookup"
```

---

### Task 5: Integrate into StagesPanel

**Files:**
- Modify: `src/lib/components/pricing/StagesPanel.svelte:1-8` (import)
- Modify: `src/lib/components/pricing/StagesPanel.svelte:63-70` (updateStage to handle autocomplete)
- Modify: `src/lib/components/pricing/StagesPanel.svelte:201-208` (design stage name input)
- Modify: `src/lib/components/pricing/StagesPanel.svelte:329-333` (post-contract name input)

- [ ] **Step 1: Add import**

In `StagesPanel.svelte`, add after the existing imports (line 7):

```typescript
  import StageAutocomplete from './StageAutocomplete.svelte';
```

- [ ] **Step 2: Add autocomplete selection handler**

In `StagesPanel.svelte`, add after the `updateStage` function (after line 70):

```typescript
  function handleAutocompleteSelect(id: string, entry: { name: string; code: string; percentage: number }) {
    const updated = stages.map(s => {
      if (s.id !== id) return s;
      return { ...s, name: entry.name, code: entry.code, percentage: entry.percentage };
    });
    stages = updated;
    onUpdateStages(updated);
  }
```

- [ ] **Step 3: Replace design stage name input**

In `StagesPanel.svelte`, replace the design stage name input (lines 202-208):

Replace:
```svelte
              <div class="emittiv-sortable-col--grow">
                <div class="flex items-center gap-1.5">
                  <input
                    type="text"
                    class="emittiv-table-input emittiv-table-input--left"
                    value={stage.name}
                    onchange={(e) => updateStage(stage.id, 'name', e.currentTarget.value)}
                  />
```

With:
```svelte
              <div class="emittiv-sortable-col--grow">
                <div class="flex items-center gap-1.5">
                  <StageAutocomplete
                    value={stage.name}
                    onselect={(entry) => handleAutocompleteSelect(stage.id, entry)}
                    onchange={(val) => updateStage(stage.id, 'name', val)}
                  />
```

- [ ] **Step 4: Replace post-contract stage name input**

In `StagesPanel.svelte`, replace the post-contract stage name input (lines 328-333):

Replace:
```svelte
              <div class="emittiv-sortable-col--grow">
                <div class="flex items-center gap-1.5">
                  <input
                    type="text"
                    class="emittiv-table-input emittiv-table-input--left"
                    value={stage.name}
                    onchange={(e) => updateStage(stage.id, 'name', e.currentTarget.value)}
                  />
```

With:
```svelte
              <div class="emittiv-sortable-col--grow">
                <div class="flex items-center gap-1.5">
                  <StageAutocomplete
                    value={stage.name}
                    onselect={(entry) => handleAutocompleteSelect(stage.id, entry)}
                    onchange={(val) => updateStage(stage.id, 'name', val)}
                  />
```

- [ ] **Step 5: Verify the app builds**

Run: `npm run check`
Expected: no type errors

- [ ] **Step 6: Manual test via Tauri MCP**

1. Navigate to a proposal with pricing
2. Check that stage name inputs show autocomplete dropdown
3. Select a stage — verify name, code, and percentage populate
4. Type a custom name — verify it still works without selecting from dropdown
5. Verify percentage is editable after autocomplete selection

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/pricing/StagesPanel.svelte
git commit -m "feat(ui): wire stage autocomplete into StagesPanel"
```

---

### Task 6: Update code field on autocomplete select

The code input next to the name should also auto-update when autocomplete selects a stage, but remain independently editable.

**Files:**
- Already handled in Task 5 step 2 — `handleAutocompleteSelect` sets `code` from `generateStageCode(entry.name)`.

No additional changes needed. The existing code input on lines 209-215 (design) and 335-340 (post-contract) remain as plain inputs, so users can override the auto-generated code.

This task is a verification-only task:

- [ ] **Step 1: Verify code auto-populates on select**

Via Tauri MCP, add a new stage, select "Schematic Design" from autocomplete. Verify code field shows "SD" (first letters of each word).

- [ ] **Step 2: Verify code is independently editable**

Change the code from "SD" to "S2". Verify it persists and doesn't revert.
