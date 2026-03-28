<!--
  Refactored Proposal Modal using BaseModal, FormInput, FormSelect, and TypeaheadSelect components
  Reduced from ~790 lines to ~480 lines using base components and utilities
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { ProjectStatus } from '../../types';
  import { feesStore, feesActions, projectsActions, projectsStore, companiesStore, contactsStore } from '$lib/stores';
  import { settingsStore } from '$lib/stores/settings';
  import { extractSurrealId, getEntityId } from '$lib/utils/surrealdb';
  import { validateForm, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import { writeFeeToJsonSafe } from '$lib/api';
  import { logger, logApiError } from '$lib/services/logger';
  import { PROPOSAL_STATUS_OPTIONS, type ProposalStatus } from '$lib/constants';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import TypeaheadSelect from './TypeaheadSelect.svelte';
  import Button from './Button.svelte';
  import NewProjectModal from './NewProjectModal.svelte';
  import CompanyModal from './CompanyModal.svelte';
  import ContactModal from './ContactModal.svelte';
  import type { Fee, Project, Company, Contact, UnknownSurrealThing } from '../../types';
  import type { PricingBreakdown } from '../../types/database';
  import FeePricingModal from './pricing/FeePricingModal.svelte';
  import CurrencyAmount from './CurrencyAmount.svelte';
  import ScopeViewer from './scope/ScopeViewer.svelte';
  
  let { isOpen = $bindable(false), proposal = null, mode = 'create', onclose }: {
    isOpen?: boolean;
    proposal?: Fee | null;
    mode?: 'create' | 'edit';
    onclose?: () => void;
  } = $props();
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Form data with better typing
  interface ProposalFormData {
    number: string;
    name: string;
    issue_date: string;
    rev: string;
    status: ProposalStatus;
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
  
  let formData: ProposalFormData = $state({
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
  });


  // Validation setup
  const validationRules = [
    { field: 'number' as keyof ProposalFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'name' as keyof ProposalFormData, required: true, minLength: 1, maxLength: 255 },
    { field: 'issue_date' as keyof ProposalFormData, required: true, minLength: 6, maxLength: 6 },
    { field: 'project_id' as keyof ProposalFormData, required: true, minLength: 1 },
    { field: 'company_id' as keyof ProposalFormData, required: true, minLength: 1 }
  ];

  // Form validation state
  let formErrors: Record<string, string> = $state({});

  // UI state
  let showDeleteConfirm = $state(false);
  let showProjectStatusSync = $state(false);
  let showJsonExportAlert = $state(false);
  let showDiscardConfirm = $state(false);
  let originalStatus = $state('');
  let pendingUpdateData: Partial<Fee> | null = $state(null);
  let formInitialized = $state(false);
  let dataLoaded = $state(false);

  // Unsaved changes guard
  let initialFormData: ProposalFormData | null = $state(null);
  const isDirty = $derived(
    initialFormData !== null &&
    JSON.stringify(formData) !== JSON.stringify(initialFormData)
  );

  // Failsafe: Store original proposal data when modal opens
  let originalProposal: Fee | null = $state(null);

  // Auto-export checkbox state (activated by default for new proposals)
  let autoExportToJson = $state(true);

  // Nested modal states
  let showNewProjectModal = $state(false);
  let showCompanyModal = $state(false);
  let showContactModal = $state(false);
  let showPricingModal = $state(false);
  let showScopeModal = $state(false);
  let scopeDirty = $state(false);
  let companyModalMode: 'create' | 'edit' = $state('create');
  let contactModalMode: 'create' | 'edit' = $state('create');
  let selectedCompany: Company | null = $state(null);
  let selectedContact: Contact | null = $state(null);

  // Typeahead search states
  let projectSearchText = $state('');
  let companySearchText = $state('');
  let contactSearchText = $state('');

  // Filtered options for typeahead dropdowns (projectOptions is mutable, others are $derived below)
  let projectOptions: Array<{ id: string; name: string; name_short: string | undefined; number: string; country: string; city: string; area: string | undefined; updated_at: string }> = $state([]);
  
  // Helper function to extract ID from various formats
  function extractId(value: UnknownSurrealThing): string {
    return extractSurrealId(value) || '';
  }
  
  // All dropdown options for typeahead - sorted by update date (newest first)
  const allProjectOptions = $derived($projectsStore
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
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()));

  // All company options - sorted by update date (newest first)
  const allCompanyOptions = $derived($companiesStore
    .map(company => ({
      id: extractId(company.id),
      name: company.name,
      name_short: company.name_short,
      abbreviation: company.abbreviation,
      updated_at: company.time?.updated_at || ''
    }))
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()));

  // All contact options - sorted by update date (newest first)
  const allContactOptions = $derived($contactsStore
    .map(contact => ({
      id: extractId(contact.id),
      full_name: contact.full_name,
      company: extractId(contact.company),
      updated_at: contact.time?.updated_at || ''
    }))
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()));

  // PERF-C2: O(1) lookup maps instead of O(n) .find() calls
  const projectOptionsMap = $derived(new Map(allProjectOptions.map(p => [p.id, p])));
  const companyOptionsMap = $derived(new Map(allCompanyOptions.map(c => [c.id, c])));
  const contactOptionsMap = $derived(new Map(allContactOptions.map(c => [c.id, c])));
  const projectStoreMap = $derived(new Map($projectsStore.map(p => [extractId(p.id), p])));
  // Set of project IDs that already have fee proposals
  const projectsWithFees = $derived(new Set($feesStore.map(fee => extractId(fee.project_id))));

  // Filtered options based on selections
  const filteredCompanyOptions = $derived(formData.contact_id
    ? allCompanyOptions.filter(company => {
        const selectedContact = contactOptionsMap.get(formData.contact_id);
        return selectedContact ? company.id === selectedContact.company : true;
      })
    : allCompanyOptions);

  // Fix reactivity by explicitly depending on formData.company_id
  const filteredContactOptions = $derived(formData.company_id
    ? allContactOptions.filter(contact => contact.company === formData.company_id)
    : allContactOptions);
  
  // Project search handler with fuzzy search
  function handleProjectSearch(searchText: string) {
    if (!searchText || searchText.length < 1) {
      projectOptions = allProjectOptions.filter(project =>
        // Don't show projects that already have an RFP
        !projectsWithFees.has(project.id)
      ).slice(0, 10);
      return;
    }

    const search = searchText.toLowerCase();
    projectOptions = allProjectOptions.filter(project => {
      // Don't show projects that already have an RFP
      if (projectsWithFees.has(project.id)) {
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
  $effect(() => {
    if (!projectSearchText) {
      handleProjectSearch('');
    }
  });
  
  // Filtered company options for search (use filtered options as base)
  const companyOptions = $derived(filteredCompanyOptions.filter(company => {
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
  }).slice(0, 20));

  // Filtered contact options for search (use filtered options as base)
  const contactOptions = $derived(filteredContactOptions.filter(contact =>
    !contactSearchText ||
    (contact.full_name || '').toLowerCase().includes(contactSearchText.toLowerCase())
  ).slice(0, 20));
  
  
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
      const project = projectStoreMap.get(formData.project_id);
      if (project?.number?.id) {
        formData.number = `${project.number.id}-FP-${formData.rev}`;
      }
    }
  }
  
  // Check if proposal status is compatible with project status
  function isCompatibleProjectStatus(proposalStatus: string): boolean {
    const proposalToProjectMapping: Record<string, string> = {
      'Draft': 'Lead',
      'Sent': 'RFP',
      'Negotiation': 'Submitted',
      'Accepted': 'Awarded',
      'Rejected': 'Lost',
      'No Response': 'No Response',
      'Superseded': 'Superseded'
    };
    return proposalStatus in proposalToProjectMapping;
  }

  // Get the mapped project status for a proposal status
  function getProjectStatusFromProposalStatus(proposalStatus: string): ProjectStatus {
    const proposalToProjectMapping: Record<string, ProjectStatus> = {
      'Draft': 'Lead',
      'Sent': 'RFP',
      'Negotiation': 'Submitted',
      'Accepted': 'Awarded',
      'Rejected': 'Lost',
      'No Response': 'No Response',
      'Superseded': 'Superseded'
    };
    return proposalToProjectMapping[proposalStatus] || (proposalStatus as ProjectStatus);
  }
  
  // Form submission handler
  function handleSubmit(event: Event) {
    event.preventDefault();

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
      handleCreate();
    } else {
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

      // QUAL-L6: Create timestamp once for consistency
      const timestamp = new Date().toISOString();

      const proposalData = {
        ...formData,
        rev: parseInt(formData.rev) || 1,
        project_id: projectId,
        company_id: companyId,
        contact_id: contactId,
        revisions: [],
        time: {
          created_at: timestamp,
          updated_at: timestamp
        }
      };
      
      const result = await feesActions.create(proposalData);
      
      // Auto-export to JSON if enabled
      if (autoExportToJson && result?.id) {
        try {
          const feeId = getEntityId(result);
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
          logApiError('auto-export proposal', error as Error);
          operationActions.setMessage(`Proposal created successfully, but JSON export failed: ${error}`);
        }
      } else {
        operationActions.setMessage('Proposal created successfully');
      }
      
      resetForm();
      doClose();
      return result;
    }, operationActions, 'saving');
  }

  // Update proposal with loading state
  async function handleUpdate() {
    const activeProposal = proposal || originalProposal;
    if (!activeProposal) {
      logger.error('ProposalModal: No proposal data available');
      operationActions.setError('No proposal data available for update');
      return;
    }

    const proposalId = getEntityId(activeProposal);
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
      
      operationActions.setMessage('Proposal updated successfully');
      doClose();

      return result;
    }, operationActions, 'saving');
  }

  // Delete proposal with loading state
  async function handleDelete() {
    const activeProposal = proposal || originalProposal;
    if (!activeProposal || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      const proposalId = getEntityId(activeProposal);
      if (!proposalId) throw new Error('Invalid proposal ID');

      const result = await feesActions.delete(proposalId);
      operationActions.setMessage('Proposal deleted successfully');
      doClose();
      return result;
    }, operationActions, 'deleting');
  }
  
  // Handle project status sync confirmation
  async function handleProjectStatusSync(syncStatus: boolean) {
    showProjectStatusSync = false;

    await withLoadingState(async () => {
      // Use failsafe: try current proposal first, then fall back to originalProposal
      const activeProposal = proposal || originalProposal;

      if (!activeProposal) {
        logger.error('ProposalModal: No proposal data available');
        throw new Error('No proposal data available for update');
      }

      const proposalId = getEntityId(activeProposal);
      if (!proposalId) {
        logger.error('ProposalModal: Failed to extract proposal ID');
        throw new Error('Invalid proposal ID');
      }

      let updateData = pendingUpdateData;
      if (!updateData) {
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
          const currentProject = projectStoreMap.get(projectId);
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

      doClose();
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
      const proposalId = getEntityId(activeProposal);
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
        doClose();
      } else {
        operationActions.setError('JSON export failed - no result returned');
      }
    } catch (error) {
      logApiError('JSON export', error as Error);
      operationActions.setError(`JSON export failed: ${error}`);
    }
  }
  
  // Handle dismissing the JSON export alert
  function handleJsonExportDismiss() {
    showJsonExportAlert = false;
    doClose();
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
    showDiscardConfirm = false;
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
    if (isDirty) {
      showDiscardConfirm = true;
      return;
    }
    doClose();
  }

  function doClose() {
    showDiscardConfirm = false;
    resetForm();
    operationActions.reset();
    onclose?.();
  }
  
  // Typeahead handlers
  function handleProjectSelect(data: { id: string; option: { id: string; [key: string]: unknown } }) {
    formData.project_id = data.id;
    projectSearchText = data.option.name as string; // Keep search text in sync
    generateProposalNumber();
  }
  
  function handleProjectClear() {
    formData.project_id = '';
    projectSearchText = '';
  }
  
  function handleCompanySelect(data: { id: string; option: { id: string; [key: string]: unknown } }) {
    formData.company_id = data.id;
    companySearchText = data.option.name as string;
    // Clear contact when company changes
    formData.contact_id = '';
    contactSearchText = '';
  }

  function handleContactSelect(data: { id: string; option: { id: string; [key: string]: unknown } }) {
    formData.contact_id = data.id;
    contactSearchText = data.option.full_name as string;

    // Only auto-select company if form is initialized and not loading existing data
    if (formInitialized && mode === 'create') {
      const selectedContact = contactOptionsMap.get(data.id);
      if (selectedContact && selectedContact.company) {
        const contactCompanyId = selectedContact.company;
        if (contactCompanyId && contactCompanyId !== formData.company_id) {
          formData.company_id = contactCompanyId;
          const company = companyOptionsMap.get(contactCompanyId);
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
  let previousProjectCount = $state(0);
  let previousCompanyCount = $state(0);
  let previousContactCount = $state(0);
  
  // Handle successful creation from nested modals
  $effect(() => {
    if ($projectsStore.length > previousProjectCount && !showNewProjectModal) {
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
  });

  $effect(() => {
    if ($companiesStore.length > previousCompanyCount && !showCompanyModal) {
      // A new company was created
      const latestCompany = $companiesStore[$companiesStore.length - 1];
      if (latestCompany) {
        const companyId = extractId(latestCompany.id);
        formData.company_id = companyId;
        companySearchText = latestCompany.name;
      }
      previousCompanyCount = $companiesStore.length;
    }
  });

  $effect(() => {
    if ($contactsStore.length > previousContactCount && !showContactModal) {
      // A new contact was created
      const latestContact = $contactsStore[$contactsStore.length - 1];
      if (latestContact) {
        const contactId = extractId(latestContact.id);
        formData.contact_id = contactId;
        contactSearchText = latestContact.full_name || '';

        // Auto-select the contact's company if we don't have one selected
        if (latestContact.company && !formData.company_id) {
          const contactCompanyId = extractSurrealId(latestContact.company) || '';
          if (contactCompanyId) {
            formData.company_id = contactCompanyId;
            const company = companyOptionsMap.get(contactCompanyId);
            if (company) {
              companySearchText = company.name;
            }
          }
        }
      }
      previousContactCount = $contactsStore.length;
    }
  });

  // Initialize store counts
  $effect(() => {
    if ($projectsStore.length > 0 && previousProjectCount === 0) {
      previousProjectCount = $projectsStore.length;
    }
  });
  $effect(() => {
    if ($companiesStore.length > 0 && previousCompanyCount === 0) {
      previousCompanyCount = $companiesStore.length;
    }
  });
  $effect(() => {
    if ($contactsStore.length > 0 && previousContactCount === 0) {
      previousContactCount = $contactsStore.length;
    }
  });
  
  // Capture original proposal when modal opens (failsafe)
  $effect(() => {
    if (proposal && isOpen && !originalProposal) {
      originalProposal = JSON.parse(JSON.stringify(proposal)); // Deep copy
    }
  });

  // Load form data when proposal changes - only when modal opens
  $effect(() => {
    if (proposal && mode === 'edit' && isOpen && !dataLoaded) {
      loadProposalForEdit();
    }
  });

  // Reset dataLoaded flag when modal closes
  $effect(() => {
    if (!isOpen) {
      dataLoaded = false;
      originalStatus = '';
      originalProposal = null; // Clear failsafe data
      initialFormData = null;
    }
  });

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
    const selectedProject = projectOptionsMap.get(formData.project_id);
    if (selectedProject) {
      projectSearchText = `${selectedProject.number} - ${selectedProject.name}`;
    }

    const selectedCompany = companyOptionsMap.get(formData.company_id);
    if (selectedCompany) {
      companySearchText = selectedCompany.name;
    }

    const selectedContact = contactOptionsMap.get(formData.contact_id);
    if (selectedContact) {
      contactSearchText = selectedContact.full_name || '';
    }

    // Clear any existing validation errors when loading edit data
    formErrors = {};

    // Capture initial snapshot for dirty-check
    initialFormData = JSON.parse(JSON.stringify(formData));
  }

  // Initialize form for create mode
  $effect(() => {
    if (mode === 'create' && isOpen && !formInitialized) {
      resetForm();
      // Capture initial snapshot after reset (next microtask so all effects have run)
      setTimeout(() => {
        initialFormData = JSON.parse(JSON.stringify(formData));
      }, 0);
    }
  });

  // Set form as initialized after modal opens and data is loaded
  $effect(() => {
    if (isOpen && !formInitialized) {
      // Small delay to ensure all reactive statements have run
      setTimeout(() => {
        formInitialized = true;
      }, 100);
    } else if (!isOpen) {
      formInitialized = false;
    }
  });

  // Auto-populate staff fields from settings when settings change or form is reset
  $effect(() => {
    if (mode === 'create' && $settingsStore.staff_name) {
      if (!formData.staff_name) formData.staff_name = $settingsStore.staff_name;
      if (!formData.staff_email) formData.staff_email = $settingsStore.staff_email || '';
      if (!formData.staff_phone) formData.staff_phone = $settingsStore.staff_phone || '';
      if (!formData.staff_position) formData.staff_position = $settingsStore.staff_position || '';
    }
  });

  // Regenerate proposal number when revision changes
  $effect(() => {
    if (formData.project_id && formData.rev && mode === 'create') {
      const project = projectStoreMap.get(formData.project_id);
      if (project?.number?.id) {
        formData.number = `${project.number.id}-FP-${formData.rev}`;
      }
    }
  });
</script>

<BaseModal 
  {isOpen} 
  title={mode === 'create' ? 'Create New Fee Proposal' : 'Edit Fee Proposal'}
  size="lg"
  onclose={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} class="emittiv-form-section emittiv-form-section--wide">

    <!-- PROJECT & CLIENT INFORMATION SECTION - MOVED TO TOP -->
    <div class="emittiv-form-section">
      <h3 class="emittiv-form-section__title">
        Project & Client Information
      </h3>
      <div class="emittiv-form-section">
        
        <!-- Project Selection -->
        <div class="emittiv-form-row">
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
              oninput={handleProjectSearch}
              onselect={handleProjectSelect}
              onclear={handleProjectClear}
            >
              <svelte:fragment slot="option" let:option>
                <span class="font-medium">{option.number}</span> - <span class="truncate">{option.name}</span>
              </svelte:fragment>
            </TypeaheadSelect>
          </div>
          <button
            type="button"
            on:click={handleNewProject}
            class="emittiv-fab-sm mt-6"
            aria-label="Add new project"
            title="Add new project"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
        
        <!-- Company Selection -->
        <div class="emittiv-form-row">
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
              onselect={handleCompanySelect}
              onclear={handleCompanyClear}
            />
          </div>
          <button
            type="button"
            on:click={handleNewCompany}
            class="emittiv-fab-sm mt-6"
            aria-label="Add new company"
            title="Add new company"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>
        
        <!-- Contact Selection -->
        <div class="emittiv-form-row">
          <div class="flex-1">
            <TypeaheadSelect
              label="Contact"
              value=""
              bind:searchText={contactSearchText}
              options={contactOptions}
              displayFields={['full_name']}
              placeholder="Search contacts..."
              onselect={handleContactSelect}
              onclear={handleContactClear}
            />
          </div>
          <button
            type="button"
            on:click={handleNewContact}
            class="emittiv-fab-sm mt-6"
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
    <div class="emittiv-form-section">
      <h3 class="emittiv-form-section__title">
        Basic Information
      </h3>
      <div class="emittiv-form-section">
        
        <!-- Number and Name -->
        <div class="emittiv-form-grid">
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
        <div class="emittiv-form-grid">
          <FormInput
            label="Issue Date"
            bind:value={formData.issue_date}
            placeholder="YYMMDD format"
            maxlength={6}
            inputmode="numeric"
            required
            error={formErrors.issue_date}
            onblur={() => {
              if (formData.issue_date && !/^\d{6}$/.test(formData.issue_date)) {
                formErrors = { ...formErrors, issue_date: 'Must be 6 digits (YYYYMM)' };
              } else {
                const { issue_date: _, ...rest } = formErrors;
                formErrors = rest;
              }
            }}
          />

          <FormInput
            label="Release"
            bind:value={formData.rev}
            placeholder="1"
            min={1}
          />
        </div>
        
        <!-- Status and Package -->
        <div class="emittiv-form-grid">
          <FormSelect
            label="Status"
            bind:value={formData.status}
            options={PROPOSAL_STATUS_OPTIONS}
          />
          
          <FormInput
            label="Package"
            bind:value={formData.package}
            placeholder="Package description"
          />
        </div>
        
        <!-- Activity and Strap Line -->
        <div class="emittiv-form-grid">
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
    <div class="emittiv-form-section">
      <h3 class="emittiv-form-section__title">
        Staff Information
      </h3>
      <div class="emittiv-form-section">
        
        <!-- Staff Name and Email -->
        <div class="emittiv-form-grid">
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
        <div class="emittiv-form-grid">
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
    
    <!-- Pricing Section (Edit Mode Only) -->
    {#if mode === 'edit'}
      <div class="emittiv-form-section">
        <h3 class="emittiv-form-section__title">
          Fee Pricing
        </h3>
        <div class="flex items-center justify-between">
          <p class="text-emittiv-light text-sm">
            {#if proposal?.pricing}
              Pricing configured: <CurrencyAmount amount={proposal.pricing.grand_total || 0} config={proposal.pricing.config} />
            {:else}
              No pricing configured yet
            {/if}
          </p>
          <Button
            variant="secondary"
            size="sm"
            on:click={() => showPricingModal = true}
          >
            {proposal?.pricing ? 'Edit Pricing' : 'Configure Pricing'}
          </Button>
        </div>
      </div>
    {/if}

    <!-- Scope Section (Edit Mode Only) -->
    {#if mode === 'edit'}
      <div class="emittiv-form-section">
        <h3 class="emittiv-form-section__title">
          Proposal Scope
        </h3>
        <div class="flex items-center justify-between">
          <p class="text-emittiv-light text-sm">
            Generate and edit scope text for this proposal
          </p>
          <Button
            variant="secondary"
            size="sm"
            on:click={() => showScopeModal = true}
          >
            Generate Scope
          </Button>
        </div>
      </div>
    {/if}

    <!-- Auto-Export Options (Create Mode Only) -->
    {#if mode === 'create'}
      <div class="emittiv-form-section">
        <h3 class="emittiv-form-section__title">
          Export Options
        </h3>
        <div class="emittiv-form-row items-center">
          <input
            type="checkbox"
            id="auto-export-json"
            bind:checked={autoExportToJson}
            class="emittiv-checkbox"
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
      <div class="emittiv-alert emittiv-alert--error">
        {$operationState.error}
      </div>
    {/if}
    
    {#if $operationState.message}
      <div class="emittiv-alert emittiv-alert--success">
        {$operationState.message}
      </div>
    {/if}
    
    <!-- Delete Confirmation -->
    {#if showDeleteConfirm && mode === 'edit'}
      <div class="emittiv-alert emittiv-alert--sm emittiv-alert--error">
        <p class="emittiv-alert__title">Are you sure you want to delete this proposal?</p>
        <p class="text-xs opacity-80">This action cannot be undone.</p>
      </div>
    {/if}
    
    <!-- Project Status Sync Confirmation -->
    {#if showProjectStatusSync}
      <div class="emittiv-alert emittiv-alert--info">
        <p class="font-medium mb-2">Also update the project status?</p>
        <p class="text-xs opacity-80 mb-3">
          Changing the proposal to "{formData.status}" would set the project to "{getProjectStatusFromProposalStatus(formData.status)}". Update both, or the proposal only?
        </p>
        <div class="flex gap-2">
          <button
            type="button"
            on:click={() => handleProjectStatusSync(true)}
            class="emittiv-confirm-btn emittiv-confirm-btn--blue"
            disabled={$operationState.saving}
          >
            Update both
          </button>
          <button
            type="button"
            on:click={() => handleProjectStatusSync(false)}
            class="emittiv-confirm-btn emittiv-confirm-btn--outline emittiv-confirm-btn--outline-blue"
            disabled={$operationState.saving}
          >
            Proposal only
          </button>
        </div>
      </div>
    {/if}
    
    <!-- JSON Export Alert -->
    {#if showJsonExportAlert}
      <div class="emittiv-alert emittiv-alert--sm emittiv-alert--warning">
        <p class="emittiv-alert__title">Export to JSON?</p>
        <div class="flex gap-2">
          <button
            type="button"
            on:click={handleJsonExportFromAlert}
            class="emittiv-confirm-btn emittiv-confirm-btn--primary"
            disabled={$operationState.saving}
          >
            Yes, export
          </button>
          <button
            type="button"
            on:click={handleJsonExportDismiss}
            class="emittiv-confirm-btn emittiv-confirm-btn--outline emittiv-confirm-btn--outline-orange"
            disabled={$operationState.saving}
          >
            No, close
          </button>
        </div>
      </div>
    {/if}
    
    <!-- Discard Unsaved Changes Confirmation -->
    {#if showDiscardConfirm}
      <div class="emittiv-alert emittiv-alert--warning">
        <p class="emittiv-alert__title">Discard unsaved changes?</p>
        <p class="emittiv-alert__subtitle">Your changes have not been saved.</p>
        <div class="flex gap-2">
          <button
            type="button"
            on:click={doClose}
            class="emittiv-confirm-btn emittiv-confirm-btn--primary"
          >
            Discard
          </button>
          <button
            type="button"
            on:click={() => showDiscardConfirm = false}
            class="emittiv-confirm-btn emittiv-confirm-btn--outline emittiv-confirm-btn--outline-orange"
          >
            Keep Editing
          </button>
        </div>
      </div>
    {/if}

    <!-- Actions - Full Width Container -->
    <div class="emittiv-modal__actions">
      {#if mode === 'edit' && !showDeleteConfirm}
        <!-- Edit Mode: Delete button on left, Cancel/Update on right -->
        <div class="emittiv-modal__actions-group emittiv-modal__actions-group--between">
          <Button
            variant="danger"
            size="sm"
            className="h-full"
            on:click={() => showDeleteConfirm = true}
            disabled={$operationState.saving || $operationState.deleting || showProjectStatusSync || showJsonExportAlert}
          >
            Delete
          </Button>
          
          <div class="emittiv-modal__actions-group">
            <Button
              variant="secondary"
              size="sm"
              className=""
              on:click={closeModal}
              disabled={$operationState.saving || $operationState.deleting}
            >
              Cancel
            </Button>
            
            <Button
              type="submit"
              variant="primary"
              size="sm"
              className=""
              disabled={$operationState.saving || $operationState.deleting || showProjectStatusSync || showJsonExportAlert}
            >
              {#if $operationState.saving}
                <div 
                  class="emittiv-spinner-sm"
                ></div>
              {/if}
              Update
            </Button>
          </div>
        </div>
      {:else if mode === 'edit' && showDeleteConfirm}
        <!-- Delete Confirmation Mode -->
        <div class="emittiv-modal__actions-group emittiv-modal__actions-group--between">
          <Button
            variant="danger"
            size="sm"
            className="h-full"
            on:click={handleDelete}
            disabled={$operationState.deleting}
          >
            {#if $operationState.deleting}
              <div class="emittiv-spinner-sm emittiv-spinner-sm--light"></div>
            {/if}
            Confirm Delete
          </Button>
          <Button
            variant="secondary"
            size="sm"
            className=""
            on:click={() => showDeleteConfirm = false}
            disabled={$operationState.deleting}
          >
            Cancel
          </Button>
        </div>
      {:else}
        <!-- Create Mode: Just Cancel/Create buttons -->
        <div class="emittiv-modal__actions-group">
          <Button
            variant="secondary"
            size="sm"
            className=""
            on:click={closeModal}
            disabled={$operationState.saving}
          >
            Cancel
          </Button>
          
          <Button
            type="submit"
            variant="primary"
            size="sm"
            className=""
            disabled={$operationState.saving || showProjectStatusSync || showJsonExportAlert}
          >
            {#if $operationState.saving}
              <div class="emittiv-spinner-sm"></div>
            {/if}
            Create Proposal
          </Button>
        </div>
      {/if}
    </div>
  </form>
</BaseModal>

<!-- Nested Modals with higher z-index (200 to appear above ProposalModal at 100) -->
<!-- New Project Modal -->
<NewProjectModal
  bind:isOpen={showNewProjectModal}
  zIndex={200}
  onclose={handleNewProjectClosed}
/>

<!-- Company Modal -->
<CompanyModal
  bind:isOpen={showCompanyModal}
  company={selectedCompany}
  mode={companyModalMode}
  zIndex={200}
  onclose={handleCompanyModalClosed}
/>

<!-- Contact Modal -->
<ContactModal
  bind:isOpen={showContactModal}
  contact={selectedContact}
  mode={contactModalMode}
  zIndex={200}
  onclose={handleContactModalClosed}
/>

<FeePricingModal
  bind:isOpen={showPricingModal}
  fee={proposal}
  onclose={() => showPricingModal = false}
  onsave={async () => {
    await feesActions.load();
    // Refresh local proposal from store so pricing data persists
    if (proposal?.id) {
      const feeId = extractId(proposal.id);
      const updated = $feesStore.find(f => extractId(f.id) === feeId);
      if (updated) proposal = updated;
    }
    showPricingModal = false;
  }}
/>

<!-- Scope Viewer Modal -->
{#if showScopeModal}
  <BaseModal
    isOpen={showScopeModal}
    title="Proposal Scope"
    size="xl"
    zIndex={200}
    onclose={() => {
      if (scopeDirty && !confirm('Discard unsaved scope changes?')) return;
      showScopeModal = false;
      scopeDirty = false;
    }}
  >
    {@const scopeProject = projectStoreMap.get(extractId(proposal?.project_id))}
    <ScopeViewer
      feeId={getEntityId(proposal)}
      stages={proposal?.pricing?.stages ?? []}
      projectName={scopeProject?.name || ''}
      projectNumber={scopeProject?.project_number || ''}
      ondirtychange={(d) => scopeDirty = d}
    />
  </BaseModal>
{/if}

<style>
  /* z-index for nested modals is now handled via props (zIndex={200}) */
</style>