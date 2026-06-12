# Code Patterns Quick Reference

> **Purpose**: Common patterns used in e-fees project
> **Use**: Copy these proven patterns for consistency

---

## 🗄️ Database Patterns

### Thing Object Extraction

**Problem**: SurrealDB returns IDs as Thing objects, not strings

```typescript
// ✅ CORRECT - Always use extractId()
import { extractId } from '$lib/utils/database'

const companyId = extractId(contact.company)
const projectId = extractId(invoice.project)

// ❌ WRONG - Don't use Thing directly
const companyId = contact.company  // This is { tb: 'companies', id: '...' }
```

### Safe ID Extraction Helper

```typescript
// src/lib/utils/database.ts
export function extractId(value: any): string | null {
  if (!value) return null
  if (typeof value === 'object' && 'id' in value) {
    return String(value.id)
  }
  if (typeof value === 'string') return value
  return null
}
```

### Database Query Pattern

```rust
// src-tauri/src/db/queries.rs
use surrealdb::sql::Thing;

pub async fn get_contact(db: &Surreal<Any>, id: &str) -> Result<Contact, String> {
    let result: Option<Contact> = db
        .select(("contacts", id))
        .await
        .map_err(|e| format!("Failed to get contact: {}", e))?;

    result.ok_or_else(|| "Contact not found".to_string())
}
```

---

## 🎨 Tauri IPC Patterns

### Command Registration

```rust
// src-tauri/src/main.rs
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::contacts::create_contact,
            commands::contacts::get_all_contacts,
            commands::contacts::update_contact,
            commands::contacts::delete_contact,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Command Implementation

```rust
// src-tauri/src/commands/contacts.rs
use tauri::State;
use crate::db::AppState;

#[tauri::command]
pub async fn create_contact(
    state: State<'_, AppState>,
    contact: Contact,
) -> Result<Contact, String> {
    let db = &state.db;

    let created: Contact = db
        .create("contacts")
        .content(contact)
        .await
        .map_err(|e| format!("Failed to create contact: {}", e))?;

    Ok(created)
}
```

### Frontend Invocation

```typescript
// src/lib/api/contacts.ts
import { invoke } from '@tauri-apps/api/core'
import type { Contact } from '$lib/types'

export async function createContact(contact: Contact): Promise<Contact> {
  try {
    return await invoke<Contact>('create_contact', { contact })
  } catch (error) {
    console.error('Failed to create contact:', error)
    throw new Error(`Failed to create contact: ${error}`)
  }
}
```

---

## 🎭 Svelte 5 Patterns

### State Management (Runes)

```svelte
<script lang="ts">
  // ✅ CORRECT - Svelte 5 runes
  let count = $state(0)
  let doubled = $derived(count * 2)

  $effect(() => {
    console.log('Count changed:', count)
  })

  // ❌ WRONG - Legacy Svelte syntax
  // let count = 0
  // $: doubled = count * 2
</script>
```

### Component Props

```svelte
<script lang="ts">
  import type { Contact } from '$lib/types'

  interface Props {
    contact: Contact
    onUpdate?: (contact: Contact) => void
  }

  let { contact, onUpdate }: Props = $props()
</script>
```

### Event Handling

```svelte
<script lang="ts">
  import { createContact } from '$lib/api/contacts'

  let name = $state('')
  let loading = $state(false)
  let error = $state<string | null>(null)

  async function handleSubmit() {
    loading = true
    error = null

    try {
      await createContact({ name })
      // Success handling
    } catch (e) {
      error = e instanceof Error ? e.message : 'Unknown error'
    } finally {
      loading = false
    }
  }
</script>

<form onsubmit={handleSubmit}>
  <input bind:value={name} disabled={loading} />
  {#if error}
    <p class="text-red-600">{error}</p>
  {/if}
  <button type="submit" disabled={loading}>
    {loading ? 'Saving...' : 'Save'}
  </button>
</form>
```

---

## 🎨 DPI Scaling Patterns

### Rem-Based Sizing

```svelte
<!-- ✅ CORRECT - Rem-based (scales with OS zoom) -->
<div class="h-8 w-32 p-4">  <!-- 2rem, 8rem, 1rem -->

<!-- ❌ WRONG - Fixed pixels (breaks at 125%/150%) -->
<div class="h-[32px] w-[128px] p-[16px]">
```

### Min-Height for Text

```svelte
<!-- ✅ CORRECT - Prevents text clipping -->
<div class="min-h-24 p-4">
  {longText}
</div>

<!-- ❌ WRONG - Fixed height clips at higher DPI -->
<div class="h-24 p-4">
  {longText}
</div>
```

### Responsive Containers

```svelte
<!-- ✅ CORRECT - Flexible, scales properly -->
<div class="max-w-4xl w-full px-4">
  <div class="space-y-4">
    <!-- Content -->
  </div>
</div>
```

---

## 🧪 Testing Patterns

### "DELETE ME" Pattern (MANDATORY)

```typescript
// ✅ CORRECT - All test data gets cleaned up
import { test, expect } from '@playwright/test'

test('should create contact', async ({ page }) => {
  const testData = {
    name: `DELETE ME - Test Contact ${Date.now()}`,
    email: `delete-me-${Date.now()}@example.com`,
    company: `DELETE ME - Test Company ${Date.now()}`
  }

  // Test implementation
})

// ❌ WRONG - Contaminates database
const testData = {
  name: 'Test Contact',
  email: 'test@example.com'
}
```

### E2E Test Structure

```typescript
// e2e-mcp/tests/contacts.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Contacts', () => {
  test.beforeEach(async ({ page }) => {
    // Setup
    await page.goto('/')
  })

  test.afterEach(async () => {
    // Cleanup test data
    // Runs automatically if using "DELETE ME" prefix
  })

  test('should perform action', async ({ page }) => {
    // Arrange
    const testData = {
      name: `DELETE ME - Test ${Date.now()}`
    }

    // Act
    await page.click('[data-testid="create-button"]')
    await page.fill('[data-testid="name-input"]', testData.name)
    await page.click('[data-testid="save-button"]')

    // Assert
    await expect(page.locator(`text=${testData.name}`)).toBeVisible()
  })
})
```

---

## 🔒 Error Handling Patterns

### Rust Error Handling

```rust
// ✅ CORRECT - Proper error propagation
use surrealdb::Error as DbError;

pub async fn get_contact(db: &Surreal<Any>, id: &str) -> Result<Contact, String> {
    let contact: Option<Contact> = db
        .select(("contacts", id))
        .await
        .map_err(|e| format!("Database error: {}", e))?;  // ? operator

    contact.ok_or_else(|| format!("Contact {} not found", id))
}

// ❌ WRONG - Using unwrap (panics on error)
pub async fn get_contact(db: &Surreal<Any>, id: &str) -> Contact {
    db.select(("contacts", id)).await.unwrap()  // Don't do this!
}
```

### TypeScript Error Handling

```typescript
// ✅ CORRECT - Try/catch with proper error message
export async function loadContacts(): Promise<Contact[]> {
  try {
    return await invoke<Contact[]>('get_all_contacts')
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error'
    console.error('Failed to load contacts:', message)
    throw new Error(`Failed to load contacts: ${message}`)
  }
}

// ❌ WRONG - Silent failure
export async function loadContacts(): Promise<Contact[]> {
  try {
    return await invoke<Contact[]>('get_all_contacts')
  } catch {
    return []  // Hides the error!
  }
}
```

---

## 🎨 Tailwind Patterns

### Common Component Styles

```svelte
<!-- Button -->
<button class="
  px-4 py-2
  bg-blue-600 hover:bg-blue-700
  text-white font-medium rounded-lg
  transition-colors duration-200
  disabled:opacity-50 disabled:cursor-not-allowed
">
  Click Me
</button>

<!-- Card -->
<div class="
  bg-white dark:bg-gray-800
  rounded-lg shadow-md
  p-6
  space-y-4
">
  <!-- Content -->
</div>

<!-- Input -->
<input class="
  w-full px-3 py-2
  border border-gray-300 rounded-lg
  focus:outline-none focus:ring-2 focus:ring-blue-500
  disabled:bg-gray-100
" />
```

### Layout Patterns

```svelte
<!-- Centered container -->
<div class="max-w-4xl mx-auto px-4 py-8">
  <!-- Content -->
</div>

<!-- Two-column layout -->
<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
  <div><!-- Left column --></div>
  <div><!-- Right column --></div>
</div>

<!-- Flex layout -->
<div class="flex items-center justify-between">
  <div><!-- Left --></div>
  <div><!-- Right --></div>
</div>
```

---

## 🔄 Async Patterns

### Loading States

```svelte
<script lang="ts">
  import { onMount } from 'svelte'
  import { loadContacts } from '$lib/api/contacts'

  let contacts = $state<Contact[]>([])
  let loading = $state(true)
  let error = $state<string | null>(null)

  onMount(async () => {
    try {
      contacts = await loadContacts()
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load'
    } finally {
      loading = false
    }
  })
</script>

{#if loading}
  <div>Loading...</div>
{:else if error}
  <div class="text-red-600">Error: {error}</div>
{:else}
  {#each contacts as contact}
    <div>{contact.name}</div>
  {/each}
{/if}
```

---

## 📝 Type Patterns

### Common Types

```typescript
// src/lib/types/index.ts

// Thing object type
export interface Thing {
  tb: string
  id: string
}

// Database models
export interface Contact {
  id?: Thing | string
  name: string
  email: string
  company?: Thing | string
  created_at?: string
  updated_at?: string
}

// API response types
export type ApiResult<T> = {
  success: true
  data: T
} | {
  success: false
  error: string
}
```

---

## 🔗 Import Patterns

```typescript
// ✅ CORRECT - Organized imports
// External libraries
import { invoke } from '@tauri-apps/api/core'
import { onMount } from 'svelte'

// Types
import type { Contact } from '$lib/types'

// Utils
import { extractId } from '$lib/utils/database'

// Components
import ContactCard from '$lib/components/ContactCard.svelte'

// API
import { createContact } from '$lib/api/contacts'
```

---

## 📚 Related Resources

- [Database Schema](../context/database-schema.md) - Full schema reference
- [Tauri Development](../prompts/tauri-development.md) - IPC patterns
- [Testing Strategy](../context/testing-strategy.md) - Testing approach
- [Frontend Specialist](../subagents/subagent-frontend-specialist.md) - UI patterns

---

**Remember**: Consistency is key. When in doubt, find an existing example and follow the same pattern.
