# 📊 Status to Folder Mapping Analysis & Reconciliation Guide

## Current Implementation Status

### 🗂️ **Folder Structure**
```
/Volumes/base/emittiv/emittiv/01 Projects/
├── 00 Inactive/     (Cancelled, Lost projects)
├── 01 RFPs/         (Proposals, RFPs in progress)
├── 11 Current/      (Active, ongoing projects)
└── 99 Completed/    (Finished, delivered projects)
```

### 🔄 **Current Status-to-Folder Mapping** (in `folder_management.rs`)
```rust
match status.to_lowercase().as_str() {
    "rfp" | "proposal" | "submitted" => "01 RFPs",
    "active" | "current" | "awarded" | "ongoing" => "11 Current",
    "completed" | "finished" | "delivered" => "99 Completed",
    "cancelled" | "inactive" | "lost" => "00 Inactive",
    _ => Error("Unknown status")
}
```

## 📋 Status Lists Across Components

### **ProjectModal.svelte** (9 statuses)
1. **Draft** → ❌ Not mapped
2. **RFP** → ✅ Maps to `01 RFPs`
3. **Active** → ✅ Maps to `11 Current`
4. **Awarded** → ✅ Maps to `11 Current`
5. **Completed** → ✅ Maps to `99 Completed`
6. **Lost** → ✅ Maps to `00 Inactive`
7. **Cancelled** → ✅ Maps to `00 Inactive`
8. **On Hold** → ❌ Not mapped
9. **Revised** → ❌ Not mapped

### **ProposalModal.svelte** (9 statuses)
1. **Draft** → ❌ Not mapped
2. **Sent** → ❌ Not mapped
3. **Negotiation** → ❌ Not mapped
4. **Awarded** → ✅ Maps to `11 Current` (if linked to project)
5. **Completed** → ✅ Maps to `99 Completed` (if linked to project)
6. **Lost** → ✅ Maps to `00 Inactive` (if linked to project)
7. **Cancelled** → ✅ Maps to `00 Inactive` (if linked to project)
8. **On Hold** → ❌ Not mapped
9. **Revised** → ❌ Not mapped

### **NewProjectModal.svelte** (6 statuses)
1. **Draft** → ❌ Not mapped
2. **FP** → ❌ Not mapped (should be RFP?)
3. **Active** → ✅ Maps to `11 Current`
4. **On Hold** → ❌ Not mapped
5. **Completed** → ✅ Maps to `99 Completed`
6. **Cancelled** → ✅ Maps to `00 Inactive`

## 🚨 **Reconciliation Needed**

### **1. Unmapped Statuses**
These statuses don't trigger folder movements:
- **Draft**: Typically means project hasn't started (no folder yet?)
- **On Hold**: Project paused but not cancelled
- **Revised**: Project scope changed
- **Sent**: Proposal sent to client
- **Negotiation**: Proposal under discussion

### **2. Proposed Mapping Updates**

```rust
// Updated status mapping logic
match status.to_lowercase().as_str() {
    // RFP Stage (proposals and initial stages)
    "rfp" | "proposal" | "submitted" | "draft" | "sent" | "negotiation" | "revised" => "01 RFPs",
    
    // Active Stage (ongoing work)
    "active" | "current" | "awarded" | "ongoing" => "11 Current",
    
    // On Hold (special case - stays in current location)
    "on hold" => {
        // Don't move folder, just update database status
        // Return current location
    },
    
    // Completed Stage
    "completed" | "finished" | "delivered" => "99 Completed",
    
    // Inactive/Cancelled Stage
    "cancelled" | "inactive" | "lost" => "00 Inactive",
    
    _ => Error("Unknown status")
}
```

### **3. Business Logic Questions**

1. **Draft Status**: 
   - Should "Draft" projects have folders at all?
   - Or should they stay in `01 RFPs` until awarded?

2. **On Hold Status**:
   - Should "On Hold" projects stay in their current folder?
   - Or need a separate "02 On Hold" folder?

3. **Proposal Statuses** (Sent, Negotiation):
   - These are proposal-specific, not project statuses
   - Should proposals trigger project folder movements?
   - Or only project status changes should move folders?

4. **Revised Status**:
   - Is this for revised proposals (stays in RFP)?
   - Or revised active projects (stays in Current)?

### **4. Recommended Approach**

#### **Option A: Separate Project vs Proposal Status**
- **Projects** control folder movements
- **Proposals** don't affect folder location
- Only these project statuses trigger moves:
  - RFP → 01 RFPs
  - Active/Awarded → 11 Current
  - Completed → 99 Completed
  - Cancelled/Lost → 00 Inactive
  - Draft/On Hold/Revised → No movement

#### **Option B: Create Additional Folders**
```
00 Inactive/
01 RFPs/
02 On Hold/        (new)
03 Draft/          (new)
11 Current/
99 Completed/
```

#### **Option C: Intelligent Status Handling**
- "On Hold" → Keeps current location, adds suffix to folder name
- "Draft" → Creates folder in special "00 Draft" location
- "Revised" → Keeps current location, updates timestamp

## 🔧 **Implementation Steps**

### **1. Update Backend Mapping**
```rust
// In folder_management.rs
fn get_folder_for_status(status: &str) -> Result<&str, String> {
    match status.to_lowercase().as_str() {
        // Draft and proposal stages
        "draft" | "rfp" | "proposal" | "submitted" | "sent" | "negotiation" | "revised" => Ok("01 RFPs"),
        
        // Active work
        "active" | "current" | "awarded" | "ongoing" => Ok("11 Current"),
        
        // Completed work
        "completed" | "finished" | "delivered" => Ok("99 Completed"),
        
        // Cancelled/Lost work
        "cancelled" | "inactive" | "lost" => Ok("00 Inactive"),
        
        // Special case - don't move
        "on hold" => Ok("no_move"),
        
        _ => Err(format!("Unknown status: {}", status))
    }
}
```

### **2. Update StatusChangeModal Logic**
- Add special handling for "On Hold" status
- Show different message for non-moving statuses
- Clarify which statuses trigger folder movements

### **3. Database Considerations**
- Some statuses might need project creation without folders
- Consider adding a "has_folder" flag to projects table
- Track folder location separately from status

### **4. Testing Requirements**
- Test each status transition
- Verify "On Hold" doesn't move folders
- Ensure "Draft" projects handle correctly
- Test proposal status changes don't affect project folders

## 📊 **Status Flow Diagram**

```
Draft → RFP → Active → Completed
  ↓      ↓      ↓         ↓
  ↓      ↓   On Hold      ↓
  ↓      ↓      ↓         ↓
  └──────┴──────┴─────────┴───→ Cancelled/Lost

Folder Movements:
- Draft: Creates in 01 RFPs (or no folder?)
- RFP → Active: Move from 01 RFPs → 11 Current (+ templates)
- Active → Completed: Move from 11 Current → 99 Completed
- Any → Cancelled/Lost: Move to 00 Inactive
- Any → On Hold: No movement (status only)
```

## 🎯 **Recommendations**

1. **Clarify Business Rules**: 
   - When should project folders be created?
   - Which statuses should trigger movements?
   - How to handle "On Hold" projects?

2. **Separate Concerns**:
   - Project status (controls folders)
   - Proposal status (doesn't control folders)

3. **Add Validation**:
   - Prevent folder movements for certain statuses
   - Add warnings for irreversible moves

4. **Enhanced UI Feedback**:
   - Show which statuses move folders
   - Indicate "no movement" statuses
   - Preview folder operations

Would you like me to implement any of these reconciliation options?