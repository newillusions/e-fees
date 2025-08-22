# E-Fees Logging Infrastructure Migration Guide

## Overview

This guide shows how to migrate from `console.error()` calls to the new professional logging service. The new system provides structured logging, multiple log levels, and integration with the Tauri backend.

## Quick Migration Reference

### Before (Old Pattern)
```typescript
console.error('Failed to create project:', error);
```

### After (New Pattern)
```typescript
import { logger } from '$lib/services/logger';
await logger.error('Failed to create project', { component: 'ProjectModal' }, error);
```

## Migration Examples by File

### 1. API Functions (`src/lib/api.ts`)

**Before:**
```typescript
export async function getProjects(): Promise<Project[]> {
  try {
    const projects = await invoke('get_projects');
    return projects as Project[];
  } catch (error) {
    console.error('Failed to fetch projects from database:', error);
    throw error;
  }
}
```

**After:**
```typescript
import { logger } from '$lib/services/logger';

export async function getProjects(): Promise<Project[]> {
  try {
    const projects = await invoke('get_projects');
    logger.info('Projects fetched successfully', { count: projects.length });
    return projects as Project[];
  } catch (error) {
    await logger.error('Failed to fetch projects from database', { 
      action: 'getProjects',
      module: 'api' 
    }, error as Error);
    throw error;
  }
}
```

### 2. Svelte Components (`src/lib/components/*.svelte`)

**Before:**
```typescript
// In ProjectModal.svelte
async function handleSave() {
  try {
    await createProject(projectData);
  } catch (error) {
    console.error('Failed to create project:', error);
  }
}
```

**After:**
```typescript
import { createComponentLogger } from '$lib/services/logger';
const logger = createComponentLogger('ProjectModal');

async function handleSave() {
  try {
    await createProject(projectData);
    await logger.info('Project created successfully', { 
      projectName: projectData.name,
      projectNumber: projectData.number 
    });
  } catch (error) {
    await logger.error('Failed to create project', { 
      action: 'handleSave',
      projectData: projectData.name 
    }, error as Error);
  }
}
```

### 3. Store Functions (`src/lib/stores.ts`)

**Before:**
```typescript
export const createProject = async (projectData: any) => {
  try {
    const result = await api.createProject(projectData);
    return result;
  } catch (error) {
    console.error('Failed to create project:', error);
    throw error;
  }
};
```

**After:**
```typescript
import { logger } from '$lib/services/logger';

export const createProject = async (projectData: any) => {
  try {
    const result = await api.createProject(projectData);
    await logger.info('Project created via store', { 
      projectName: projectData.name,
      source: 'store'
    });
    return result;
  } catch (error) {
    await logger.error('Store: Failed to create project', { 
      action: 'createProject',
      module: 'stores',
      projectName: projectData.name 
    }, error as Error);
    throw error;
  }
};
```

### 4. Route Components (`src/routes/*.svelte`)

**Before:**
```typescript
// In Dashboard.svelte
onMount(async () => {
  try {
    await loadData();
  } catch (error) {
    console.error('Failed to load dashboard data:', error);
  }
});
```

**After:**
```typescript
import { createComponentLogger } from '$lib/services/logger';
const logger = createComponentLogger('Dashboard');

onMount(async () => {
  const timer = logger.timer('DashboardMount');
  try {
    await loadData();
    await logger.info('Dashboard data loaded successfully');
  } catch (error) {
    await logger.error('Failed to load dashboard data', { 
      route: 'Dashboard',
      action: 'onMount' 
    }, error as Error);
  } finally {
    timer(); // Log how long it took
  }
});
```

## Batch Migration Script

Here's a Node.js script to help with bulk migration:

```javascript
// migrate-logging.js
const fs = require('fs');
const path = require('path');

const filesToMigrate = [
  'src/lib/api.ts',
  'src/lib/stores.ts',
  'src/lib/components/ProjectModal.svelte',
  // Add more files as needed
];

function migrateFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  
  // Add import at the top
  let newContent = content;
  if (!content.includes('$lib/services/logger')) {
    const importLine = "import { logger, createComponentLogger } from '$lib/services/logger';\n";
    newContent = importLine + newContent;
  }
  
  // Replace console.error patterns
  newContent = newContent.replace(
    /console\.error\('([^']+)'[^;]*;/g,
    "await logger.error('$1', { component: 'Unknown' });"
  );
  
  fs.writeFileSync(filePath, newContent);
  console.log(`Migrated: ${filePath}`);
}

filesToMigrate.forEach(migrateFile);
```

## Complete File Migration Examples

### Example 1: ContactModal.svelte

**Before:**
```typescript
async function handleSubmit() {
  try {
    if (editMode && originalContact) {
      await updateContact(contactData);
    } else {
      await createContact(contactData);
    }
    closeModal();
  } catch (error) {
    console.error('Failed to save contact:', error);
  }
}
```

**After:**
```typescript
import { createComponentLogger } from '$lib/services/logger';
const logger = createComponentLogger('ContactModal');

async function handleSubmit() {
  const action = editMode ? 'updateContact' : 'createContact';
  const timer = logger.timer(action);
  
  try {
    if (editMode && originalContact) {
      await updateContact(contactData);
      await logger.info('Contact updated successfully', {
        contactId: originalContact.id,
        action: 'update'
      });
    } else {
      await createContact(contactData);
      await logger.info('Contact created successfully', {
        contactName: contactData.first_name + ' ' + contactData.last_name,
        action: 'create'
      });
    }
    closeModal();
  } catch (error) {
    await logger.error(`Failed to ${action}`, {
      action,
      contactData: contactData.first_name + ' ' + contactData.last_name,
      editMode
    }, error as Error);
  } finally {
    timer();
  }
}
```

### Example 2: ConnectionStatus.svelte

**Before:**
```typescript
async function checkConnection() {
  try {
    const status = await getConnectionStatus();
    connectionStatus = status.connected;
  } catch (error) {
    console.error('Failed to check connection:', error);
    connectionStatus = false;
  }
}
```

**After:**
```typescript
import { createComponentLogger } from '$lib/services/logger';
const logger = createComponentLogger('ConnectionStatus');

async function checkConnection() {
  try {
    const status = await getConnectionStatus();
    connectionStatus = status.connected;
    
    if (status.connected) {
      await logger.debug('Database connection confirmed', { 
        status: status.status_message 
      });
    } else {
      await logger.warn('Database connection failed', { 
        error: status.error_message 
      });
    }
  } catch (error) {
    await logger.error('Failed to check database connection', {
      action: 'checkConnection'
    }, error as Error);
    connectionStatus = false;
  }
}
```

## Migration Checklist

### Phase 1: Core Infrastructure
- [x] ✅ Create logging service (`src/lib/services/logger.ts`)
- [x] ✅ Add Tauri command for frontend logging
- [x] ✅ Update invoke handler registration
- [ ] Test logging service basic functionality

### Phase 2: API Layer Migration (33 files)
- [ ] `src/lib/api.ts` - All API functions (30+ console.error calls)
- [ ] `src/lib/api.test.ts` - Update test expectations
- [ ] `src/lib/stores.ts` - Store error handling (8 console.error calls)
- [ ] `src/lib/stores/settings.ts` - Settings store (2 console.error calls)

### Phase 3: Component Migration (20+ files)
- [ ] `src/lib/components/ConnectionStatus.svelte` (3 calls)
- [ ] `src/lib/components/ProposalModal.svelte` (8 calls)
- [ ] `src/lib/components/ProjectModal.svelte` (5 calls)
- [ ] `src/lib/components/ProposalDetail.svelte` (6 calls)
- [ ] `src/lib/components/CompanyModal.svelte` (2 calls)
- [ ] `src/lib/components/NewProjectModal.svelte` (5 calls)
- [ ] `src/lib/components/StatusChangeModal.svelte` (1 call)
- [ ] `src/lib/components/FirstRunSetup.svelte` (1 call)
- [ ] `src/lib/components/SettingsModal.svelte` (1 call)
- [ ] `src/lib/components/ProjectDetail.svelte` (4 calls)
- [ ] `src/lib/components/dashboard/QuickActions.svelte` (1 call)
- [ ] `src/lib/components/SplashScreen.svelte` (1 call)

### Phase 4: Route Migration (4 files)
- [ ] `src/routes/Projects.svelte` (1 call)
- [ ] `src/routes/ProjectDetailPage.svelte` (1 call)
- [ ] `src/routes/ProposalDetailPage.svelte` (1 call)
- [ ] `src/App.svelte` (2 calls + console.log cleanup)
- [ ] `src/main.ts` (3 calls)

### Phase 5: Configuration & Production
- [ ] Configure log levels for production vs development
- [ ] Set up log rotation in Tauri backend
- [ ] Add log filtering and searching capabilities
- [ ] Performance testing and optimization

## Configuration Options

### Development Configuration
```typescript
// In development, enable all logging
logger.setLevel(LogLevel.DEBUG);
```

### Production Configuration  
```typescript
// In production, limit to warnings and errors
logger.setLevel(LogLevel.WARN);
```

### Tauri Backend Log Configuration
```rust
// In src-tauri/src/lib.rs
app.handle().plugin(
    tauri_plugin_log::Builder::default()
        .level(if cfg!(debug_assertions) { 
            log::LevelFilter::Debug 
        } else { 
            log::LevelFilter::Warn 
        })
        .build(),
)?;
```

## Best Practices

1. **Always use component loggers**: `createComponentLogger('ComponentName')`
2. **Include context**: Add relevant data like IDs, actions, user info
3. **Use appropriate levels**: Error for failures, warn for issues, info for events
4. **Don't log sensitive data**: Avoid passwords, tokens, personal info
5. **Use structured logging**: Consistent context object format
6. **Performance timing**: Use `logger.timer()` for slow operations
7. **Async logging**: Always await logger calls in async functions

## Testing the Migration

```typescript
// Test that logging works
import { logger } from '$lib/services/logger';

// This should appear in both console and Tauri logs
await logger.error('Test error message', { 
  component: 'Test',
  testField: 'testValue' 
});
```

## Log Output Examples

### Console Output (Development)
```
[ContactModal] updateContact: Contact updated successfully | Context: {"contactId": "contact:123", "action": "update"}
```

### Tauri Log Output (Production)
```
2024-08-21T10:30:45.123Z ERROR [ContactModal] Failed to update contact - Network timeout | Context: {"contactId": "contact:123", "action": "update", "error": "NetworkError"}
```

---

**Next Steps:**
1. Test the logging infrastructure with a few key files
2. Begin systematic migration starting with API layer
3. Update all components following the patterns above
4. Configure production logging levels
5. Set up log monitoring and rotation