<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { projectsStore, projectsActions, companiesStore, companiesActions, contactsStore, contactsActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId } from '$lib/utils';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import { copyProjectTemplate, writeFeeToJson, checkProjectFolderExists, checkVarJsonExists, checkVarJsonTemplateExists, renameVarJsonWithOldSuffix } from '$lib/api';
  import DetailPanel from './DetailPanel.svelte';
  import DetailHeader from './DetailHeader.svelte';
  import InfoCard from './InfoCard.svelte';
  import WarningModal from './WarningModal.svelte';
  import type { Fee, Project, Company, Contact } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let proposal: Fee | null = null;
  
  // Modal state
  let warningModal = {
    isOpen: false,
    title: 'Warning',
    message: '',
    confirmText: 'OK',
    cancelText: '',
    onConfirm: null,
    onCancel: null
  };
  
  // Create optimized company lookup
  $: companyLookup = createCompanyLookup($companiesStore);
  
  // Find related project
  $: relatedProject = proposal ? $projectsStore.find(p => {
    if (!proposal.project_id || !p?.id) return false;
    
    let projectIdStr = '';
    if (typeof p.id === 'string') {
      projectIdStr = p.id;
    } else if (p.id && typeof p.id === 'object') {
      if ((p.id as any).tb && (p.id as any).id) {
        if (typeof (p.id as any).id === 'string') {
          projectIdStr = `${(p.id as any).tb}:${(p.id as any).id}`;
        } else if ((p.id as any).id.String) {
          projectIdStr = `${(p.id as any).tb}:${(p.id as any).id.String}`;
        }
      }
    }
    
    let feeProjectId = '';
    if (typeof proposal.project_id === 'string') {
      feeProjectId = proposal.project_id;
    } else if (proposal.project_id && typeof proposal.project_id === 'object') {
      if ((proposal.project_id as any).tb && (proposal.project_id as any).id) {
        if (typeof (proposal.project_id as any).id === 'string') {
          feeProjectId = `${(proposal.project_id as any).tb}:${(proposal.project_id as any).id}`;
        } else if ((proposal.project_id as any).id.String) {
          feeProjectId = `${(proposal.project_id as any).tb}:${(proposal.project_id as any).id.String}`;
        }
      }
    }
    
    const id1 = feeProjectId.replace('projects:', '');
    const id2 = projectIdStr.replace('projects:', '');
    return id1 === id2 || feeProjectId === projectIdStr;
  }) : null;
  
  // Find related company
  $: relatedCompany = proposal ? companyLookup.getCompany(proposal.company_id) : null;
  
  // Find related contact
  $: relatedContact = proposal ? $contactsStore.find(c => {
    if (!proposal.contact_id || !c?.id) return false;
    
    let contactIdStr = '';
    if (typeof c.id === 'string') {
      contactIdStr = c.id;
    } else if (c.id && typeof c.id === 'object') {
      if ((c.id as any).tb && (c.id as any).id) {
        if (typeof (c.id as any).id === 'string') {
          contactIdStr = `${(c.id as any).tb}:${(c.id as any).id}`;
        } else if ((c.id as any).id.String) {
          contactIdStr = `${(c.id as any).tb}:${(c.id as any).id.String}`;
        }
      }
    }
    
    let feeContactId = '';
    if (typeof proposal.contact_id === 'string') {
      feeContactId = proposal.contact_id;
    } else if (proposal.contact_id && typeof proposal.contact_id === 'object') {
      if ((proposal.contact_id as any).tb && (proposal.contact_id as any).id) {
        if (typeof (proposal.contact_id as any).id === 'string') {
          feeContactId = `${(proposal.contact_id as any).tb}:${(proposal.contact_id as any).id}`;
        } else if ((proposal.contact_id as any).id.String) {
          feeContactId = `${(proposal.contact_id as any).tb}:${(proposal.contact_id as any).id.String}`;
        }
      }
    }
    
    const id1 = feeContactId.replace('contacts:', '');
    const id2 = contactIdStr.replace('contacts:', '');
    return id1 === id2 || feeContactId === contactIdStr;
  }) : null;

  // Load related data when component mounts
  onMount(() => {
    projectsActions.load();
    companiesActions.load();
    contactsActions.load();
  });
  
  function handleEdit() {
    dispatch('edit', proposal);
  }
  
  function handleClose() {
    dispatch('close');
  }
  
  function formatIssueDate(dateStr: string): string {
    if (dateStr.length === 6) {
      return new Date(`20${dateStr.substring(0,2)}-${dateStr.substring(2,4)}-${dateStr.substring(4,6)}`).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
    }
    return new Date(dateStr).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }
  
  // Project creation workflow with existence check
  async function handleCreateProject() {
    if (!proposal || !relatedProject) {
      console.error('Cannot create project: missing proposal or related project data');
      return;
    }
    
    try {
      const rawProjectNumber = relatedProject.number?.id || '';
      const projectName = relatedProject.name_short || relatedProject.name || '';
      const proposalId = extractId(proposal.id);
      
      // Strip angle brackets from project number if present
      const projectNumber = rawProjectNumber.replace(/[⟨⟩]/g, '');
      
      if (!projectNumber || !projectName) {
        console.error('Cannot create project: missing project number or name');
        warningModal = {
          isOpen: true,
          title: 'Missing Information',
          message: 'Cannot create project: missing project number or name',
          confirmText: 'OK',
          cancelText: '',
          onConfirm: null,
          onCancel: null
        };
        return;
      }
      
      // Check if project folder already exists
      const folderExists = await checkProjectFolderExists(projectNumber, projectName);
      
      if (folderExists) {
        // Folder exists - check first for template file (with "Default Values")
        const templateExists = await checkVarJsonTemplateExists(projectNumber, projectName);
        
        if (templateExists) {
          // Template file exists - populate it directly
          const jsonResult = await writeFeeToJson(proposalId);
          
          warningModal = {
            isOpen: true,
            title: 'Success',
            message: `Project folder exists with template file.\n\nTemplate populated with fee proposal data and renamed to final JSON file!`,
            confirmText: 'OK',
            cancelText: '',
            onConfirm: null,
            onCancel: null
          };
        } else {
          // No template file - check if final var.json file exists
          const jsonExists = await checkVarJsonExists(projectNumber, projectName);
          
          if (jsonExists) {
            // Final JSON file exists - ask for overwrite permission
            warningModal = {
              isOpen: true,
              title: 'JSON File Already Exists',
              message: `Project folder and var.json file already exist!\n\nDo you want to rename the existing var.json with _old suffix and create a new one?`,
              confirmText: 'Overwrite',
              cancelText: 'Cancel',
              onConfirm: async () => {
                try {
                  // Rename existing JSON file with _old suffix
                  const renameResult = await renameVarJsonWithOldSuffix(projectNumber, projectName);
                  
                  // Now create new JSON file
                  const jsonResult = await writeFeeToJson(proposalId);
                  
                  warningModal = {
                    isOpen: true,
                    title: 'Success',
                    message: `Existing var.json renamed with _old suffix.\n\nNew JSON file created with fee proposal data!`,
                    confirmText: 'OK',
                    cancelText: '',
                    onConfirm: null,
                    onCancel: null
                  };
                } catch (error) {
                  console.error('Failed to overwrite JSON file:', error);
                  warningModal = {
                    isOpen: true,
                    title: 'Error',
                    message: `Failed to overwrite JSON file:\n\n${error}`,
                    confirmText: 'OK',
                    cancelText: '',
                    onConfirm: null,
                    onCancel: null
                  };
                }
              },
              onCancel: null
            };
          } else {
            // Folder exists but no JSON files - just create JSON
            const jsonResult = await writeFeeToJson(proposalId);
            
            warningModal = {
              isOpen: true,
              title: 'Success',
              message: `Project folder already exists.\n\nJSON file created with fee proposal data!`,
              confirmText: 'OK',
              cancelText: '',
              onConfirm: null,
              onCancel: null
            };
          }
        }
        
      } else {
        // Folder doesn't exist - create folder then update JSON
        
        // Step 1: Copy project template
        const copyResult = await copyProjectTemplate(projectNumber, projectName);
        
        // Step 2: Write fee proposal data to JSON
        const jsonResult = await writeFeeToJson(proposalId);
        
        warningModal = {
          isOpen: true,
          title: 'Success',
          message: `Project creation completed successfully!\n\nFolder: ${projectNumber} ${projectName}\nJSON file populated with fee proposal data.`,
          confirmText: 'OK',
          cancelText: '',
          onConfirm: null,
          onCancel: null
        };
      }
      
    } catch (error) {
      console.error('Failed to create/update project:', error);
      warningModal = {
        isOpen: true,
        title: 'Error',
        message: `Failed to create/update project:\n\n${error}`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
    }
  }
  
  // Custom actions for the detail panel
  $: customActions = [
    {
      handler: handleCreateProject,
      label: 'Create Project Folder',
      tooltip: 'Create project folder with template and populate data',
      icon: 'M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4',
      disabled: !proposal || !relatedProject
    }
  ];
</script>

{#if proposal}
<DetailPanel 
  bind:isOpen 
  title="proposal"
  {customActions}
  on:edit={handleEdit}
  on:close={handleClose}
>
  <svelte:fragment slot="header">
    <DetailHeader 
      name="{proposal.number} - {proposal.name}"
      subtitle="{relatedProject?.name || 'Unknown Project'}{proposal.package ? ` • ${proposal.package}` : ''}"
      location="{relatedCompany?.city || 'Unknown'}, {relatedCompany?.country || 'Unknown'}"
      stats={[
        { label: 'Revision', value: proposal.rev || 0 },
        { label: 'Days Active', value: Math.floor((new Date().getTime() - new Date(proposal.time.created_at).getTime()) / (1000 * 60 * 60 * 24)) }
      ]}
    />
  </svelte:fragment>
  
  <svelte:fragment slot="content">
    <!-- Proposal Information Section -->
    <InfoCard 
      title="Proposal Information" 
      columns={3}
      fields={[
        { label: 'FP Number', value: proposal.number },
        { label: 'Status', value: proposal.status },
        { label: 'Revision', value: proposal.rev.toString() },
        { label: 'Issue Date', value: formatIssueDate(proposal.issue_date) },
        { label: 'Staff Member', value: proposal.staff_name || '—' },
        { label: 'Created', value: proposal.time.created_at, type: 'date' },
        { label: 'Last Updated', value: proposal.time.updated_at, type: 'date' },
        { label: 'Record ID', value: extractId(proposal.id), type: 'id' }
      ]}
    />
    
    <!-- Project Information Section -->
    {#if relatedProject}
      <InfoCard 
        title="Related Project" 
        columns={2}
        fields={[
          { label: 'Project Number', value: relatedProject.number?.id || '—' },
          { label: 'Project Name', value: relatedProject.name },
          { label: 'Short Name', value: relatedProject.name_short },
          { label: 'Status', value: relatedProject.status },
          { label: 'Area', value: relatedProject.area },
          { label: 'Location', value: `${relatedProject.city}, ${relatedProject.country}` }
        ]}
      />
    {:else}
      <section>
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">Related Project</h2>
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p class="text-emittiv-light text-sm">Project not found</p>
        </div>
      </section>
    {/if}
    
    <!-- Company Information Section -->
    {#if relatedCompany}
      <InfoCard 
        title="Client Company" 
        columns={2}
        fields={[
          { label: 'Company Name', value: relatedCompany.name },
          { label: 'Short Name', value: relatedCompany.name_short },
          { label: 'Abbreviation', value: relatedCompany.abbreviation },
          { label: 'Location', value: `${relatedCompany.city}, ${relatedCompany.country}` },
          { label: 'Registration', value: relatedCompany.reg_no || '—' },
          { label: 'Tax Number', value: relatedCompany.tax_no || '—' }
        ]}
      />
    {:else}
      <section>
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">Client Company</h2>
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
          </svg>
          <p class="text-emittiv-light text-sm">Company not found</p>
        </div>
      </section>
    {/if}
    
    <!-- Contact Information Section -->
    {#if relatedContact}
      <InfoCard 
        title="Primary Contact" 
        columns={2}
        fields={[
          { label: 'Full Name', value: relatedContact.full_name },
          { label: 'Position', value: relatedContact.position || '—' },
          { label: 'Email', value: relatedContact.email },
          { label: 'Phone', value: relatedContact.phone || '—' }
        ]}
      />
    {:else}
      <section>
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider mb-2">Primary Contact</h2>
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
          </svg>
          <p class="text-emittiv-light text-sm">Contact not found</p>
        </div>
      </section>
    {/if}
  </svelte:fragment>
</DetailPanel>
{/if}

<!-- Warning/Success Modal -->
<WarningModal
  bind:isOpen={warningModal.isOpen}
  title={warningModal.title}
  message={warningModal.message}
  confirmText={warningModal.confirmText}
  cancelText={warningModal.cancelText}
  onConfirm={warningModal.onConfirm}
  onCancel={warningModal.onCancel}
/>