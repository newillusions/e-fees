# 📂 Project Folder Management Testing Guide

## ✅ Implementation Complete

**Phase 1 of the Project Folder Management System has been fully implemented!**

### 🎯 What's Been Built

#### 🔧 **Backend (Rust)**
- **6 core commands** for folder operations
- **Automatic template copying** when projects move from RFP → Current
- **Smart status-to-folder mapping**
- **Safe test environment** using `/Volumes/base/mms/DevTest/`

#### 🎨 **Frontend (TypeScript/Svelte)**
- **StatusChangeModal** with comprehensive impact analysis
- **Integrated into ProjectModal** for seamless UX
- **Visual feedback** showing folder movements and downstream effects
- **Related proposal impact** warnings

#### 📁 **Test Environment Setup**
- **Test folders**: 25-97199, 25-97107 in RFP directory
- **Proper folder structure**: 00 Inactive, 01 RFPs, 11 Current, 99 Completed
- **Template folders**: Contract, Deliverables, Submittals, etc.

---

## 🧪 Testing Instructions

### Step 1: Create Test Database Entries

1. **Start the application**:
   ```bash
   npm run tauri:dev
   ```

2. **Open browser console** (F12) and paste the test data creation script:
   ```javascript
   // Copy contents from create_test_data.js
   // Then run: createTestData()
   ```

3. **Verify test data creation**:
   - Check Projects page for "25-97199 - Test Project for Folder Management"
   - Check Companies page for "Test Client LLC" 
   - Check Proposals page for "25-97199-FP-01"

### Step 2: Test Folder Movement Scenarios

#### 🔄 **Scenario 1: RFP → Current (Award Project)**

1. **Navigate to Projects page**
2. **Find project**: "25-97199 - Test Project for Folder Management"
3. **Click Edit** (pencil icon)
4. **Change status** from "RFP" to "Active"
5. **Click Update**

**Expected Results:**
- ✅ StatusChangeModal appears
- ✅ Shows folder movement: `01 RFPs` → `11 Current`
- ✅ Shows "Awarded project templates will be copied automatically"
- ✅ Lists related proposals that may be affected
- ✅ After confirmation, project folder moves to Current directory
- ✅ Template folders are copied to project folder

#### 🗃️ **Scenario 2: Current → Archive (Complete Project)**

1. **Change status** from "Active" to "Completed"
2. **Click Update**

**Expected Results:**
- ✅ StatusChangeModal appears
- ✅ Shows folder movement: `11 Current` → `99 Completed` 
- ✅ Shows proposal completion recommendations
- ✅ After confirmation, project moves to Completed directory

#### ❌ **Scenario 3: Any Status → Cancelled**

1. **Change status** to "Cancelled"
2. **Click Update**

**Expected Results:**
- ✅ StatusChangeModal appears
- ✅ Shows folder movement to `00 Inactive`
- ✅ Warns about affected proposals needing updates
- ✅ After confirmation, project moves to Inactive directory

### Step 3: Verify File System Changes

After each test, check the actual folders:

```bash
# List test folders
ls -la "/Volumes/base/mms/DevTest/"

# Check if project moved correctly
ls -la "/Volumes/base/mms/DevTest/11 Current/"
ls -la "/Volumes/base/mms/DevTest/99 Completed/"
ls -la "/Volumes/base/mms/DevTest/00 Inactive/"

# Verify template folders were copied (for RFP → Current)
ls -la "/Volumes/base/mms/DevTest/11 Current/25-97199 Test Project/"
```

**Expected folder structure after RFP → Current:**
```
25-97199 Test Project/
├── 01 Client Info/
├── 02 Proposal/
├── 03 Contract/          ← NEW (from templates)
├── 04 Deliverables/      ← NEW (from templates)
├── 05 Submittals/        ← NEW (from templates)
├── 11 SubContractors/    ← NEW (from templates)
├── 98 Outgoing/          ← NEW (from templates)
└── 99 Temp/              ← NEW (from templates)
```

### Step 4: Test Error Scenarios

#### 🚫 **Test 1: Missing Project Folder**
1. Create a project in database with a number that doesn't have a folder
2. Try to change its status
3. **Expected**: Error message about folder not found

#### 🚫 **Test 2: Destination Already Exists**
1. Manually create a duplicate folder in destination
2. Try to move project
3. **Expected**: Error message about destination already existing

---

## 🔍 Backend Testing (Optional)

You can also test the backend functions directly via browser console:

```javascript
// Test folder location detection
const location = await invoke('get_project_folder_location', { 
  projectNumber: '25-97199' 
});
console.log('Project location:', location);

// Test base path validation
const validation = await invoke('validate_project_base_path');
console.log('Path validation:', validation);

// Test project listing
const rfpProjects = await invoke('list_projects_in_folder', { 
  folderPath: '01 RFPs' 
});
console.log('RFP projects:', rfpProjects);

// Test manual folder movement
const moveResult = await invoke('move_project_from_rfp', { 
  projectNumber: '25-97199',
  destination: 'current'
});
console.log('Move result:', moveResult);
```

---

## 📊 Success Criteria

### ✅ **Functional Requirements**
- [ ] Projects can be moved between lifecycle folders via UI
- [ ] StatusChangeModal shows accurate impact analysis
- [ ] Template folders are copied automatically for awarded projects
- [ ] Database status stays synchronized with folder location
- [ ] Related proposal impact is clearly communicated
- [ ] Error handling provides clear feedback

### ✅ **Technical Requirements**
- [ ] No data loss during folder operations
- [ ] Atomic operations (folder + database updates)
- [ ] Proper error recovery and rollback
- [ ] Performance is acceptable (< 3 seconds for folder moves)
- [ ] UI remains responsive during operations

### ✅ **User Experience**
- [ ] Clear visual feedback for all operations
- [ ] Confirmation dialogs prevent accidental changes
- [ ] Progress indicators for long operations
- [ ] Helpful error messages with suggested actions

---

## 🐛 Troubleshooting

### Common Issues:

1. **"Template folder not found" error**
   - Check that `/Volumes/base/mms/DevTest/11 Current/00 Additional Folders/` exists
   - Verify template folders are properly structured

2. **"Project folder not found" error**
   - Ensure test project folders exist in expected locations
   - Check folder naming matches project numbers exactly

3. **Database connection issues**
   - Verify SurrealDB is running on ws://10.0.1.17:8000
   - Check network connectivity to database server

4. **TypeScript compilation errors**
   - Run `npm run check` to identify type issues
   - Ensure all imports are correctly resolved

### Reset Test Environment:

```bash
# Move any test projects back to RFPs for fresh testing
mv "/Volumes/base/mms/DevTest/11 Current/25-97199"* "/Volumes/base/mms/DevTest/01 RFPs/" 2>/dev/null
mv "/Volumes/base/mms/DevTest/99 Completed/25-97199"* "/Volumes/base/mms/DevTest/01 RFPs/" 2>/dev/null
mv "/Volumes/base/mms/DevTest/00 Inactive/25-97199"* "/Volumes/base/mms/DevTest/01 RFPs/" 2>/dev/null
```

---

## 🎉 **Ready for Testing!**

The Project Folder Management System Phase 1 is complete and ready for comprehensive testing. The system provides intelligent folder management with clear user feedback and proper error handling.

**Next Phase**: Background reconciliation system will scan for discrepancies between folder locations and database status.