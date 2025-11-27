<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { feesStore, feesActions, companiesStore, companiesActions, settingsStore, settingsActions } from '$lib/stores';
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
  
  // Filter fees for this project
  $: projectFees = project ? $feesStore.filter(fee => {
    if (!fee.project_id || !project?.id) return false;
    
    let feeProjectId = '';
    if (typeof fee.project_id === 'string') {
      feeProjectId = fee.project_id;
    } else if (fee.project_id && typeof fee.project_id === 'object') {
      if ((fee.project_id as any).tb && (fee.project_id as any).id) {
        if (typeof (fee.project_id as any).id === 'string') {
          feeProjectId = `${(fee.project_id as any).tb}:${(fee.project_id as any).id}`;
        } else if ((fee.project_id as any).id.String) {
          feeProjectId = `${(fee.project_id as any).tb}:${(fee.project_id as any).id.String}`;
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
    
    const id1 = feeProjectId.replace('projects:', '');
    const id2 = projectIdStr.replace('projects:', '');
    return id1 === id2 || feeProjectId === projectIdStr;
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
    feesActions.load();
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
      const folderExists = await checkProjectFolderExists(projectNumber, projectName);
      
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
              const renameResult = await renameFolderWithOldSuffix(projectNumber, projectName);
              
              // Now create new folder
              const copyResult = await copyProjectTemplate(projectNumber, projectName);
              
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
      const copyResult = await copyProjectTemplate(projectNumber, projectName);
      
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

<DetailPanel 
  {isOpen}
  show={!!project}
  title="project"
  {customActions}
  on:edit={handleEdit}
  on:close={handleClose}
>
  <svelte:fragment slot="header">
    {#if project}
      <DetailHeader 
        name="{project.number?.id} - {project.name}"
        subtitle="{project.name_short} • {project.area}"
        location="{project.city}, {project.country}"
        stats={[
          { label: 'Proposals', value: projectFees.length },
          { label: 'Awarded', value: projectFees.filter(fee => fee.status === 'Awarded').length },
          { label: 'Pending', value: projectFees.filter(fee => fee.status === 'Sent').length },
          { label: 'Lost', value: projectFees.filter(fee => fee.status === 'Lost').length }
        ]}
      />
    {/if}
  </svelte:fragment>
  
  <svelte:fragment slot="content">
    {#if project}
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
          {projectFees.length} total
        </span>
      </div>
      
      {#if projectFees.length === 0}
        <div class="bg-emittiv-black/50 rounded-xl p-8 text-center border border-emittiv-dark/50">
          <svg class="w-12 h-12 mx-auto mb-3 text-emittiv-dark" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-emittiv-light text-sm">No proposals yet</p>
        </div>
      {:else}
        <div class="grid gap-2">
          {#each projectFees as fee}
            <ListCard clickable={false}>
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <div>
                    <h3 class="text-xs font-medium text-emittiv-light">Fee Number:</h3>
                    <p class="text-sm text-emittiv-white">{fee.number}</p>
                  </div>
                  <div class="mt-2">
                    <h3 class="text-xs font-medium text-emittiv-light">Proposal Name:</h3>
                    <p class="text-sm text-emittiv-lighter">{fee.name}{#if fee.package} - {fee.package}{/if}</p>
                  </div>
                  <div class="mt-2 space-y-1">
                    <div>
                      <h3 class="text-xs font-medium text-emittiv-light">Company:</h3>
                      <p class="text-sm text-emittiv-white">{companyLookup.getCompanyName(fee.company_id) || 'N/A'}</p>
                    </div>
                    {#if fee.staff_name}
                      <div>
                        <h3 class="text-xs font-medium text-emittiv-light">Staff:</h3>
                        <p class="text-sm text-emittiv-white">{fee.staff_name}</p>
                      </div>
                    {/if}
                    <div class="flex items-center gap-4 text-xs text-emittiv-light">
                      <span>Rev: {fee.rev}</span>
                      <span>
                        {new Date(fee.issue_date.length === 6 ? `20${fee.issue_date.substring(0,2)}-${fee.issue_date.substring(2,4)}-${fee.issue_date.substring(4,6)}` : fee.issue_date).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}
                      </span>
                    </div>
                  </div>
                </div>
                <div class="flex items-center gap-2 flex-shrink-0">
                  <StatusBadge status={fee.status} type="proposal" />
                </div>
              </div>
            </ListCard>
          {/each}
        </div>
      {/if}
    </section>
    {/if}
  </svelte:fragment>
</DetailPanel>

<!-- Warning/Success Modal -->
<WarningModal
  isOpen={warningModal.isOpen}
  title={warningModal.title}
  message={warningModal.message}
  confirmText={warningModal.confirmText}
  cancelText={warningModal.cancelText}
  onConfirm={warningModal.onConfirm}
  onCancel={warningModal.onCancel}
  on:close={() => warningModal.isOpen = false}
/>