// Comprehensive Test for Proposal Status Change Functionality
// This script tests all status change scenarios and verifies the enhanced modal

class ProposalStatusTester {
  constructor() {
    this.testResults = [];
    this.currentStep = 0;
  }
  
  log(message, type = 'info') {
    const timestamp = new Date().toLocaleTimeString();
    const logEntry = `[${timestamp}] ${type.toUpperCase()}: ${message}`;
    console.log(logEntry);
    this.testResults.push({ message, type, timestamp });
  }
  
  async wait(ms = 1000) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  async clickElement(selector, description) {
    try {
      const element = document.querySelector(selector);
      if (element) {
        element.click();
        this.log(`✓ Clicked ${description}`, 'success');
        return true;
      } else {
        this.log(`✗ ${description} not found (${selector})`, 'error');
        return false;
      }
    } catch (error) {
      this.log(`✗ Error clicking ${description}: ${error.message}`, 'error');
      return false;
    }
  }
  
  async navigateToProposals() {
    this.log('Step 1: Navigating to Proposals page...');
    
    // Wait for app to load
    await this.wait(3000);
    
    // Find and click Proposals button
    const proposalsBtn = Array.from(document.querySelectorAll('button')).find(btn => 
      btn.textContent.includes('Proposals')
    );
    
    if (proposalsBtn) {
      proposalsBtn.click();
      await this.wait(1000);
      this.log('✓ Successfully navigated to Proposals page', 'success');
      return true;
    } else {
      this.log('✗ Proposals navigation button not found', 'error');
      return false;
    }
  }
  
  async openFirstProposal() {
    this.log('Step 2: Opening first proposal for editing...');
    
    await this.wait(500);
    
    // Find first proposal card
    const proposalCard = document.querySelector('.list-card, .cursor-pointer, [data-proposal]') ||
                         Array.from(document.querySelectorAll('*')).find(el => 
                           el.textContent.includes('RFP-') && el.style.cursor === 'pointer'
                         );
    
    if (proposalCard) {
      proposalCard.click();
      await this.wait(1500);
      this.log('✓ Proposal modal opened', 'success');
      return true;
    } else {
      this.log('✗ No clickable proposal found', 'error');
      return false;
    }
  }
  
  async enterEditMode() {
    this.log('Step 3: Entering edit mode...');
    
    await this.wait(500);
    
    // Look for edit button (pencil icon)
    const editButtons = Array.from(document.querySelectorAll('button')).filter(btn => 
      btn.innerHTML.includes('<svg') && 
      (btn.className.includes('hover:text-emittiv-splash') || 
       btn.getAttribute('aria-label') === 'Edit' ||
       btn.title === 'Edit')
    );
    
    if (editButtons.length > 0) {
      editButtons[0].click();
      await this.wait(1000);
      this.log('✓ Entered edit mode', 'success');
      return true;
    } else {
      this.log('✗ Edit button not found', 'error');
      return false;
    }
  }
  
  async changeProposalStatus(fromStatus, toStatus) {
    this.log(`Step 4: Testing status change from ${fromStatus} to ${toStatus}...`);
    
    await this.wait(500);
    
    // Find status dropdown
    const statusSelect = Array.from(document.querySelectorAll('select')).find(select => 
      select.value === fromStatus || 
      select.parentElement?.textContent?.includes('Status') ||
      select.previousElementSibling?.textContent?.includes('Status')
    );
    
    if (statusSelect) {
      const originalValue = statusSelect.value;
      this.log(`   Original status: ${originalValue}`);
      
      // Change status
      statusSelect.value = toStatus;
      statusSelect.dispatchEvent(new Event('change', { bubbles: true }));
      statusSelect.dispatchEvent(new Event('input', { bubbles: true }));
      
      this.log(`✓ Changed status from ${originalValue} to ${toStatus}`, 'success');
      return true;
    } else {
      this.log('✗ Status select field not found', 'error');
      return false;
    }
  }
  
  async submitForm() {
    this.log('Step 5: Submitting form...');
    
    await this.wait(500);
    
    // Look for Update/Submit button
    const submitBtn = Array.from(document.querySelectorAll('button')).find(btn => 
      btn.textContent.trim().includes('Update') || 
      btn.textContent.trim().includes('Save') ||
      btn.type === 'submit'
    );
    
    if (submitBtn) {
      submitBtn.click();
      this.log('✓ Form submitted', 'success');
      return true;
    } else {
      this.log('✗ Submit button not found', 'error');
      this.log('   Available buttons: ' + 
        Array.from(document.querySelectorAll('button')).map(b => 
          `"${b.textContent.trim()}" (type: ${b.type})`
        ).join(', '));
      return false;
    }
  }
  
  async checkForStatusChangeModal() {
    this.log('Step 6: Checking for enhanced StatusChangeModal...');
    
    await this.wait(2000); // Give time for modal to appear
    
    // Look for status change modal
    const statusModal = document.querySelector('[role="dialog"]') ||
                       Array.from(document.querySelectorAll('*')).find(el => 
                         el.textContent.includes('Confirm Status Change') ||
                         el.textContent.includes('Status Change') ||
                         el.textContent.includes('Folder Movement') ||
                         el.textContent.includes('Project Status') ||
                         el.textContent.includes('Downstream Impact')
                       );
    
    if (statusModal) {
      this.log('✓ SUCCESS: Enhanced StatusChangeModal appeared!', 'success');
      this.log(`   Modal content preview: ${statusModal.textContent.slice(0, 200)}...`);
      
      // Check for key features
      const hasProjectInfo = statusModal.textContent.includes('Project');
      const hasFolderInfo = statusModal.textContent.includes('Folder');
      const hasImpactInfo = statusModal.textContent.includes('Impact') || statusModal.textContent.includes('proposal');
      
      this.log(`   ✓ Contains project information: ${hasProjectInfo}`, hasProjectInfo ? 'success' : 'warning');
      this.log(`   ✓ Contains folder movement info: ${hasFolderInfo}`, hasFolderInfo ? 'success' : 'warning');
      this.log(`   ✓ Contains impact analysis: ${hasImpactInfo}`, hasImpactInfo ? 'success' : 'warning');
      
      return statusModal;
    } else {
      this.log('✗ FAILURE: Enhanced StatusChangeModal did not appear', 'error');
      this.log('   This indicates the status change trigger logic failed');
      return null;
    }
  }
  
  async testModalConfirmation(modal) {
    this.log('Step 7: Testing modal confirmation...');
    
    if (!modal) return false;
    
    // Find confirm button
    const confirmBtn = Array.from(modal.querySelectorAll('button')).find(btn =>
      btn.textContent.includes('Confirm') || 
      btn.textContent.includes('Yes') ||
      btn.className.includes('primary')
    );
    
    if (confirmBtn) {
      confirmBtn.click();
      await this.wait(1000);
      this.log('✓ Clicked confirm button', 'success');
      
      // Check if modal closes and success message appears
      await this.wait(1000);
      const modalStillThere = document.querySelector('[role="dialog"]');
      if (!modalStillThere) {
        this.log('✓ Modal closed after confirmation', 'success');
        return true;
      } else {
        this.log('⚠ Modal still visible after confirmation', 'warning');
        return false;
      }
    } else {
      this.log('✗ Confirm button not found in modal', 'error');
      return false;
    }
  }
  
  async runStatusChangeTest(fromStatus, toStatus) {
    this.log(`\n=== TESTING STATUS CHANGE: ${fromStatus} → ${toStatus} ===`);
    
    try {
      // Reset to fresh state
      location.reload();
      await this.wait(3000);
      
      const success1 = await this.navigateToProposals();
      if (!success1) return false;
      
      const success2 = await this.openFirstProposal();
      if (!success2) return false;
      
      const success3 = await this.enterEditMode();
      if (!success3) return false;
      
      const success4 = await this.changeProposalStatus(fromStatus, toStatus);
      if (!success4) return false;
      
      const success5 = await this.submitForm();
      if (!success5) return false;
      
      const modal = await this.checkForStatusChangeModal();
      const success6 = modal !== null;
      
      if (success6) {
        await this.testModalConfirmation(modal);
      }
      
      this.log(`=== TEST RESULT: ${success6 ? 'PASSED' : 'FAILED'} ===\n`);
      return success6;
      
    } catch (error) {
      this.log(`✗ Test failed with error: ${error.message}`, 'error');
      return false;
    }
  }
  
  async runFullTestSuite() {
    this.log('🚀 Starting Comprehensive Proposal Status Change Test Suite');
    this.log('This will test the enhanced StatusChangeModal functionality\n');
    
    const testCases = [
      ['Draft', 'Sent'],
      ['Sent', 'Lost'],     // This was the original failing case
      ['Draft', 'Lost'],
      ['Sent', 'Awarded']
    ];
    
    const results = [];
    
    for (const [from, to] of testCases) {
      const result = await this.runStatusChangeTest(from, to);
      results.push({ from, to, passed: result });
      await this.wait(2000); // Brief pause between tests
    }
    
    // Summary
    this.log('\n📊 TEST SUITE RESULTS:');
    this.log('='.repeat(50));
    
    results.forEach(({ from, to, passed }) => {
      const status = passed ? '✅ PASSED' : '❌ FAILED';
      this.log(`${from} → ${to}: ${status}`);
    });
    
    const passedCount = results.filter(r => r.passed).length;
    const totalCount = results.length;
    
    this.log(`\nOverall: ${passedCount}/${totalCount} tests passed`);
    
    if (passedCount === totalCount) {
      this.log('🎉 ALL TESTS PASSED! Status change functionality is working correctly.', 'success');
    } else {
      this.log('⚠️ Some tests failed. Status change functionality needs debugging.', 'warning');
    }
    
    return results;
  }
}

// Create and run the test
window.statusTester = new ProposalStatusTester();

// Export for manual execution
window.runStatusTests = () => window.statusTester.runFullTestSuite();

// Auto-run if page is loaded
if (document.readyState === 'complete') {
  console.log('🎯 Comprehensive Status Change Test Script Loaded');
  console.log('📋 Run window.runStatusTests() to start testing');
} else {
  window.addEventListener('load', () => {
    console.log('🎯 Comprehensive Status Change Test Script Loaded');
    console.log('📋 Run window.runStatusTests() to start testing');
  });
}