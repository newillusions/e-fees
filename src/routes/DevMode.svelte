<script lang="ts">
  import CompanyModal from '$lib/components/CompanyModal.svelte';
  import ContactModal from '$lib/components/ContactModal.svelte';
  import NewProjectModal from '$lib/components/NewProjectModal.svelte';
  import Card from '$lib/components/Card.svelte';
  import { companiesActions, contactsActions, projectsActions } from '$lib/stores';
  
  // Form states for testing - AUTO-OPEN COMPANY FORM FOR DEMO
  let activeForm = $state('company');
  let showCompanyForm = $state(true);
  let showContactForm = $state(false);
  let showProjectForm = $state(false);

  // Form data for testing
  let companyData = $state({
    name: 'DELETE ME Test Company Ltd',
    name_short: 'DELETE ME TestCo',
    abbreviation: 'DMTC',
    country: 'United Arab Emirates',
    city: 'Dubai',
    address: '123 Test Street, Business Bay',
    phone: '+971 4 123 4567',
    email: 'test@deletecompany.com',
    website: 'www.deletecompany.com',
    reg_no: 'REG-DELETE-123456',
    tax_no: 'VAT-DELETE-789012'
  });
  
  // Mock company for editing test
  const mockCompany = {
    id: 'test-company-1',
    name: 'Test Company',
    name_short: 'TestCo',
    abbreviation: 'TC',
    city: 'Dubai',
    country: 'United Arab Emirates',
    address: '123 Test Street',
    phone: '+971 4 123 4567',
    email: 'test@company.com',
    website: 'www.testcompany.com',
    reg_no: 'REG123456',
    tax_no: 'TAX789012'
  };
  
  async function handleFormSubmit(event: Event) {
    event.preventDefault();

    if (activeForm === 'company') {
      try {
        console.log('DEV MODE: Submitting company data:', companyData);
        alert(`🔄 DEV MODE: Submitting company "${companyData.name}" to database...\n\nData: ${JSON.stringify(companyData, null, 2)}`);

        const result = await companiesActions.create(companyData);
        console.log('DEV MODE: Company creation result:', result);

        alert(`✅ SUCCESS: Company "${companyData.name}" created successfully!\nResult: ${JSON.stringify(result, null, 2)}`);

        // Reset form
        handleFormClose();

      } catch (error) {
        console.error('DEV MODE: Company creation error:', error);
        alert(`❌ ERROR: Failed to create company\nError: ${error.message || error}`);
      }
    } else {
      console.log('DEV MODE: Form submitted for:', activeForm);
      alert(`DEV MODE: ${activeForm.toUpperCase()} form submitted successfully!\nCheck console for details.`);
    }
  }
  
  function handleFormClose() {
    console.log('DEV MODE: handleFormClose called');
    showCompanyForm = false;
    showContactForm = false;
    showProjectForm = false;
    console.log('DEV MODE: Form states after close - Company:', showCompanyForm, 'Contact:', showContactForm, 'Project:', showProjectForm);
  }
  
  function openForm(formType: string) {
    console.log('DEV MODE: Opening form type:', formType);
    activeForm = formType;
    switch (formType) {
      case 'company':
        showCompanyForm = true;
        console.log('DEV MODE: Company form should be open, showCompanyForm =', showCompanyForm);
        break;
      case 'contact':
        showContactForm = true;
        console.log('DEV MODE: Contact form should be open, showContactForm =', showContactForm);
        break;
      case 'project':
        showProjectForm = true;
        console.log('DEV MODE: Project form should be open, showProjectForm =', showProjectForm);
        break;
    }
  }
</script>

<div class="p-8">
  <div class="max-w-4xl mx-auto">
    <!-- Dev Mode Header -->
    <div class="mb-8">
      <h1 class="text-3xl font-bold text-emittiv-white mb-2">🛠️ Development Mode</h1>
      <p class="text-emittiv-lighter">Test forms and modals in standalone mode without z-index issues</p>
    </div>
    
    <!-- Form Selection -->
    <div class="mb-8">
      <h2 class="text-xl font-semibold text-emittiv-white mb-4">Select Form to Test</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        
        <!-- Company Form -->
        <Card>
          <div class="text-center p-6">
            <div class="w-12 h-12 bg-green-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg class="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
              </svg>
            </div>
            <h3 class="text-lg font-medium text-emittiv-white mb-2">Company Form</h3>
            <p class="text-sm text-emittiv-light mb-4">Test company creation and editing</p>
            <button
              class="w-full px-4 py-2 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded-lg transition-colors"
              on:click|stopPropagation={() => openForm('company')}
            >
              Open Company Form
            </button>
          </div>
        </Card>
        
        <!-- Contact Form -->
        <Card>
          <div class="text-center p-6">
            <div class="w-12 h-12 bg-blue-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg class="w-6 h-6 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0z" />
              </svg>
            </div>
            <h3 class="text-lg font-medium text-emittiv-white mb-2">Contact Form</h3>
            <p class="text-sm text-emittiv-light mb-4">Test contact creation and editing</p>
            <button
              class="w-full px-4 py-2 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded-lg transition-colors"
              on:click|stopPropagation={() => openForm('contact')}
            >
              Open Contact Form
            </button>
          </div>
        </Card>
        
        <!-- Project Form -->
        <Card>
          <div class="text-center p-6">
            <div class="w-12 h-12 bg-purple-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg class="w-6 h-6 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </div>
            <h3 class="text-lg font-medium text-emittiv-white mb-2">Project Form</h3>
            <p class="text-sm text-emittiv-light mb-4">Test project creation and editing</p>
            <button
              class="w-full px-4 py-2 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded-lg transition-colors"
              on:click|stopPropagation={() => openForm('project')}
            >
              Open Project Form
            </button>
          </div>
        </Card>
      </div>
    </div>
    
    <!-- Active Form Display -->
    {#if showCompanyForm || showContactForm || showProjectForm}
      <div class="mb-8">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-emittiv-white">Active Form: {activeForm.toUpperCase()}</h2>
          <button
            class="px-3 py-1 bg-emittiv-dark hover:bg-emittiv-light text-emittiv-lighter rounded"
            on:click|stopPropagation={() => handleFormClose()}
          >
            Close Form
          </button>
        </div>
        
        <!-- Form Container -->
        <div class="bg-emittiv-darker border border-emittiv-dark rounded-lg p-6">
          <div class="mb-4 p-4 bg-green-800/20 border border-green-600 rounded">
            <h3 class="text-green-400 font-semibold mb-2">🟢 Form Debug Info</h3>
            <p class="text-sm text-green-300">
              • showCompanyForm: {showCompanyForm}<br/>
              • Form should render below this section<br/>
              • Look for Company Information fields
            </p>
          </div>

          {#if showCompanyForm}
            <div class="border-2 border-green-500 p-6 mb-4 bg-emittiv-dark rounded-lg">
              <h4 class="text-green-400 font-semibold mb-4">📝 Company Form - Ready for Review</h4>
              <p class="text-green-300 text-sm mb-4">✅ Form filled with test data - Please review before submission</p>

              <!-- Direct Form Fields for Testing -->
              <form class="space-y-4">
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Company Name *</label>
                    <input
                      type="text"
                      bind:value={companyData.name}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="Full company name"
                    />
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Short Name *</label>
                    <input
                      type="text"
                      bind:value={companyData.name_short}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="Short name"
                    />
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Abbreviation *</label>
                    <input
                      type="text"
                      bind:value={companyData.abbreviation}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="ABC"
                    />
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Country *</label>
                    <input
                      type="text"
                      bind:value={companyData.country}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="Search countries..."
                    />
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">City *</label>
                    <input
                      type="text"
                      bind:value={companyData.city}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="Search cities..."
                    />
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Registration No.</label>
                    <input
                      type="text"
                      bind:value={companyData.reg_no}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="Company registration number (optional)"
                    />
                  </div>
                </div>

                <div>
                  <label class="block text-sm font-medium text-emittiv-white mb-2">Tax/VAT No.</label>
                  <input
                    type="text"
                    bind:value={companyData.tax_no}
                    class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                    placeholder="Tax identification number (optional)"
                  />
                </div>

                <!-- Additional Fields ---->
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Address</label>
                    <input
                      type="text"
                      bind:value={companyData.address}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="Full address (optional)"
                    />
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Phone</label>
                    <input
                      type="tel"
                      bind:value={companyData.phone}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="+971 4 123 4567"
                    />
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Email</label>
                    <input
                      type="email"
                      bind:value={companyData.email}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="company@example.com"
                    />
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-emittiv-white mb-2">Website</label>
                    <input
                      type="url"
                      bind:value={companyData.website}
                      class="w-full px-3 py-2 bg-emittiv-darker border border-emittiv-light rounded-md text-emittiv-white placeholder-emittiv-light focus:outline-none focus:ring-2 focus:ring-emittiv-splash"
                      placeholder="www.company.com"
                    />
                  </div>
                </div>

                <!-- Form Actions -->
                <div class="flex justify-between pt-4 border-t border-emittiv-light">
                  <button
                    type="button"
                    class="px-4 py-2 bg-emittiv-dark hover:bg-emittiv-light text-emittiv-lighter rounded-lg transition-colors"
                    on:click|stopPropagation={() => handleFormClose()}
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    class="px-6 py-2 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded-lg transition-colors font-semibold"
                    on:click|stopPropagation|preventDefault={() => handleFormSubmit(event)}
                  >
                    Create Company
                  </button>
                </div>
              </form>
            </div>
          {:else}
            <div class="p-4 bg-gray-800 border border-gray-600 rounded">
              <p class="text-gray-400">Form not open (showCompanyForm = false)</p>
            </div>
          {/if}
          
          {#if showContactForm}
            <ContactModal 
              isOpen={true}
              onSubmit={handleFormSubmit}
              onClose={handleFormClose}
              mode="create"
              contact={null}
            />
          {/if}
          
          {#if showProjectForm}
            <NewProjectModal 
              isOpen={true}
              onSubmit={handleFormSubmit}
              onClose={handleFormClose}
              mode="create"
            />
          {/if}
        </div>
      </div>
    {/if}
    
    <!-- Development Notes -->
    <div class="mt-8 p-4 bg-emittiv-darker border border-emittiv-dark rounded-lg">
      <h3 class="text-lg font-medium text-emittiv-white mb-2">🔧 Development Notes</h3>
      <ul class="text-sm text-emittiv-light space-y-1">
        <li>• Forms are rendered inline without modal overlays for easier testing</li>
        <li>• All form submissions are logged to console and show alerts</li>
        <li>• Forms use real store actions for database operations</li>
        <li>• Create test data with "DELETE ME" prefixes for easy cleanup</li>
        <li>• Close dev server when testing is complete</li>
      </ul>
    </div>
  </div>
</div>

<style>
  /* Ensure forms render properly in dev mode */
  :global(.modal-content) {
    position: relative !important;
    transform: none !important;
    width: 100% !important;
    max-width: none !important;
    margin: 0 !important;
    box-shadow: none !important;
  }
  
  :global(.modal-overlay) {
    position: relative !important;
    background: transparent !important;
    padding: 0 !important;
  }
</style>