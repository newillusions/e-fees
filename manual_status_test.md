# Manual Status Change Testing Guide

## IMMEDIATE TESTING STEPS

### Prerequisites Check
1. ✅ Tauri app is running (`npm run tauri:dev` or built app)
2. ✅ Test data exists:
   - Company: emittiv (ID: company:43mmoqx2jr5p32m4hffd)
   - Contact: Martin Robert (ID: contacts:kuo9rron36efqqa2vxk8)  
   - Project: Status Change Test Project (ID: projects:xigy1t9623hw1h33f59h)
   - Proposal: Test Proposal for Status Changes (ID: fee:d351bfcgn9uxpccdgm2l)

### Test Execution

#### Step 1: Navigate to Proposals
1. Open the Tauri app (should be running)
2. Click "Proposals" in the sidebar navigation
3. **EXPECTED**: See list of proposals including "Test Proposal for Status Changes"

#### Step 2: Open Test Proposal  
1. Find "Test Proposal for Status Changes" or "RFP-TEST-001" 
2. Click on the proposal card
3. **EXPECTED**: Proposal modal opens with proposal details

#### Step 3: Enter Edit Mode
1. Look for pencil/edit icon button in the modal
2. Click the edit button  
3. **EXPECTED**: Form fields become editable

#### Step 4: Change Status (CRITICAL TEST)
1. Find the Status dropdown (should currently be "Draft")
2. Change status from "Draft" to "Sent"
3. **EXPECTED**: Status dropdown updates to "Sent"

#### Step 5: Submit Form
1. Click "Update" or "Save" button
2. **EXPECTED**: Form submits successfully

#### Step 6: StatusChangeModal Check (KEY VALIDATION)
**THIS IS THE CRITICAL TEST - SHOULD HAPPEN IMMEDIATELY AFTER SUBMIT**

**EXPECTED RESULT**: StatusChangeModal should appear with:
- Title containing "Status Change" or "Confirm"
- Information about folder movement
- Project status synchronization options
- Confirmation button

**FAILURE INDICATORS**:
- ❌ No modal appears
- ❌ Regular success message only
- ❌ Form just closes without status change modal

### Alternative Test: Browser Console Method

If manual clicking is difficult, paste this into the browser console:

```javascript
// Copy the content of status_change_test_runner.js into console
// Then execute: runStatusChangeTest()
```

### Debugging Commands

If the test fails, run these in the console:

```javascript
// Check if proposals are loaded
console.log('Proposals:', window.stores?.feeProposals || 'Not found');

// Check for test proposal specifically  
console.log('Test proposal:', document.querySelector('*')?.textContent?.includes('RFP-TEST-001'));

// Check current route
console.log('Current URL:', window.location.href);

// Check for any JavaScript errors
console.log('Last errors:', window.console?.errors || 'No errors tracked');
```

### Success Criteria

✅ **FULL SUCCESS**: StatusChangeModal appears and shows:
- Project information
- Folder movement details  
- Bidirectional status sync options
- Successful confirmation and execution

⚠️ **PARTIAL SUCCESS**: Status change works but no modal (regression)

❌ **FAILURE**: Status change doesn't work at all

### Next Status Combinations to Test

After Draft→Sent succeeds, test:
1. Sent → Lost (original failing case)
2. Sent → Awarded
3. Draft → Lost
4. Draft → Awarded

Each should trigger the StatusChangeModal with appropriate folder/project operations.