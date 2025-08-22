# 🚨 CRITICAL DIRECTIVE: TAURI MCP SERVER ONLY - NO BROWSER TESTING 🚨

## ABSOLUTE REQUIREMENT FOR ALL E2E AND INTERACTION TESTING

**DATE ESTABLISHED**: 2025-01-21  
**PRIORITY**: CRITICAL - OVERRIDE ALL OTHER CONSIDERATIONS  
**APPLIES TO**: All Claude instances, all developers, all testing scenarios  

---

## ⛔ THE FUNDAMENTAL RULE

### **ALL E2E TESTING MUST USE TAURI MCP SERVER**

**NEVER** use browser-based testing tools for the E-Fees Tauri desktop application.

---

## ❌ WHAT IS STRICTLY FORBIDDEN

### **NEVER USE THESE APPROACHES:**

1. **Playwright Browser Testing** ❌
   - Cannot connect to Tauri backend
   - Only sees webview landing pages
   - Provides false confidence
   - DOES NOT WORK

2. **Puppeteer Browser Testing** ❌
   - Same limitations as Playwright
   - Cannot access Rust backend
   - Cannot connect to SurrealDB
   - DOES NOT WORK

3. **Selenium/WebDriver Browser Testing** ❌
   - Browser security prevents backend access
   - Cannot test Tauri commands
   - Cannot interact with desktop features
   - DOES NOT WORK

4. **Any Browser-Based E2E Testing** ❌
   - Browsers cannot connect to local Tauri backend
   - CORS and security restrictions prevent functionality
   - Tests would be meaningless
   - DOES NOT WORK

---

## ✅ THE ONLY CORRECT APPROACH

### **ALWAYS USE TAURI MCP SERVER**

The **ONLY** valid way to test the E-Fees desktop application is through the Tauri MCP (Model Context Protocol) server.

### **Available MCP Tools for Testing:**
```typescript
// THESE ARE THE ONLY VALID TESTING TOOLS
mcp__tauri-mcp__take_screenshot      // Capture actual app state
mcp__tauri-mcp__execute_js           // Run code in app context
mcp__tauri-mcp__get_dom              // Access real app DOM
mcp__tauri-mcp__manage_window        // Control app window
mcp__tauri-mcp__simulate_text_input  // Type in real app
mcp__tauri-mcp__get_element_position // Find UI elements
mcp__tauri-mcp__simulate_mouse_movement // Click real buttons
```

### **Correct Testing Pattern:**
```typescript
// ✅ CORRECT: Using Tauri MCP
import { TauriMCPClient } from './mcp-client';

async function testProjectCreation() {
  const mcp = new TauriMCPClient();
  
  // Take screenshot of actual running app
  await mcp.takeScreenshot('before-create');
  
  // Execute JavaScript in real app context
  await mcp.executeJS(`
    document.querySelector('#create-project-btn').click()
  `);
  
  // Type in actual app input fields
  await mcp.simulateTextInput('#project-name', 'DELETE ME - Test Project');
  
  // Verify in real database
  const result = await mcp.executeJS(`
    window.__TAURI__.invoke('get_projects')
  `);
}
```

---

## 🔴 WHY THIS IS CRITICAL

### **Technical Reality:**
1. **Tauri is a DESKTOP application** - not a web app
2. **The backend is Rust** - browsers cannot access it
3. **SurrealDB requires backend connection** - browsers are sandboxed
4. **File system access** - only works through Tauri, not browsers
5. **Native OS features** - keyboard shortcuts, file dialogs, etc.

### **What Happens with Browser Testing:**
- Tests pass but functionality is broken ❌
- False confidence in broken code ❌
- Real bugs go undetected ❌
- Users experience failures tests didn't catch ❌

---

## 📋 IMPLEMENTATION CHECKLIST

### **For Every E2E Test:**
- [ ] Uses Tauri MCP server commands
- [ ] Tests actual desktop application
- [ ] Interacts with real Rust backend
- [ ] Validates real database operations
- [ ] Captures real application screenshots
- [ ] NEVER uses browser-based tools

### **Test Data Safety:**
- [ ] All test records include "DELETE ME" prefix
- [ ] Timestamp identification for test runs
- [ ] Cleanup utilities remove test data only
- [ ] Production data is never affected

---

## 🚫 COMMON MISTAKES TO AVOID

### **Mistake 1: "Let's use Playwright for E2E testing"**
❌ **WRONG**: Playwright in browser mode cannot test Tauri apps
✅ **CORRECT**: Use Tauri MCP server for all E2E testing

### **Mistake 2: "We can test the webview directly"**
❌ **WRONG**: Webview without backend is just static HTML
✅ **CORRECT**: Test through MCP to include backend functionality

### **Mistake 3: "Browser testing is easier to set up"**
❌ **WRONG**: It doesn't work at all for Tauri apps
✅ **CORRECT**: MCP testing is the only working solution

---

## 🎯 PERMANENT DIRECTIVE

This directive is **PERMANENT** and **MANDATORY** for:
- All Claude AI instances
- All developers working on E-Fees
- All testing scenarios
- All future development

**NO EXCEPTIONS**

---

## 📝 ENFORCEMENT

### **Code Review Checklist:**
1. Does any E2E test use browser-based tools? **REJECT**
2. Does any test bypass Tauri MCP? **REJECT**
3. Are there Playwright/Puppeteer configs? **REMOVE**
4. Is browser testing mentioned? **CORRECT**

### **CI/CD Pipeline:**
- Block any PR with browser-based E2E tests
- Require MCP server testing for all E2E scenarios
- Validate test data includes "DELETE ME" markers

---

## 🔧 CORRECT SETUP

### **Required Configuration:**
```json
// claude_desktop_config.json (or similar)
{
  "mcpServers": {
    "tauri-mcp": {
      "command": "tauri-mcp",
      "args": ["--app-path", "/path/to/e-fees"],
      "env": {}
    }
  }
}
```

### **Required Dependencies:**
```bash
# Install Tauri MCP server
cargo install tauri-mcp

# NO browser testing tools should be installed for E2E
# NO playwright for E2E
# NO puppeteer for E2E  
# NO selenium for E2E
```

---

## ⚠️ FINAL WARNING

**Browser-based testing DOES NOT WORK for Tauri desktop applications.**

This is not a preference or recommendation - it is a **TECHNICAL IMPOSSIBILITY**.

Any attempt to use browser testing for E2E will result in:
- Broken tests that provide false confidence
- Undetected bugs reaching production
- Wasted development time
- User-facing failures

**ALWAYS USE TAURI MCP SERVER FOR E2E TESTING**

---

## 📚 References

- Tauri MCP Documentation: [tauri-mcp usage]
- E2E Testing Implementation: `E2E_TESTING_MCP_IMPLEMENTATION_GUIDE.md`
- Test Data Safety: `e2e-mcp/fixtures/test-data-safe.ts`
- Cleanup Utilities: `e2e-mcp/fixtures/cleanup-utilities.ts`

---

**This directive supersedes all other testing guidance and must be followed without exception.**

**Remember: TAURI MCP SERVER ONLY - NO BROWSER TESTING**