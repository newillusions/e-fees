# P3GLEG MCP Testing Quick Reference

## 🚀 After Restart: First Steps

### 1. Verify P3GLEG MCP is Available
Check for these tools in the function list:
- `tauri_take_screenshot`
- `tauri_click`  
- `tauri_type_text`
- `tauri_execute_js`
- `tauri_window_manage`

### 2. Launch Tauri Dev App
```bash
npm run tauri:dev
```
*This will include the P3GLEG plugin for development builds*

### 3. Basic App Testing
```bash
# Take screenshot to verify app is running
tauri_take_screenshot()

# Get window information
tauri_window_manage(action="get_info")
```

## 🎯 StatusChangeModal Testing Workflow

### Phase 1: Navigation
```bash
# 1. Screenshot current state
tauri_take_screenshot()

# 2. Navigate to Proposals (click navigation)
tauri_click(x=225, y=192)  # Approximate coordinates for Proposals nav

# 3. Screenshot Proposals page
tauri_take_screenshot()
```

### Phase 2: Test Proposal Interaction
```bash
# 4. Find and click test proposal "RFP-TEST-001"
# Look for proposal in the list and click edit button
tauri_click(x=<coord>, y=<coord>)  # Will need to determine coordinates

# 5. Screenshot with ProposalModal open
tauri_take_screenshot()
```

### Phase 3: Status Change Testing
```bash
# 6. Change status dropdown from "Completed" to "Lost"
tauri_click(x=<status_dropdown_x>, y=<status_dropdown_y>)
tauri_click(x=<lost_option_x>, y=<lost_option_y>)

# 7. Trigger save/submit to activate StatusChangeModal
tauri_click(x=<save_button_x>, y=<save_button_y>)

# 8. Screenshot with StatusChangeModal visible
tauri_take_screenshot()
```

### Phase 4: Modal Interaction Testing
```bash
# 9. Test modal suggestions
# - Verify project status suggestion appears
# - Test accept/reject buttons
tauri_click(x=<accept_button_x>, y=<accept_button_y>)

# 10. Final verification screenshot
tauri_take_screenshot()
```

## 📊 Test Data Reference

### Test Records
- **Project ID**: `projects:xigy1t9623hw1h33f59h`
- **Project Number**: 25-97199
- **Project Name**: "Status Change Test Project"
- **Proposal ID**: `fee:d351bfcgn9uxpccdgm2l`
- **Proposal Number**: "RFP-TEST-001"
- **Current Status**: Both Project and Proposal = "Completed"

### Database Verification Commands
```bash
# Check current proposal status
mcp__surrealdb-emittiv__select(table="fee", id="d351bfcgn9uxpccdgm2l")

# Check current project status  
mcp__surrealdb-emittiv__select(table="projects", id="xigy1t9623hw1h33f59h")

# Reset test data if needed
mcp__surrealdb-emittiv__merge(thing="fee:d351bfcgn9uxpccdgm2l", data={"status": "Draft"})
mcp__surrealdb-emittiv__merge(thing="projects:xigy1t9623hw1h33f59h", data={"status": "Draft"})
```

## 🔧 P3GLEG MCP Functions Reference

### Screenshot & Visual
- `tauri_take_screenshot()` - Capture current app state
- `tauri_window_manage()` - Get/set window properties

### Input Simulation  
- `tauri_click(x, y)` - Click at coordinates
- `tauri_type_text(text)` - Type text into active field
- `tauri_send_key(key)` - Send keyboard key

### Advanced
- `tauri_execute_js(code)` - Run JavaScript in app context
- `tauri_get_dom()` - Get DOM structure for element identification

## 🎯 Testing Success Criteria

### ✅ Expected Behaviors
1. **StatusChangeModal appears** when changing proposal status
2. **Project status suggestion** shows (e.g., "Lost" proposal suggests "Lost" project)
3. **Impact analysis** displays folder movement details
4. **Accept button** applies both status changes
5. **Folder movement** occurs automatically (verify via file system)
6. **Database updates** are consistent and timestamped

### 🚨 Critical Test Points
- Modal actually appears (not hidden or broken)
- Suggestions are contextually correct
- Folder operations execute without errors
- Rollback works if user cancels
- UI remains responsive throughout

## 📁 Folder Movement Verification
```bash
# Check current folder location
find "/Volumes/base/mms/DevTest" -name "*25-97199*" -type d

# Expected movements based on status:
# Draft → "01 RFPs"
# Active → "11 Current"  
# Completed → "99 Completed"
# Lost/Cancelled → "00 Inactive"
```

---
**Ready for comprehensive P3GLEG MCP testing!** 🚀