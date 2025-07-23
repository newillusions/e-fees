<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { rfpsStore, rfpsActions, companiesStore, companiesActions, settingsStore, settingsActions } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId } from '$lib/utils';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import { openFolderInExplorer, copyProjectTemplate, checkProjectFolderExists, renameFolderWithOldSuffix } from '$lib/api';
  import DetailPanel from './DetailPanel.svelte';
  import DetailHeader from './DetailHeader.svelte';
  import InfoCard from './InfoCard.svelte';
  import ListCard from './ListCard.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import WarningModal from './WarningModal.svelte';
  import type { Project, FeeProposal } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let project: Project | null = null;
  
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
  
  // Filter RFPs for this project
  $: projectRfps = project ? $rfpsStore.filter(rfp => {
    if (!rfp.project_id || !project?.id) return false;
    
    let rfpProjectId = '';
    if (typeof rfp.project_id === 'string') {
      rfpProjectId = rfp.project_id;
    } else if (rfp.project_id && typeof rfp.project_id === 'object') {
      if ((rfp.project_id as any).tb && (rfp.project_id as any).id) {
        if (typeof (rfp.project_id as any).id === 'string') {
          rfpProjectId = `${(rfp.project_id as any).tb}:${(rfp.project_id as any).id}`;
        } else if ((rfp.project_id as any).id.String) {
          rfpProjectId = `${(rfp.project_id as any).tb}:${(rfp.project_id as any).id.String}`;
        }
      }
    }
    
    let projectIdStr = '';
    if (typeof project.id === 'string') {
      projectIdStr = project.id;
    } else if (project.id && typeof project.id === 'object') {
      if ((project.id as any).tb && (project.id as any).id) {
        if (typeof (project.id as any).id === 'string') {
          projectIdStr = `${(project.id as any).tb}:${(project.id as any).id}`;
        } else if ((project.id as any).id.String) {
          projectIdStr = `${(project.id as any).tb}:${(project.id as any).id.String}`;
        }
      }
    }
    
    const id1 = rfpProjectId.replace('projects:', '');
    const id2 = projectIdStr.replace('projects:', '');
    return id1 === id2 || rfpProjectId === projectIdStr;
  }).sort((a, b) => {
    // Parse issue_date for proper sorting
    const parseIssueDate = (dateStr: string) => {
      if (dateStr.length === 6) {
        return new Date(`20${dateStr.substring(0,2)}-${dateStr.substring(2,4)}-${dateStr.substring(4,6)}`);
      }
      return new Date(dateStr);
    };
    
    return parseIssueDate(b.issue_date).getTime() - parseIssueDate(a.issue_date).getTime();
  }) : [];

  // Load related data when component mounts
  onMount(() => {
    rfpsActions.load();
    companiesActions.load();
    settingsActions.load();
  });
  
  function handleEdit() {
    dispatch('edit', project);
  }
  
  function handleClose() {
    dispatch('close');
  }
  
  // Function to get full project folder path
  function getFullProjectPath(): string {
    if (!project) return '';
    const basePath = $settingsStore.project_folder_path;
    if (!basePath) return project.folder;
    
    // Use appropriate path separator based on platform
    const separator = basePath.includes('\\') ? '\\' : '/';
    return `${basePath}${separator}${project.folder}`;
  }
  
  // Function to open project folder in explorer
  async function openProjectFolder() {
    if (!project) return;
    
    const projectFolderPath = $settingsStore.project_folder_path;
    if (!projectFolderPath) {
      alert('Project folder path not configured. Please set it in Settings.');
      return;
    }
    
    const fullPath = getFullProjectPath();
    
    try {
      await openFolderInExplorer(fullPath);
    } catch (error) {
      console.error('Failed to open project folder:', error);
      alert('Failed to open project folder. Please check the path exists.');
    }
  }
  
  // Handle field click events from InfoCard
  function handleFieldClick(event: CustomEvent) {
    const { field } = event.detail;
    if (field.label === 'Folder') {
      openProjectFolder();
    }
  }
  
  // Project folder creation workflow
  async function handleCreateProjectFolder() {
    if (!project) {
      console.error('Cannot create project folder: no project data');
      return;
    }
    
    try {
      const projectNumber = project.number?.id || '';
      const projectName = project.name_short || project.name || '';
      
      if (!projectNumber || !projectName) {
        console.error('Cannot create project folder: missing project number or name');
        warningModal = {
          isOpen: true,
          title: 'Missing Information',
          message: 'Cannot create project folder: missing project number or name',
          confirmText: 'OK',
          cancelText: '',
          onConfirm: null,
          onCancel: null
        };
        return;
      }
      
      // First, check if folder already exists
      console.log(`Checking if project folder exists: ${projectNumber} ${projectName}`);
      const folderExists = await checkProjectFolderExists(projectNumber, projectName);
      console.log('Folder exists result:', folderExists);
      
      if (folderExists) {
        warningModal = {
          isOpen: true,
          title: 'Folder Already Exists',
          message: `Project folder "${projectNumber} ${projectName}" already exists!\n\nDo you want to rename the existing folder with _old suffix and create a new one?`,
          confirmText: 'Overwrite',
          cancelText: 'Cancel',
          onConfirm: async () => {
            try {
              // Rename existing folder with _old suffix
              console.log('Renaming existing folder with _old suffix...');
              const renameResult = await renameFolderWithOldSuffix(projectNumber, projectName);
              console.log('Rename result:', renameResult);
              
              // Now create new folder
              console.log(`Creating new project folder for ${projectNumber} ${projectName}`);
              const copyResult = await copyProjectTemplate(projectNumber, projectName);
              console.log('Project folder created:', copyResult);
              
              warningModal = {
                isOpen: true,
                title: 'Success',
                message: `Existing folder renamed with _old suffix.\n\nNew project folder created successfully!`,
                confirmText: 'OK',
                cancelText: '',
                onConfirm: null,
                onCancel: null
              };
            } catch (error) {
              console.error('Failed to overwrite project folder:', error);
              warningModal = {
                isOpen: true,
                title: 'Error',
                message: `Failed to overwrite project folder:\n\n${error}`,
                confirmText: 'OK',
                cancelText: '',
                onConfirm: null,
                onCancel: null
              };
            }
          },
          onCancel: null
        };
        return;
      }
      
      // Create the project folder
      console.log(`Creating project folder for ${projectNumber} ${projectName}`);
      const copyResult = await copyProjectTemplate(projectNumber, projectName);
      console.log('Project folder created:', copyResult);
      
      warningModal = {
        isOpen: true,
        title: 'Success',
        message: `Project folder created successfully!\n\nFolder: ${projectNumber} ${projectName}`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
      
    } catch (error) {
      console.error('Failed to create project folder:', error);
      warningModal = {
        isOpen: true,
        title: 'Error',
        message: `Failed to create project folder:\n\n${error}`,
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
      handler: handleCreateProjectFolder,
      label: 'Create Project Folder',
      tooltip: 'Create project folder with template files',
      icon: 'M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4',
      disabled: !project
    }
  ];
</script>

{#if project}
<DetailPanel 
  bind:isOpen 
  title="project"
  {customActions}
  on:edit={handleEdit}
  on:close={handleClose}
>
  <svelte:fragment slot="header">
    <DetailHeader 
      name="{project.number?.id} - {project.name}"
      subtitle="{project.name_short} • {project.area}"
      location="{project.city}, {project.country}"
      stats={[
        { label: 'Proposals', value: projectRfps.length },
        { label: 'Awarded', value: projectRfps.filter(rfp => rfp.status === 'Awarded').length },
        { label: 'Pending', value: projectRfps.filter(rfp => rfp.status === 'Sent').length },
        { label: 'Lost', value: projectRfps.filter(rfp => rfp.status === 'Lost').length }
      ]}
    />
  </svelte:fragment>
  
  <svelte:fragment slot="content">
    <!-- Project Information Section -->
    <InfoCard 
      title="Project Information" 
      columns={3}
      fields={[
        { label: 'Project Number', value: project.number?.id || '—' },
        { label: 'Status', value: project.status },
        { label: 'Folder', value: project.folder || '—', clickable: true },
        { label: 'Created', value: project.time.created_at, type: 'date' },
        { label: 'Last Updated', value: project.time.updated_at, type: 'date' },
        { label: 'Record ID', value: extractId(project.id), type: 'id' }
      ]}
      on:field-click={handleFieldClick}
    />
    
    <!-- Fee Proposals Section -->
    <section>
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-sm font-medium text-emittiv-light uppercase tracking-wider">Fee Proposals</h2>
        <span class="text-xs text-emittiv-light px-2 py-1 rounded-lg" style="background-color: #111;">
          {projectRfps.length} total
        </span>
      </div>
      
      {#if projectRfps.length === 0}
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-emittiv-light text-sm">No proposals yet</p>
        </div>
      {:else}
        <div class="grid gap-2">
          {#each projectRfps as rfp}
            <ListCard clickable={false}>
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <div>
                    <h3 class="text-xs font-medium text-emittiv-light">RFP Number:</h3>
                    <p class="text-sm text-emittiv-white">{rfp.number}</p>
                  </div>
                  <div class="mt-2">
                    <h3 class="text-xs font-medium text-emittiv-light">Proposal Name:</h3>
                    <p class="text-sm text-emittiv-lighter">{rfp.name}{#if rfp.package} - {rfp.package}{/if}</p>
                  </div>
                  <div class="mt-2 space-y-1">
                    <div>
                      <h3 class="text-xs font-medium text-emittiv-light">Company:</h3>
                      <p class="text-sm text-emittiv-white">{companyLookup.getCompanyName(rfp.company_id) || 'N/A'}</p>
                    </div>
                    {#if rfp.staff_name}
                      <div>
                        <h3 class="text-xs font-medium text-emittiv-light">Staff:</h3>
                        <p class="text-sm text-emittiv-white">{rfp.staff_name}</p>
                      </div>
                    {/if}
                    <div class="flex items-center gap-4 text-xs text-emittiv-light">
                      <span>Rev: {rfp.rev}</span>
                      <span>
                        {new Date(rfp.issue_date.length === 6 ? `20${rfp.issue_date.substring(0,2)}-${rfp.issue_date.substring(2,4)}-${rfp.issue_date.substring(4,6)}` : rfp.issue_date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
                      </span>
                    </div>
                  </div>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                  <StatusBadge status={rfp.status} type="proposal" />
                </div>
              </div>
            </ListCard>
          {/each}
        </div>
      {/if}
    </section>
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