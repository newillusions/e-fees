<!--
  Refactored Proposal Modal using BaseModal, FormInput, FormSelect, and TypeaheadSelect components
  Reduced from ~790 lines to ~480 lines using base components and utilities
-->
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { feesStore, feesActions, projectsActions, projectsStore, companiesStore, contactsStore } from '$lib/stores';
  import { settingsStore } from '$lib/stores/settings';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { validateForm, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import { writeFeeToJsonSafe } from '$lib/api';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import TypeaheadSelect from './TypeaheadSelect.svelte';
  import Button from './Button.svelte';
  import NewProjectModal from './NewProjectModal.svelte';
  import CompanyModal from './CompanyModal.svelte';
  import ContactModal from './ContactModal.svelte';
  import type { Fee, Project, Company, Contact } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let proposal: Fee | null = null;
  export let mode: 'create' | 'edit' = 'create';
  
  // Debug logging
  onMount(() => {
    console.log('ProposalModal mounted with props:', { isOpen, proposal, mode });
  });
  
  $: {
    console.log('ProposalModal reactive props changed:', { isOpen, proposal, mode });
    if (proposal) {
      console.log('ProposalModal: proposal.id in reactive block:', proposal.id);
    }
  }
  
  // Enhanced prop tracking
  $: if (proposal !== null && proposal !== undefined) {
    console.log('ProposalModal: Proposal prop is NOT null/undefined, id:', proposal.id);
  }
  
  $: if (proposal === null) {
    console.log('ProposalModal: WARNING - Proposal prop is NULL');
  }
  
  $: if (proposal === undefined) {
    console.log('ProposalModal: WARNING - Proposal prop is UNDEFINED');
  }
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Form data with better typing
  interface ProposalFormData {
    number: string;
    name: string;
    issue_date: string;
    rev: string;
    status: 'Draft' | 'Sent' | 'Negotiation' | 'Awarded' | 'Completed' | 'Lost' | 'Cancelled' | 'On Hold' | 'Revised';
    package: string;
    staff_name: string;
    project_id: string;
    company_id: string;
    contact_id: string;
    activity: string;
    strap_line: string;
    staff_email: string;
    staff_phone: string;
    staff_position: string;
  }
  
  let formData: ProposalFormData = {
    number: '',
    name: 'Fee Proposal',
    issue_date: '',
    rev: '1',
    status: 'Draft',
    package: '',
    staff_name: '',
    project_id: '',
    company_id: '',
    contact_id: '',
    activity: 'Design and Consultancy',
    strap_line: 'sensory design studio',
    staff_email: '',
    staff_phone: '',
    staff_position: ''
  };
  
  // Status options
  const statusOptions = [
    { value: 'Draft', label: 'Draft' },
    { value: 'Sent', label: 'Sent' },
    { value: 'Negotiation', label: 'Negotiation' },
    { value: 'Awarded', label: 'Awarded' },
    { value: 'Completed', label: 'Completed' },
    { value: 'Lost', label: 'Lost' },
    { value: 'Cancelled', label: 'Cancelled' },
    { value: 'On Hold', label: 'On Hold' },
    { value: 'Revised', label: 'Revised' }
  ];
  
  // Validation setup
  const validationRules = [
    { field: 'number' as keyof ProposalFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'name' as keyof ProposalFormData, required: true, minLength: 1, maxLength: 255 },
    { field: 'issue_date' as keyof ProposalFormData, required: true, minLength: 6, maxLength: 6 },
    { field: 'project_id' as keyof ProposalFormData, required: true, minLength: 1 },
    { field: 'company_id' as keyof ProposalFormData, required: true, minLength: 1 }
  ];
  
  // Form validation state
  let formErrors: Record<string, string> = {};
  
  // UI state
  let showDeleteConfirm = false;
  let showProjectStatusSync = false;
  let showJsonExportAlert = false;
  let originalStatus = '';
  let pendingUpdateData: any = null;
  let formInitialized = false;
  let dataLoaded = false;
  
  // Failsafe: Store original proposal data when modal opens
  let originalProposal: Fee | null = null;
  
  // Auto-export checkbox state (activated by default for new proposals)
  let autoExportToJson = true;
  
  // Nested modal states
  let showNewProjectModal = false;
  let showCompanyModal = false;
  let showContactModal = false;
  let companyModalMode: 'create' | 'edit' = 'create';
  let contactModalMode: 'create' | 'edit' = 'create';
  let selectedCompany: Company | null = null;
  let selectedContact: Contact | null = null;
  
  // Typeahead search states
  let projectSearchText = '';
  let companySearchText = '';
  let contactSearchText = '';
  
  // Filtered options for typeahead dropdowns
  let projectOptions: typeof allProjectOptions = [];
  let companyOptions: typeof allCompanyOptions = [];
  let contactOptions: typeof allContactOptions = [];
  
  // Helper function to extract ID from various formats
  function extractId(value: any): string {
    return extractSurrealId(value) || '';
  }
  
  // All dropdown options for typeahead - sorted by update date (newest first)
  $: allProjectOptions = $projectsStore
    .map(project => ({
      id: extractId(project.id),
      name: project.name,
      name_short: project.name_short,
      number: project.number?.id || `${project.number?.year || ''}-${project.number?.country || ''}-${project.number?.seq || ''}`.replace(/^-+|-+$/g, '') || 'No Number',
      country: project.country,
      city: project.city,
      area: project.area,
      updated_at: project.time?.updated_at || ''
    }))
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime());
  
  // All company options - sorted by update date (newest first)
  $: allCompanyOptions = $companiesStore
    .map(company => ({
      id: extractId(company.id),
      name: company.name,
      name_short: company.name_short,
      abbreviation: company.abbreviation,
      updated_at: company.time?.updated_at || ''
    }))
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime());
  
  // All contact options - sorted by update date (newest first)
  $: allContactOptions = $contactsStore
    .map(contact => ({
      id: extractId(contact.id),
      full_name: contact.full_name,
      company: extractId(contact.company),
      updated_at: contact.time?.updated_at || ''
    }))
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime());

  // Filtered options based on selections
  $: filteredCompanyOptions = formData.contact_id 
    ? allCompanyOptions.filter(company => {
        const selectedContact = allContactOptions.find(c => c.id === formData.contact_id);
        return selectedContact ? company.id === selectedContact.company : true;
      })
    : allCompanyOptions;

  // Fix reactivity by explicitly depending on formData.company_id
  $: filteredContactOptions = formData.company_id 
    ? allContactOptions.filter(contact => contact.company === formData.company_id)
    : allContactOptions;
  
  // Project search handler with fuzzy search
  function handleProjectSearch(searchText: string) {
    if (!searchText || searchText.length < 1) {
      projectOptions = allProjectOptions.filter(project => 
        // Don't show projects that already have an RFP
        !$feesStore.some(fee => extractId(fee.project_id) === project.id)
      ).slice(0, 10);
      return;
    }
    
    const search = searchText.toLowerCase();
    projectOptions = allProjectOptions.filter(project => {
      // Don't show projects that already have an RFP
      if ($feesStore.some(fee => extractId(fee.project_id) === project.id)) {
        return false;
      }
      
      // Fuzzy search across multiple fields
      return (
        project.name?.toLowerCase().includes(search) ||
        project.name_short?.toLowerCase().includes(search) ||
        project.number?.toLowerCase().includes(search) ||
        project.country?.toLowerCase().includes(search) ||
        project.city?.toLowerCase().includes(search) ||
        project.area?.toLowerCase().includes(search)
      );
    }).slice(0, 20);
  }
  
  // Initialize project options
  $: if (!projectSearchText) {
    handleProjectSearch('');
  }
  
  // Filtered company options for search (use filtered options as base)
  $: companyOptions = filteredCompanyOptions.filter(company => {
    if (!companySearchText) return true;
    
    const searchLower = companySearchText.toLowerCase().trim();
    const nameMatch = company.name && company.name.toLowerCase().includes(searchLower);
    const shortNameMatch = company.name_short && company.name_short.toLowerCase().includes(searchLower);
    
    // Handle abbreviation search
    let abbreviationMatch = false;
    if (company.abbreviation) {
      const abbrev = String(company.abbreviation).toLowerCase().trim();
      abbreviationMatch = abbrev.includes(searchLower);
    }
    
    return nameMatch || shortNameMatch || abbreviationMatch;
  }).slice(0, 20);
  
  // Filtered contact options for search (use filtered options as base)
  $: contactOptions = filteredContactOptions.filter(contact =>
    !contactSearchText ||
    contact.full_name.toLowerCase().includes(contactSearchText.toLowerCase())
  ).slice(0, 20);
  
  
  // Helper to format today's date in YYMMDD format
  function getTodayFormatted(): string {
    const today = new Date();
    const year = today.getFullYear().toString().slice(-2);
    const month = (today.getMonth() + 1).toString().padStart(2, '0');
    const day = today.getDate().toString().padStart(2, '0');
    return `${year}${month}${day}`;
  }
  
  // Auto-generate proposal number
  function generateProposalNumber() {
    if (!formData.number && formData.project_id) {
      const project = $projectsStore.find(p => extractId(p.id) === formData.project_id);
      if (project?.number?.id) {
        formData.number = `${project.number.id}-FP-${formData.rev}`;
      }
    }
  }
  
  // Check if proposal status is compatible with project status
  function isCompatibleProjectStatus(proposalStatus: string): boolean {
    const proposalToProjectMapping: Record<string, string> = {
      'Draft': 'Draft',
      'Sent': 'RFP',
      'Negotiation': 'RFP',
      'Awarded': 'Awarded',
      'Completed': 'Completed',
      'Lost': 'Lost',
      'Cancelled': 'Cancelled',
      'On Hold': 'On Hold',
      'Revised': 'Revised'
    };
    return proposalStatus in proposalToProjectMapping;
  }
  
  // Get the mapped project status for a proposal status
  function getProjectStatusFromProposalStatus(proposalStatus: string): string {
    const proposalToProjectMapping: Record<string, string> = {
      'Draft': 'Draft',
      'Sent': 'RFP',
      'Negotiation': 'RFP',
      'Awarded': 'Awarded',
      'Completed': 'Completed',
      'Lost': 'Lost',
      'Cancelled': 'Cancelled',
      'On Hold': 'On Hold',
      'Revised': 'Revised'
    };
    return proposalToProjectMapping[proposalStatus] || proposalStatus;
  }
  
  // Form submission handler
  function handleSubmit(event: Event) {
    event.preventDefault();
    console.log('ProposalModal: handleSubmit called, mode:', mode);
    
    // If user typed in the project field but didn't select from dropdown, try to find a match
    if (projectSearchText && !formData.project_id) {
      const exactMatch = allProjectOptions.find(project => 
        project.name.toLowerCase() === projectSearchText.toLowerCase() ||
        project.number.toLowerCase() === projectSearchText.toLowerCase()
      );
      if (exactMatch) {
        formData.project_id = exactMatch.id;
      }
    }
    
    // Custom validation for date format
    const errors = validateForm(formData, validationRules);
    
    // Additional validation for issue date format (YYMMDD)
    if (formData.issue_date && !/^\d{6}$/.test(formData.issue_date)) {
      errors.issue_date = 'Issue date must be in YYMMDD format';
    }
    
    formErrors = errors;
    
    if (hasValidationErrors(errors)) {
      operationActions.setError('Please fix the validation errors above.');
      return;
    }
    
    if (mode === 'create') {
      console.log('ProposalModal: Calling handleCreate');
      handleCreate();
    } else {
      console.log('ProposalModal: Calling handleUpdate');
      handleUpdate();
    }
  }
  
  // Create proposal with loading state
  async function handleCreate() {
    await withLoadingState(async () => {
      // Send clean IDs - backend SQL now properly adds table prefixes
      const projectId = formData.project_id ? formData.project_id.replace('-', '_') : '';
      const companyId = formData.company_id || '';
      const contactId = formData.contact_id || '';
      
      const proposalData = {
        ...formData,
        rev: parseInt(formData.rev) || 1,
        project_id: projectId,
        company_id: companyId,
        contact_id: contactId,
        revisions: [],
        time: {
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
      };
      
      const result = await feesActions.create(proposalData);
      
      // Auto-export to JSON if enabled
      if (autoExportToJson && result?.id) {
        try {
          const feeId = extractSurrealId(result.id) || extractSurrealId(result) || '';
          if (feeId) {
            const exportResult = await writeFeeToJsonSafe(feeId);
            if (exportResult) {
              // Parse export result for user feedback
              const lines = exportResult.split('\n');
              const filePath = lines[0].replace('Successfully wrote fee proposal data to: ', '');
              
              let message = 'Proposal created successfully and exported to JSON!';
              
              // Add safety actions if present
              const safetyIndex = lines.findIndex(line => line.includes('Safety actions taken:'));
              if (safetyIndex !== -1) {
                const safetyActions = lines.slice(safetyIndex + 1).filter(line => line.trim().startsWith('•'));
                if (safetyActions.length > 0) {
                  message += '\n\nJSON Export Details:';
                  safetyActions.forEach(action => {
                    message += `\n${action.trim()}`;
                  });
                }
              }
              
              operationActions.setMessage(message);
            } else {
              operationActions.setMessage('Proposal created successfully, but JSON export failed');
            }
          } else {
            operationActions.setMessage('Proposal created successfully, but could not extract ID for JSON export');
          }
        } catch (error) {
          console.error('Auto-export failed:', error);
          operationActions.setMessage(`Proposal created successfully, but JSON export failed: ${error}`);
        }
      } else {
        operationActions.setMessage('Proposal created successfully');
      }
      
      resetForm();
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Update proposal with loading state  
  async function handleUpdate() {
    console.log('ProposalModal: handleUpdate called');
    console.log('ProposalModal: proposal data:', proposal);
    
    const activeProposal = proposal || originalProposal;
    if (!activeProposal) {
      console.error('ProposalModal: No proposal data available');
      operationActions.setError('No proposal data available for update');
      return;
    }
    
    // Try multiple extraction approaches like ContactModal
    const proposalId = extractSurrealId(activeProposal.id) || extractSurrealId(activeProposal) || activeProposal.id || '';
    console.log('ProposalModal: extracted proposalId:', proposalId);
    
    if (!proposalId) {
      operationActions.setError('Invalid proposal ID');
      return;
    }
    
    // Send clean IDs - backend SQL now properly adds table prefixes
    const projectId = formData.project_id ? formData.project_id.replace('-', '_') : '';
    const companyId = formData.company_id || '';
    const contactId = formData.contact_id || '';
    
    const updateData = {
      ...formData,
      rev: parseInt(formData.rev) || 1,
      project_id: projectId,
      company_id: companyId,
      contact_id: contactId,
      revisions: activeProposal?.revisions || []
    };
    
    // Check if status has changed and would result in different project status
    const originalProjectStatus = getProjectStatusFromProposalStatus(originalStatus);
    const newProjectStatus = getProjectStatusFromProposalStatus(formData.status);
    const projectStatusWouldChange = originalProjectStatus !== newProjectStatus;
    
    if (originalStatus !== formData.status && isCompatibleProjectStatus(formData.status) && projectStatusWouldChange) {
      // Store the update data and show confirmation dialog
      pendingUpdateData = updateData;
      showProjectStatusSync = true;
      return;
    }
    
    // If no project status sync needed, proceed with normal update
    await withLoadingState(async () => {
      const result = await feesActions.update(proposalId, updateData);
      
      // After successful update, show JSON export alert
      showJsonExportAlert = true;
      
      return result;
    }, operationActions, 'saving');
  }
  
  // Delete proposal with loading state
  async function handleDelete() {
    const activeProposal = proposal || originalProposal;
    if (!activeProposal || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      // Try multiple extraction approaches like ContactModal
      const proposalId = extractSurrealId(activeProposal.id) || extractSurrealId(activeProposal) || activeProposal.id || '';
      if (!proposalId) throw new Error('Invalid proposal ID');
      
      const result = await feesActions.delete(proposalId);
      operationActions.setMessage('Proposal deleted successfully');
      closeModal();
      return result;
    }, operationActions, 'deleting');
  }
  
  // Handle project status sync confirmation
  async function handleProjectStatusSync(syncStatus: boolean) {
    console.log('ProposalModal: handleProjectStatusSync called, syncStatus:', syncStatus);
    console.log('ProposalModal: pendingUpdateData:', pendingUpdateData);
    
    // Enhanced debugging before clearing dialog
    console.log('=== ENHANCED DEBUG: handleProjectStatusSync ===');
    console.log('ProposalModal: proposal prop value:', proposal);
    console.log('ProposalModal: proposal type:', typeof proposal);
    console.log('ProposalModal: proposal is null?', proposal === null);
    console.log('ProposalModal: proposal is undefined?', proposal === undefined);
    if (proposal) {
      console.log('ProposalModal: proposal.id value:', proposal.id);
      console.log('ProposalModal: proposal.id type:', typeof proposal.id);
      console.log('ProposalModal: proposal keys:', Object.keys(proposal));
    }
    console.log('ProposalModal: mode value:', mode);
    console.log('ProposalModal: isOpen value:', isOpen);
    console.log('=== END ENHANCED DEBUG ===');
    
    showProjectStatusSync = false;
    
    await withLoadingState(async () => {
      // Debug the proposal object structure
      console.log('ProposalModal: proposal object structure:', proposal);
      console.log('ProposalModal: proposal.id structure:', proposal?.id);
      console.log('ProposalModal: originalProposal fallback:', originalProposal);
      
      // Use failsafe: try current proposal first, then fall back to originalProposal
      const activeProposal = proposal || originalProposal;
      console.log('ProposalModal: activeProposal selected:', activeProposal);
      
      if (!activeProposal) {
        console.error('ProposalModal: No proposal data available (both proposal and originalProposal are null)');
        throw new Error('No proposal data available for update');
      }
      
      // Use the same ID extraction logic as handleUpdate
      const proposalId = extractSurrealId(activeProposal.id) || extractSurrealId(activeProposal) || activeProposal.id || '';
      console.log('ProposalModal: extracted proposalId in handleProjectStatusSync:', proposalId);
      
      // If proposalId is still empty, try alternative extraction methods
      if (!proposalId) {
        console.error('ProposalModal: Failed to extract proposal ID');
        console.error('ProposalModal: activeProposal:', activeProposal);
        console.error('ProposalModal: activeProposal.id:', activeProposal?.id);
        throw new Error('Invalid proposal ID');
      }
      
      let updateData = pendingUpdateData;
      if (!updateData) {
        console.warn('ProposalModal: pendingUpdateData was null, recreating from form data');
        // Recreate updateData from current form
        const projectId = formData.project_id ? formData.project_id.replace('-', '_') : '';
        const companyId = formData.company_id || '';
        const contactId = formData.contact_id || '';
        
        updateData = {
          ...formData,
          rev: parseInt(formData.rev) || 1,
          project_id: projectId,
          company_id: companyId,
          contact_id: contactId,
          revisions: activeProposal?.revisions || []
        };
      }
      
      // Update the proposal first
      await feesActions.update(proposalId, updateData);
      
      // If user confirmed, also update the project status
      if (syncStatus && formData.project_id) {
        const projectId = extractId(formData.project_id);
        if (projectId) {
          const projectStatus = getProjectStatusFromProposalStatus(formData.status);
          
          // Get the current project data from the store and update only the status
          const currentProject = $projectsStore.find(p => extractId(p.id) === projectId);
          if (currentProject) {
            const fullUpdateData = {
              name: currentProject.name,
              name_short: currentProject.name_short,
              status: projectStatus,
              area: currentProject.area,
              city: currentProject.city,
              country: currentProject.country,
              folder: currentProject.folder
            };
            
            await projectsActions.update(projectId, fullUpdateData);
          }
        }
      }
      
      operationActions.setMessage(syncStatus 
        ? 'Proposal and project status updated successfully!' 
        : 'Proposal updated successfully!');
      
      // After successful update, show JSON export alert
      showJsonExportAlert = true;
      return true;
    }, operationActions, 'saving');
    
    pendingUpdateData = null;
  }
  
  // Handle JSON export from alert
  async function handleJsonExportFromAlert() {
    const activeProposal = proposal || originalProposal;
    if (!activeProposal) return;
    
    showJsonExportAlert = false;
    
    try {
      const proposalId = extractSurrealId(activeProposal.id) || extractSurrealId(activeProposal) || activeProposal.id || '';
      if (!proposalId) {
        operationActions.setError('Could not extract proposal ID for JSON export');
        return;
      }
      
      const exportResult = await writeFeeToJsonSafe(proposalId);
      if (exportResult) {
        // Parse export result for user feedback
        const lines = exportResult.split('\n');
        const filePath = lines[0].replace('Successfully wrote fee proposal data to: ', '');
        
        let message = 'Proposal updated and exported to JSON successfully!';
        
        // Add safety actions if present
        const safetyIndex = lines.findIndex(line => line.includes('Safety actions taken:'));
        if (safetyIndex !== -1) {
          const safetyActions = lines.slice(safetyIndex + 1).filter(line => line.trim().startsWith('•'));
          if (safetyActions.length > 0) {
            message += '\n\nJSON Export Details:';
            safetyActions.forEach(action => {
              message += `\n${action.trim()}`;
            });
          }
        }
        
        operationActions.setMessage(message);
        closeModal();
      } else {
        operationActions.setError('JSON export failed - no result returned');
      }
    } catch (error) {
      console.error('JSON export failed:', error);
      operationActions.setError(`JSON export failed: ${error}`);
    }
  }
  
  // Handle dismissing the JSON export alert
  function handleJsonExportDismiss() {
    showJsonExportAlert = false;
    closeModal();
  }
  
  // Form management
  function resetForm() {
    const todayFormatted = getTodayFormatted();
    
    formData = {
      number: '',
      name: 'Fee Proposal',
      issue_date: todayFormatted,
      rev: '1',
      status: 'Draft',
      package: '',
      staff_name: $settingsStore.staff_name || '',
      project_id: '',
      company_id: '',
      contact_id: '',
      activity: 'Design and Consultancy',
      strap_line: 'sensory design studio',
      staff_email: $settingsStore.staff_email || '',
      staff_phone: $settingsStore.staff_phone || '',
      staff_position: $settingsStore.staff_position || ''
    };
    
    formErrors = {};
    showDeleteConfirm = false;
    showProjectStatusSync = false;
    showJsonExportAlert = false;
    originalStatus = '';
    pendingUpdateData = null;
    formInitialized = false;
    dataLoaded = false;
    
    // Reset auto-export checkbox to default (enabled)
    autoExportToJson = true;
    
    // Clear search texts
    projectSearchText = '';
    companySearchText = '';
    contactSearchText = '';
  }
  
  function closeModal() {
    resetForm();
    operationActions.reset();
    dispatch('close');
  }
  
  // Typeahead handlers
  function handleProjectSelect(event: CustomEvent) {
    formData.project_id = event.detail.id;
    projectSearchText = event.detail.option.name; // Keep search text in sync
    generateProposalNumber();
  }
  
  function handleProjectClear() {
    formData.project_id = '';
    projectSearchText = '';
  }
  
  function handleCompanySelect(event: CustomEvent) {
    formData.company_id = event.detail.id;
    companySearchText = event.detail.option.name;
    // Clear contact when company changes
    formData.contact_id = '';
    contactSearchText = '';
  }
  
  function handleContactSelect(event: CustomEvent) {
    formData.contact_id = event.detail.id;
    contactSearchText = event.detail.option.full_name;
    
    // Only auto-select company if form is initialized and not loading existing data
    if (formInitialized && mode === 'create') {
      const selectedContact = allContactOptions.find(c => c.id === event.detail.id);
      if (selectedContact && selectedContact.company) {
        const contactCompanyId = selectedContact.company;
        if (contactCompanyId && contactCompanyId !== formData.company_id) {
          formData.company_id = contactCompanyId;
          const company = allCompanyOptions.find(c => c.id === contactCompanyId);
          if (company) {
            companySearchText = company.name;
          }
        }
      }
    }
  }

  // Clear handlers for cross-field clearing
  function handleCompanyClear() {
    formData.company_id = '';
    companySearchText = '';
    // Also clear contact when company is cleared
    formData.contact_id = '';
    contactSearchText = '';
  }

  function handleContactClear() {
    formData.contact_id = '';
    contactSearchText = '';
    // Also clear company when contact is cleared
    formData.company_id = '';
    companySearchText = '';
  }
  
  // Nested modal handlers
  function handleNewProject() {
    showNewProjectModal = true;
  }
  
  function handleNewProjectClosed() {
    showNewProjectModal = false;
    // Refresh project list to include the newly created project
    projectsActions.load();
  }
  
  function handleNewCompany() {
    selectedCompany = null;
    companyModalMode = 'create';
    showCompanyModal = true;
  }
  
  function handleCompanyModalClosed() {
    showCompanyModal = false;
    selectedCompany = null;
  }
  
  function handleNewContact() {
    selectedContact = null;
    contactModalMode = 'create';
    showContactModal = true;
  }
  
  function handleContactModalClosed() {
    showContactModal = false;
    selectedContact = null;
  }
  
  // Keep track of store lengths to detect new entities
  let previousProjectCount = 0;
  let previousCompanyCount = 0;
  let previousContactCount = 0;
  
  // Handle successful creation from nested modals
  $: if ($projectsStore.length > previousProjectCount && !showNewProjectModal) {
    // A new project was created
    const latestProject = $projectsStore[$projectsStore.length - 1];
    if (latestProject) {
      const projectId = extractId(latestProject.id);
      formData.project_id = projectId;
      projectSearchText = `${latestProject.number?.id || ''} - ${latestProject.name}`;
      generateProposalNumber(); // Auto-generate proposal number based on new project
    }
    previousProjectCount = $projectsStore.length;
  }
  
  $: if ($companiesStore.length > previousCompanyCount && !showCompanyModal) {
    // A new company was created
    const latestCompany = $companiesStore[$companiesStore.length - 1];
    if (latestCompany) {
      const companyId = extractId(latestCompany.id);
      formData.company_id = companyId;
      companySearchText = latestCompany.name;
    }
    previousCompanyCount = $companiesStore.length;
  }
  
  $: if ($contactsStore.length > previousContactCount && !showContactModal) {
    // A new contact was created
    const latestContact = $contactsStore[$contactsStore.length - 1];
    if (latestContact) {
      const contactId = extractId(latestContact.id);
      formData.contact_id = contactId;
      contactSearchText = latestContact.full_name;
      
      // Auto-select the contact's company if we don't have one selected
      if (latestContact.company && !formData.company_id) {
        const contactCompanyId = extractSurrealId(latestContact.company) || '';
        if (contactCompanyId) {
          formData.company_id = contactCompanyId;
          const company = allCompanyOptions.find(c => c.id === contactCompanyId);
          if (company) {
            companySearchText = company.name;
          }
        }
      }
    }
    previousContactCount = $contactsStore.length;
  }
  
  // Initialize store counts
  $: if ($projectsStore.length > 0 && previousProjectCount === 0) {
    previousProjectCount = $projectsStore.length;
  }
  $: if ($companiesStore.length > 0 && previousCompanyCount === 0) {
    previousCompanyCount = $companiesStore.length;
  }
  $: if ($contactsStore.length > 0 && previousContactCount === 0) {
    previousContactCount = $contactsStore.length;
  }
  
  // Capture original proposal when modal opens (failsafe)
  $: if (proposal && isOpen && !originalProposal) {
    originalProposal = JSON.parse(JSON.stringify(proposal)); // Deep copy
    console.log('ProposalModal: Captured originalProposal:', originalProposal);
  }
  
  // Load form data when proposal changes - only when modal opens
  $: if (proposal && mode === 'edit' && isOpen && !dataLoaded) {
    loadProposalForEdit();
  }

  // Reset dataLoaded flag when modal closes
  $: if (!isOpen) {
    dataLoaded = false;
    originalStatus = '';
    originalProposal = null; // Clear failsafe data
  }

  function loadProposalForEdit() {
    if (!proposal || dataLoaded) return;
    dataLoaded = true;
    
    // Capture original status when modal first opens for editing
    originalStatus = proposal.status || 'Draft';
    
    formData = {
      number: proposal.number || '',
      name: proposal.name || '',
      issue_date: proposal.issue_date || '',
      rev: proposal.rev?.toString() || '0',
      status: proposal.status || 'Draft',
      package: proposal.package || '',
      staff_name: proposal.staff_name || '',
      project_id: extractId(proposal.project_id) || '',
      company_id: extractId(proposal.company_id) || '',
      contact_id: extractId(proposal.contact_id) || '',
      activity: proposal.activity || '',
      strap_line: proposal.strap_line || 'sensory design studio',
      staff_email: proposal.staff_email || '',
      staff_phone: proposal.staff_phone || '',
      staff_position: proposal.staff_position || ''
    };
    
    // Set search texts for selected items
    const selectedProject = allProjectOptions.find(p => p.id === formData.project_id);
    if (selectedProject) {
      projectSearchText = `${selectedProject.number} - ${selectedProject.name}`;
    }
    
    const selectedCompany = allCompanyOptions.find(c => c.id === formData.company_id);
    if (selectedCompany) {
      companySearchText = selectedCompany.name;
    }
    
    const selectedContact = allContactOptions.find(c => c.id === formData.contact_id);
    if (selectedContact) {
      contactSearchText = selectedContact.full_name;
    }
    
    // Clear any existing validation errors when loading edit data
    formErrors = {};
  }

  // Initialize form for create mode
  $: if (mode === 'create' && isOpen && !formInitialized) {
    resetForm();
  }

  // Set form as initialized after modal opens and data is loaded
  $: if (isOpen && !formInitialized) {
    // Small delay to ensure all reactive statements have run
    setTimeout(() => {
      formInitialized = true;
    }, 100);
  } else if (!isOpen) {
    formInitialized = false;
  }

  // Auto-populate staff fields from settings when settings change or form is reset
  $: if (mode === 'create' && $settingsStore.staff_name) {
    if (!formData.staff_name) formData.staff_name = $settingsStore.staff_name;
    if (!formData.staff_email) formData.staff_email = $settingsStore.staff_email || '';
    if (!formData.staff_phone) formData.staff_phone = $settingsStore.staff_phone || '';
    if (!formData.staff_position) formData.staff_position = $settingsStore.staff_position || '';
  }

  // Regenerate proposal number when revision changes
  $: if (formData.project_id && formData.rev && mode === 'create') {
    const project = $projectsStore.find(p => extractId(p.id) === formData.project_id);
    if (project?.number?.id) {
      formData.number = `${project.number.id}-FP-${formData.rev}`;
    }
  }
</script>

<BaseModal 
  {isOpen} 
  title={mode === 'create' ? 'Create New Fee Proposal' : 'Edit Fee Proposal'}
  maxWidth="500px"
  on:close={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
    
    <!-- PROJECT & CLIENT INFORMATION SECTION - MOVED TO TOP -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Project & Client Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Project Selection -->
        <div class="flex relative" style="gap: 8px;">
          <div class="flex-1">
            <TypeaheadSelect
              label="Project"
              value=""
              bind:searchText={projectSearchText}
              options={projectOptions}
              displayFields={['number', 'name']}
              placeholder="Search projects..."
              required
              error={formErrors.project_id}
              on:input={(e) => handleProjectSearch(e.detail)}
              on:select={handleProjectSelect}
              on:clear={handleProjectClear}
            >
              <svelte:fragment slot="option" let:option>
                <span class="font-medium">{option.number}</span> - <span class="truncate">{option.name}</span>
              </svelte:fragment>
            </TypeaheadSelect>
          </div>
          <button
            type="button"
            on:click={handleNewProject}
            class="w-8 h-8 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded flex items-center justify-center transition-all hover:scale-105 active:scale-95 mt-6"
            aria-label="Add new project"
            title="Add new project"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
        
        <!-- Company Selection -->
        <div class="flex relative" style="gap: 8px;">
          <div class="flex-1">
            <TypeaheadSelect
              label="Company"
              value=""
              bind:searchText={companySearchText}
              options={companyOptions}
              displayFields={['name']}
              placeholder="Search companies..."
              required
              error={formErrors.company_id}
              on:select={handleCompanySelect}
              on:clear={handleCompanyClear}
            />
          </div>
          <button
            type="button"
            on:click={handleNewCompany}
            class="w-8 h-8 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded flex items-center justify-center transition-all hover:scale-105 active:scale-95 mt-6"
            aria-label="Add new company"
            title="Add new company"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
        
        <!-- Contact Selection -->
        <div class="flex relative" style="gap: 8px;">
          <div class="flex-1">
            <TypeaheadSelect
              label="Contact"
              value=""
              bind:searchText={contactSearchText}
              options={contactOptions}
              displayFields={['full_name']}
              placeholder="Search contacts..."
              on:select={handleContactSelect}
              on:clear={handleContactClear}
            />
          </div>
          <button
            type="button"
            on:click={handleNewContact}
            class="w-8 h-8 bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded flex items-center justify-center transition-all hover:scale-105 active:scale-95 mt-6"
            aria-label="Add new contact"
            title="Add new contact"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
      </div>
    </div>
    
    <!-- BASIC INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Basic Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Number and Name -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Proposal Number"
            bind:value={formData.number}
            placeholder="25-97105-FP-1"
            required
            error={formErrors.number}
          />
          
          <FormInput
            label="Proposal Name"
            bind:value={formData.name}
            placeholder="Design Services"
            required
            error={formErrors.name}
          />
        </div>
        
        <!-- Issue Date and Revision -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Issue Date"
            bind:value={formData.issue_date}
            placeholder="YYMMDD format"
            maxlength="6"
            required
            error={formErrors.issue_date}
          />
          
          <FormInput
            label="Release"
            bind:value={formData.rev}
            placeholder="1"
            min="1"
          />
        </div>
        
        <!-- Status and Package -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormSelect
            label="Status"
            bind:value={formData.status}
            options={statusOptions}
          />
          
          <FormInput
            label="Package"
            bind:value={formData.package}
            placeholder="Package description"
          />
        </div>
        
        <!-- Activity and Strap Line -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Activity"
            bind:value={formData.activity}
            placeholder="Design and Consultancy"
          />
          
          <FormInput
            label="Strap Line"
            bind:value={formData.strap_line}
            placeholder="sensory design studio"
          />
        </div>
      </div>
    </div>
    
    <!-- STAFF INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Staff Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Staff Name and Email -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Staff Name"
            bind:value={formData.staff_name}
            placeholder="Staff member name"
          />
          
          <FormInput
            label="Staff Email"
            type="email"
            bind:value={formData.staff_email}
            placeholder="staff@emittiv.com"
          />
        </div>
        
        <!-- Staff Phone and Position -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Staff Phone"
            type="tel"
            bind:value={formData.staff_phone}
            placeholder="+971 50 123 4567"
          />
          
          <FormInput
            label="Staff Position"
            bind:value={formData.staff_position}
            placeholder="Lighting Director"
          />
        </div>
      </div>
    </div>
    
    <!-- Auto-Export Options (Create Mode Only) -->
    {#if mode === 'create'}
      <div>
        <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
          Export Options
        </h3>
        <div class="flex items-center" style="gap: 8px;">
          <input
            type="checkbox"
            id="auto-export-json"
            bind:checked={autoExportToJson}
            class="w-4 h-4 text-emittiv-splash bg-emittiv-dark border-emittiv-dark rounded focus:ring-emittiv-splash focus:ring-2"
          />
          <label for="auto-export-json" class="text-emittiv-lighter text-sm cursor-pointer">
            Automatically export to project JSON file after creation
          </label>
        </div>
        <p class="text-emittiv-light text-xs mt-2">
          When enabled, the proposal data will be safely exported to the project's JSON file with automatic backup of existing data.
        </p>
      </div>
    {/if}
    
    <!-- Error/Success Messages -->
    {#if $operationState.error}
      <div class="text-red-400 text-sm bg-red-900/20 border border-red-500/30 rounded p-3">
        {$operationState.error}
      </div>
    {/if}
    
    {#if $operationState.message}
      <div class="text-green-400 text-sm bg-green-900/20 border border-green-500/30 rounded p-3">
        {$operationState.message}
      </div>
    {/if}
    
    <!-- Delete Confirmation -->
    {#if showDeleteConfirm && mode === 'edit'}
      <div class="text-red-400 text-sm bg-red-900/20 border border-red-500/30 rounded" style="padding: 8px 12px;">
        <p class="font-medium" style="margin-bottom: 4px;">Are you sure you want to delete this proposal?</p>
        <p class="text-xs opacity-80">This action cannot be undone.</p>
      </div>
    {/if}
    
    <!-- Project Status Sync Confirmation -->
    {#if showProjectStatusSync}
      <div class="text-blue-400 text-sm bg-blue-900/20 border border-blue-500/30 rounded p-3">
        <p class="font-medium mb-2">Update Project Status</p>
        <p class="text-xs opacity-80 mb-3">
          The proposal status change would also update the project status to "{getProjectStatusFromProposalStatus(formData.status)}". 
          Would you like to sync the project status as well?
        </p>
        <div class="flex gap-2">
          <button
            type="button"
            on:click={() => handleProjectStatusSync(true)}
            class="bg-blue-600 hover:bg-blue-700 text-white rounded font-medium transition-all flex items-center justify-center disabled:opacity-50"
            style="height: 24px; padding: 4px 8px; font-size: 11px;"
            disabled={$operationState.saving}
          >
            Yes, sync both
          </button>
          <button
            type="button"
            on:click={() => handleProjectStatusSync(false)}
            class="border border-blue-500/30 rounded text-blue-300 hover:text-blue-200 transition-all"
            style="height: 24px; padding: 4px 8px; font-size: 11px;"
            disabled={$operationState.saving}
          >
            No, proposal only
          </button>
        </div>
      </div>
    {/if}
    
    <!-- JSON Export Alert -->
    {#if showJsonExportAlert}
      <div class="text-emittiv-splash text-sm bg-orange-900/20 border border-orange-500/30 rounded" style="padding: 8px 12px;">
        <p class="font-medium" style="margin-bottom: 4px;">Export to JSON?</p>
        <div class="flex gap-2">
          <button
            type="button"
            on:click={handleJsonExportFromAlert}
            class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded font-medium transition-all flex items-center justify-center disabled:opacity-50"
            style="height: 24px; padding: 4px 8px; font-size: 11px;"
            disabled={$operationState.saving}
          >
            Yes, export
          </button>
          <button
            type="button"
            on:click={handleJsonExportDismiss}
            class="border border-orange-500/30 rounded text-orange-300 hover:text-orange-200 transition-all"
            style="height: 24px; padding: 4px 8px; font-size: 11px;"
            disabled={$operationState.saving}
          >
            No, close
          </button>
        </div>
      </div>
    {/if}
    
    <!-- Actions - Full Width Container -->
    <div class="w-full" style="height: 40px;">
      {#if mode === 'edit' && !showDeleteConfirm}
        <!-- Edit Mode: Delete button on left, Cancel/Update on right -->
        <div class="flex justify-between items-stretch h-full" style="gap: 12px;">
          <Button
            variant="ghost"
            size="sm"
            className="!bg-red-600 !text-white hover:!bg-red-700 !border !border-red-500 h-full !py-1 !flex !items-center !justify-center"
            on:click={() => showDeleteConfirm = true}
            disabled={$operationState.saving || $operationState.deleting || showProjectStatusSync || showJsonExportAlert}
          >
            Delete
          </Button>
          
          <div class="flex h-full" style="gap: 12px;">
            <Button
              variant="secondary"
              size="sm"
              className="h-full !py-1 !flex !items-center !justify-center"
              on:click={closeModal}
              disabled={$operationState.saving || $operationState.deleting}
            >
              Cancel
            </Button>
            
            <Button
              type="submit"
              variant="primary"
              size="sm"
              className="h-full !py-1 !flex !items-center !justify-center"
              disabled={$operationState.saving || $operationState.deleting || showProjectStatusSync || showJsonExportAlert}
            >
              {#if $operationState.saving}
                <div 
                  class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
                  style="width: 14px; height: 14px; margin-right: 6px;"
                ></div>
              {/if}
              Update
            </Button>
          </div>
        </div>
      {:else if mode === 'edit' && showDeleteConfirm}
        <!-- Delete Confirmation Mode -->
        <div class="flex justify-between items-stretch h-full" style="gap: 12px;">
          <Button
            variant="ghost"
            size="sm"
            className="!bg-red-600 !text-white hover:!bg-red-700 !border !border-red-500 h-full !py-1 !flex !items-center !justify-center"
            on:click={handleDelete}
            disabled={$operationState.deleting}
          >
            {#if $operationState.deleting}
              <div 
                class="border-2 border-white border-t-transparent rounded-full animate-spin"
                style="width: 14px; height: 14px; margin-right: 6px;"
              ></div>
            {/if}
            Confirm Delete
          </Button>
          <Button
            variant="secondary"
            size="sm"
            className="h-full !py-1 !flex !items-center !justify-center"
            on:click={() => showDeleteConfirm = false}
            disabled={$operationState.deleting}
          >
            Cancel
          </Button>
        </div>
      {:else}
        <!-- Create Mode: Just Cancel/Create buttons -->
        <div class="flex justify-end items-stretch h-full" style="gap: 12px;">
          <Button
            variant="secondary"
            size="sm"
            className="h-full !py-1 !flex !items-center !justify-center"
            on:click={closeModal}
            disabled={$operationState.saving}
          >
            Cancel
          </Button>
          
          <Button
            type="submit"
            variant="primary"
            size="sm"
            className="h-full !py-1 !flex !items-center !justify-center"
            disabled={$operationState.saving || showProjectStatusSync || showJsonExportAlert}
          >
            {#if $operationState.saving}
              <div 
                class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
                style="width: 14px; height: 14px; margin-right: 6px;"
              ></div>
            {/if}
            Create Proposal
          </Button>
        </div>
      {/if}
    </div>
  </form>
</BaseModal>

<!-- Nested Modals with higher z-index -->
<!-- New Project Modal -->
<NewProjectModal 
  bind:isOpen={showNewProjectModal}
  on:close={handleNewProjectClosed}
/>

<!-- Company Modal -->
<CompanyModal 
  bind:isOpen={showCompanyModal}
  company={selectedCompany}
  mode={companyModalMode}
  on:close={handleCompanyModalClosed}
/>

<!-- Contact Modal -->
<ContactModal 
  bind:isOpen={showContactModal}
  contact={selectedContact}
  mode={contactModalMode}
  on:close={handleContactModalClosed}
/>

<style>
  /* Ensure nested modals appear above the proposal modal */
  :global(.base-modal) {
    z-index: 1000;
  }
  
  /* Higher z-index for nested modals */
  :global(.base-modal:last-child) {
    z-index: 1100;
  }
</style>