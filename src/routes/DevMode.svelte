<script lang="ts">
  import CompanyModal from '$lib/components/CompanyModal.svelte';
  import ContactModal from '$lib/components/ContactModal.svelte';
  import NewProjectModal from '$lib/components/NewProjectModal.svelte';

  // Form states
  let activeForm = $state<string | null>(null);
  let showCompanyForm = $state(false);
  let showContactForm = $state(false);
  let showProjectForm = $state(false);

  function handleFormClose() {
    showCompanyForm = false;
    showContactForm = false;
    showProjectForm = false;
    activeForm = null;
  }

  function openForm(formType: string) {
    // Close any open form first
    handleFormClose();

    activeForm = formType;
    switch (formType) {
      case 'company':
        showCompanyForm = true;
        break;
      case 'contact':
        showContactForm = true;
        break;
      case 'project':
        showProjectForm = true;
        break;
    }
  }

  function handleCompanySubmit() {
    handleFormClose();
  }

  function handleContactSubmit() {
    handleFormClose();
  }

  function handleProjectSubmit() {
    handleFormClose();
  }
</script>

<div class="dev-container">
  <!-- Form Selection Cards -->
  <section class="dev-section">
    <h2 class="section-title">Select Form to Test</h2>
    <div class="form-cards-grid">

      <!-- Company Form Card -->
      <button
        class="form-card"
        class:active={activeForm === 'company'}
        onclick={() => openForm('company')}
      >
        <div class="form-card-content">
          <div class="form-icon">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
            </svg>
          </div>
          <div class="form-info">
            <div class="form-title">Company Form</div>
            <div class="form-desc">Create and edit companies</div>
          </div>
        </div>
      </button>

      <!-- Contact Form Card -->
      <button
        class="form-card"
        class:active={activeForm === 'contact'}
        onclick={() => openForm('contact')}
      >
        <div class="form-card-content">
          <div class="form-icon">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
            </svg>
          </div>
          <div class="form-info">
            <div class="form-title">Contact Form</div>
            <div class="form-desc">Create and edit contacts</div>
          </div>
        </div>
      </button>

      <!-- Project Form Card -->
      <button
        class="form-card"
        class:active={activeForm === 'project'}
        onclick={() => openForm('project')}
      >
        <div class="form-card-content">
          <div class="form-icon">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
          </div>
          <div class="form-info">
            <div class="form-title">Project Form</div>
            <div class="form-desc">Create and edit projects</div>
          </div>
        </div>
      </button>
    </div>
  </section>

  <!-- Instructions Panel -->
  <section class="dev-section">
    <div class="info-panel">
      <h3 class="info-title">Testing Guidelines</h3>
      <ul class="info-list">
        <li>Click a card above to open the corresponding modal</li>
        <li>Use "DELETE ME" prefix for test data names</li>
        <li>Forms connect to the live database</li>
        <li>Press Escape or click outside to close modals</li>
      </ul>
    </div>
  </section>
</div>

<!-- Modals rendered at root level for proper z-index -->
<CompanyModal
  isOpen={showCompanyForm}
  mode="create"
  company={null}
  on:close={handleFormClose}
  on:save={handleCompanySubmit}
/>

<ContactModal
  isOpen={showContactForm}
  mode="create"
  contact={null}
  on:close={handleFormClose}
  on:save={handleContactSubmit}
/>

<NewProjectModal
  isOpen={showProjectForm}
  mode="create"
  on:close={handleFormClose}
  on:save={handleProjectSubmit}
/>

<style>
  .dev-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 24px;
  }

  .dev-section {
    margin-bottom: 32px;
  }

  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--emittiv-lighter);
    margin-bottom: 16px;
  }

  .form-cards-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }

  .form-card {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    padding: 20px;
    transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
    cursor: pointer;
    text-align: left;
    min-height: 88px;
  }

  .form-card:hover {
    border-color: var(--emittiv-light);
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(255, 153, 0, 0.1);
  }

  .form-card:focus {
    outline: none;
    border-color: var(--emittiv-splash);
    box-shadow: 0 0 0 2px rgba(255, 153, 0, 0.2);
  }

  .form-card.active {
    border-color: var(--emittiv-splash);
    background: rgba(255, 153, 0, 0.05);
  }

  .form-card-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .form-icon {
    width: 48px;
    height: 48px;
    padding: 12px;
    background: rgba(255, 153, 0, 0.1);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .form-icon svg {
    width: 24px;
    height: 24px;
    color: var(--emittiv-splash);
  }

  .form-info {
    flex: 1;
  }

  .form-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--emittiv-white);
    line-height: 1.2;
    margin-bottom: 4px;
  }

  .form-desc {
    font-size: 14px;
    color: var(--emittiv-lighter);
    line-height: 1.2;
  }

  .info-panel {
    background: var(--emittiv-darker);
    border: 1px solid var(--emittiv-dark);
    border-radius: 12px;
    padding: 20px;
  }

  .info-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--emittiv-white);
    margin-bottom: 12px;
  }

  .info-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .info-list li {
    font-size: 14px;
    color: var(--emittiv-light);
    padding: 4px 0;
    padding-left: 16px;
    position: relative;
  }

  .info-list li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 4px;
    background: var(--emittiv-splash);
    border-radius: 50%;
  }

  @media (max-width: 768px) {
    .dev-container {
      padding: 16px;
    }

    .form-cards-grid {
      grid-template-columns: 1fr;
      gap: 16px;
    }
  }
</style>
