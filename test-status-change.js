// Test script for proposal status change functionality
// Run this in the browser console when the proposals page is open

async function testProposalStatusChange() {
  console.log('=== Testing Proposal Status Change Functionality ===');
  
  // Step 1: Navigate to proposals page
  console.log('1. Navigating to Proposals page...');
  const proposalsButton = Array.from(document.querySelectorAll('button')).find(btn => 
    btn.textContent.includes('Proposals')
  );
  if (proposalsButton) {
    proposalsButton.click();
    await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for navigation
    console.log('✓ Navigated to Proposals page');
  } else {
    console.error('✗ Proposals button not found');
    return;
  }
  
  // Step 2: Open first proposal for editing
  console.log('2. Opening first proposal...');
  await new Promise(resolve => setTimeout(resolve, 500));
  const proposalCard = document.querySelector('.group.list-card.cursor-pointer');
  if (proposalCard) {
    proposalCard.click();
    await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for modal to open
    console.log('✓ Proposal modal opened');
  } else {
    console.error('✗ Proposal card not found');
    return;
  }
  
  // Step 3: Switch to edit mode
  console.log('3. Switching to edit mode...');
  await new Promise(resolve => setTimeout(resolve, 500));
  const editButtons = Array.from(document.querySelectorAll('button')).filter(btn => 
    btn.innerHTML.includes('<svg') && 
    btn.className.includes('p-1') && 
    btn.className.includes('hover:text-emittiv-splash')
  );
  if (editButtons.length > 0) {
    editButtons[0].click(); // Click edit button
    await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for edit mode
    console.log('✓ Switched to edit mode');
  } else {
    console.error('✗ Edit button not found');
    return;
  }
  
  // Step 4: Find and change status
  console.log('4. Changing proposal status...');
  await new Promise(resolve => setTimeout(resolve, 500));
  const statusSelect = Array.from(document.querySelectorAll('select')).find(select => 
    select.value === 'Draft' || 
    select.parentElement?.textContent?.includes('Status')
  );
  
  if (statusSelect) {
    const originalStatus = statusSelect.value;
    console.log(`   Original status: ${originalStatus}`);
    
    // Change to "Sent" status
    statusSelect.value = 'Sent';
    statusSelect.dispatchEvent(new Event('change', { bubbles: true }));
    statusSelect.dispatchEvent(new Event('input', { bubbles: true }));
    console.log('✓ Changed status to "Sent"');
    
    // Step 5: Submit form
    console.log('5. Submitting form...');
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Look for Update button
    const updateButton = Array.from(document.querySelectorAll('button')).find(btn => 
      btn.textContent.trim() === 'Update' && 
      btn.type === 'submit'
    );
    
    if (updateButton) {
      updateButton.click();
      console.log('✓ Clicked Update button');
      
      // Step 6: Check if StatusChangeModal appears
      console.log('6. Checking for StatusChangeModal...');
      await new Promise(resolve => setTimeout(resolve, 2000)); // Wait for modal
      
      const statusChangeModal = document.querySelector('[role="dialog"]') || 
                               Array.from(document.querySelectorAll('*')).find(el => 
                                 el.textContent.includes('Confirm Status Change') ||
                                 el.textContent.includes('Status Change') ||
                                 el.textContent.includes('Folder Movement')
                               );
      
      if (statusChangeModal) {
        console.log('✓ SUCCESS: StatusChangeModal appeared!');
        console.log('   Modal content preview:', statusChangeModal.textContent.slice(0, 200));
        
        // Look for confirm button and click it to test the full flow
        const confirmButton = Array.from(statusChangeModal.querySelectorAll('button')).find(btn =>
          btn.textContent.includes('Confirm')
        );
        
        if (confirmButton) {
          console.log('7. Testing confirm functionality...');
          confirmButton.click();
          await new Promise(resolve => setTimeout(resolve, 1000));
          console.log('✓ Clicked confirm button');
        }
        
      } else {
        console.error('✗ FAILURE: StatusChangeModal did not appear');
        console.log('   This indicates the status change trigger logic is not working');
      }
      
    } else {
      console.error('✗ Update button not found');
      console.log('   Available buttons:', Array.from(document.querySelectorAll('button')).map(b => b.textContent.trim()));
    }
    
  } else {
    console.error('✗ Status select field not found');
    console.log('   Available selects:', Array.from(document.querySelectorAll('select')).map(s => s.value));
  }
  
  console.log('=== Test Complete ===');
}

// Run the test
testProposalStatusChange().catch(console.error);