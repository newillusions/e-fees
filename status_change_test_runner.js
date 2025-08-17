// Status Change Test Runner - Simplified for Manual Execution
// Execute this in the browser console of the running Tauri app

console.log('🎯 Status Change Test Runner Loading...');

// Simple test function that can be run in browser console
function runStatusChangeTest() {
  console.log('🚀 Starting Status Change Test...');
  
  // Step 1: Navigate to Proposals
  console.log('Step 1: Navigating to Proposals...');
  const proposalsBtn = Array.from(document.querySelectorAll('button')).find(btn => 
    btn.textContent.includes('Proposals')
  );
  
  if (!proposalsBtn) {
    console.error('❌ Proposals button not found');
    return;
  }
  
  proposalsBtn.click();
  console.log('✅ Clicked Proposals button');
  
  // Step 2: Wait and find test proposal
  setTimeout(() => {
    console.log('Step 2: Looking for test proposal...');
    
    // Look for our test proposal
    const proposalCards = document.querySelectorAll('[data-testid], .cursor-pointer, .list-card');
    const testProposal = Array.from(proposalCards).find(card => 
      card.textContent.includes('Test Proposal') || 
      card.textContent.includes('RFP-TEST-001') ||
      card.textContent.includes('Status Change')
    );
    
    if (!testProposal) {
      console.error('❌ Test proposal not found');
      console.log('Available proposals:', Array.from(document.querySelectorAll('*')).filter(el => 
        el.textContent.includes('RFP-') || el.textContent.includes('Draft')
      ).map(el => el.textContent.slice(0, 100)));
      return;
    }
    
    testProposal.click();
    console.log('✅ Clicked test proposal');
    
    // Step 3: Enter edit mode
    setTimeout(() => {
      console.log('Step 3: Entering edit mode...');
      
      const editBtn = Array.from(document.querySelectorAll('button')).find(btn => 
        btn.innerHTML.includes('svg') && (
          btn.getAttribute('aria-label') === 'Edit' ||
          btn.title === 'Edit' ||
          btn.className.includes('hover:text-emittiv-splash')
        )
      );
      
      if (!editBtn) {
        console.error('❌ Edit button not found');
        return;
      }
      
      editBtn.click();
      console.log('✅ Clicked edit button');
      
      // Step 4: Change status
      setTimeout(() => {
        console.log('Step 4: Changing status from Draft to Sent...');
        
        const statusSelect = Array.from(document.querySelectorAll('select')).find(select => 
          select.value === 'Draft' || 
          select.options[0]?.value === 'Draft' ||
          select.parentElement?.textContent?.includes('Status')
        );
        
        if (!statusSelect) {
          console.error('❌ Status select not found');
          console.log('Available selects:', Array.from(document.querySelectorAll('select')).map(s => s.outerHTML));
          return;
        }
        
        console.log('Found status select:', statusSelect.value);
        statusSelect.value = 'Sent';
        statusSelect.dispatchEvent(new Event('change', { bubbles: true }));
        statusSelect.dispatchEvent(new Event('input', { bubbles: true }));
        console.log('✅ Changed status to Sent');
        
        // Step 5: Submit form
        setTimeout(() => {
          console.log('Step 5: Submitting form...');
          
          const submitBtn = Array.from(document.querySelectorAll('button')).find(btn => 
            btn.textContent.includes('Update') || 
            btn.textContent.includes('Save') ||
            btn.type === 'submit'
          );
          
          if (!submitBtn) {
            console.error('❌ Submit button not found');
            console.log('Available buttons:', Array.from(document.querySelectorAll('button')).map(b => 
              `"${b.textContent.trim()}" (type: ${b.type})`
            ));
            return;
          }
          
          submitBtn.click();
          console.log('✅ Clicked submit button');
          
          // Step 6: Check for StatusChangeModal
          setTimeout(() => {
            console.log('Step 6: Checking for StatusChangeModal...');
            
            const modal = document.querySelector('[role="dialog"]') ||
                         Array.from(document.querySelectorAll('*')).find(el => 
                           el.textContent.includes('Status Change') ||
                           el.textContent.includes('Confirm') ||
                           el.textContent.includes('Folder Movement') ||
                           el.textContent.includes('Project Status')
                         );
            
            if (modal) {
              console.log('🎉 SUCCESS: StatusChangeModal appeared!');
              console.log('Modal content:', modal.textContent.slice(0, 300));
              
              // Check for confirm button
              const confirmBtn = Array.from(modal.querySelectorAll('button')).find(btn =>
                btn.textContent.includes('Confirm') || 
                btn.textContent.includes('Yes') ||
                btn.className.includes('primary')
              );
              
              if (confirmBtn) {
                console.log('✅ Found confirm button, clicking...');
                confirmBtn.click();
                
                setTimeout(() => {
                  const modalGone = !document.querySelector('[role="dialog"]');
                  console.log(modalGone ? '✅ Modal closed successfully' : '⚠️ Modal still visible');
                }, 1000);
              }
              
            } else {
              console.error('❌ CRITICAL: StatusChangeModal did NOT appear!');
              console.log('This means the status change trigger logic is not working');
            }
            
          }, 2000);
          
        }, 500);
        
      }, 500);
      
    }, 1000);
    
  }, 1000);
}

// Make function available globally
window.runStatusChangeTest = runStatusChangeTest;

console.log('✅ Test runner loaded. Execute: runStatusChangeTest()');