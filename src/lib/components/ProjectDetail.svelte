<script lang="ts">
  import {
    feesStore,
    feesActions,
    companiesStore,
    companiesActions,
    settingsStore,
    settingsActions,
    paginatedProjectsStore
  } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId, compareIds } from '$lib/utils';
  import { extractIdFromRelation } from '$lib/utils/surrealdb';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import {
    openFolderInExplorer,
    copyProjectTemplate,
    checkProjectFolderExists,
    renameFolderWithOldSuffix,
    previewProjectDelete,
    deleteProjectCascade
  } from '$lib/api';
  import { previewTrashProjectFolder, executeTrashProjectFolder } from '$lib/api/folderReconcile';
  import { getFolderForStatus } from '$lib/api/folderManagement';
  import DetailPanel from './DetailPanel.svelte';
  import DetailHeader from './DetailHeader.svelte';
  import InfoCard from './InfoCard.svelte';
  import ListCard from './ListCard.svelte';
  import StatusBadge from './StatusBadge.svelte';
  import WarningModal from './WarningModal.svelte';
  import MergeProjectModal from './MergeProjectModal.svelte';
  import { logger, logApiError } from '$lib/services/logger';
  import { projectLogger, feeLogger } from '$lib/services/activityLogger';
  import type { Project, Fee, ProjectMergeResult } from '../../types';

  let {
    isOpen = $bindable(false),
    project = null,
    onedit,
    onclose
  }: {
    isOpen?: boolean;
    project?: Project | null;
    onedit?: (project: Project | null) => void;
    onclose?: () => void;
  } = $props();

  // Inline folder error state
  let folderError = $state('');

  // Merge/delete state
  let showMergeModal = $state(false);
  let deletingProject = $state(false);
  // Cascade-delete's opt-in "also trash the on-disk folder" checkbox.
  // Default OFF (Martin's decision) - only shown at all when the project
  // actually has a resolvable on-disk folder. Reset alongside every
  // warningModal reassignment below so an unrelated dialog never inherits
  // a stale checkbox.
  let trashFolderOnDelete = $state(false);
  let deleteFolderCheckboxLabel = $state('');

  // Modal state
  let warningModal: {
    isOpen: boolean;
    title: string;
    message: string;
    confirmText: string;
    cancelText: string;
    onConfirm: (() => void | Promise<void>) | null;
    onCancel: (() => void) | null;
  } = {
    isOpen: false,
    title: 'Warning',
    message: '',
    confirmText: 'OK',
    cancelText: '',
    onConfirm: null,
    onCancel: null
  };

  // Create optimized company lookup
  const companyLookup = $derived(createCompanyLookup($companiesStore));

  // Helper to parse issue dates for sorting
  const parseIssueDate = (dateStr: string): Date => {
    if (dateStr.length === 6) {
      return new Date(
        `20${dateStr.substring(0, 2)}-${dateStr.substring(2, 4)}-${dateStr.substring(4, 6)}`
      );
    }
    return new Date(dateStr);
  };

  // Filter fees for this project using type-safe comparison
  const projectFees = $derived(
    project?.id
      ? $feesStore
          .filter(fee => compareIds(fee.project_id, project.id))
          .sort(
            (a, b) =>
              parseIssueDate(b.issue_date).getTime() - parseIssueDate(a.issue_date).getTime()
          )
      : []
  );

  // Load related data when component mounts
  onMount(() => {
    feesActions.load();
    companiesActions.load();
    settingsActions.load();
  });

  function handleEdit() {
    onedit?.(project);
  }

  function handleClose() {
    onclose?.();
  }

  // Open the merge modal for the current project.
  function handleOpenMerge() {
    if (!project) return;
    showMergeModal = true;
  }

  function handleMerged(_result: ProjectMergeResult) {
    // The source project (this panel's project) no longer exists - close.
    showMergeModal = false;
    handleClose();
  }

  // Delete this project. Fetches a live preview first so the confirm
  // dialog names exactly what will be removed - a project with fee
  // proposals needs an explicit cascade confirmation, never a silent
  // orphan-and-delete.
  function formatFolderSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  async function handleDeleteProject() {
    if (!project?.id) return;
    const projectKey = extractIdFromRelation(project.id);
    const projectName = project.name || project.number?.id || 'Unknown Project';
    const projectNumber = project.number?.id || '';

    try {
      const [preview, folderPreview] = await Promise.all([
        previewProjectDelete(projectKey),
        // Best-effort: an on-disk folder lookup failing must never block
        // the delete confirm dialog from appearing - fall back to "no
        // folder" so the checkbox is simply omitted.
        projectNumber
          ? previewTrashProjectFolder(projectNumber).catch(() => ({
              project_number: projectNumber,
              folder_exists: false,
              source_path: null,
              file_count: 0,
              total_size_bytes: 0
            }))
          : Promise.resolve(null)
      ]);

      const message =
        preview.dependent_fees.length === 0
          ? `Delete project "${projectName}"?\n\nThis project has no fee proposals and cannot be undone.`
          : `Delete project "${projectName}"?\n\n` +
            `This will also permanently delete ${preview.dependent_fees.length} fee proposal(s):\n` +
            preview.dependent_fees.map(fee => `- ${fee.number} (${fee.status})`).join('\n') +
            '\n\nThis cannot be undone.';

      trashFolderOnDelete = false;
      deleteFolderCheckboxLabel =
        folderPreview && folderPreview.folder_exists
          ? `Also move the on-disk folder (${folderPreview.file_count} file${folderPreview.file_count === 1 ? '' : 's'}, ${formatFolderSize(folderPreview.total_size_bytes)}) to .reconcile-backups`
          : '';

      warningModal = {
        isOpen: true,
        title: 'Delete Project',
        message,
        confirmText: 'Delete',
        cancelText: 'Cancel',
        onConfirm: async () => {
          deletingProject = true;
          try {
            const cascade = preview.dependent_fees.length > 0;
            const result = await deleteProjectCascade(projectKey, cascade);

            paginatedProjectsStore.actions.removeItem(project?.id || '');
            if (result.deleted_fees.length > 0) {
              await feesActions.load();
            }

            projectLogger.onDelete(projectKey, projectName);
            for (const fee of result.deleted_fees) {
              const feeId = extractIdFromRelation(fee.id || '');
              feeLogger.onDelete(feeId, fee.number || fee.name || 'Unknown Fee');
            }

            // Opt-in on-disk trash, deliberately AFTER the DB delete has
            // already committed - decoupled by design, so a failure here
            // never rolls back or blocks the delete that already
            // succeeded. Best-effort: log and move on.
            if (trashFolderOnDelete && projectNumber) {
              try {
                await executeTrashProjectFolder(projectNumber);
              } catch (trashError) {
                logApiError('executeTrashProjectFolder', trashError as Error, {
                  component: 'ProjectDetail'
                });
              }
            }

            handleClose();
          } catch (error) {
            logApiError('deleteProjectCascade', error as Error, {
              component: 'ProjectDetail'
            });
            warningModal = {
              isOpen: true,
              title: 'Error',
              message: `Failed to delete project:\n\n${error}`,
              confirmText: 'OK',
              cancelText: '',
              onConfirm: null,
              onCancel: null
            };
          } finally {
            deletingProject = false;
          }
        },
        onCancel: null
      };
    } catch (error) {
      logApiError('previewProjectDelete', error as Error, { component: 'ProjectDetail' });
      warningModal = {
        isOpen: true,
        title: 'Error',
        message: `Failed to check project dependents:\n\n${error}`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
    }
  }

  // Function to get full project folder path
  function getFullProjectPath(): string {
    if (!project) return '';
    const basePath = $settingsStore.project_folder_path;
    if (!basePath) return project.folder || '';

    // Normalize paths - remove trailing slashes from base and leading slashes from folder
    const isWindows = basePath.includes('\\');
    const separator = isWindows ? '\\' : '/';

    // Normalize base path (remove trailing separators)
    let normalizedBase = basePath;
    while (
      normalizedBase.endsWith(separator) ||
      normalizedBase.endsWith('/') ||
      normalizedBase.endsWith('\\')
    ) {
      normalizedBase = normalizedBase.slice(0, -1);
    }

    // Normalize project folder (remove leading separators)
    let normalizedFolder = project.folder || '';
    while (normalizedFolder.startsWith('/') || normalizedFolder.startsWith('\\')) {
      normalizedFolder = normalizedFolder.slice(1);
    }

    if (!normalizedFolder) return normalizedBase;

    // Get the status-based subfolder (e.g., "01 RFPs", "11 Current")
    const statusFolder = getFolderForStatus(project.status || 'rfp');

    return `${normalizedBase}${separator}${statusFolder}${separator}${normalizedFolder}`;
  }

  // Function to open project folder in explorer
  async function openProjectFolder() {
    if (!project) return;

    const projectFolderPath = $settingsStore.project_folder_path;
    if (!projectFolderPath) {
      folderError = 'Project folder path not configured. Please set it in Settings.';
      setTimeout(() => (folderError = ''), 5000);
      return;
    }

    const fullPath = getFullProjectPath();

    try {
      const result = await openFolderInExplorer(fullPath);
      if (result.includes('Failed')) {
        folderError = 'Failed to open project folder. Please check the path exists.';
        setTimeout(() => (folderError = ''), 5000);
      }
    } catch (error) {
      logApiError('open project folder', error as Error);
      folderError = 'Failed to open project folder. Please check the path exists.';
      setTimeout(() => (folderError = ''), 5000);
    }
  }

  // Handle field click events from InfoCard
  function handleFieldClick(detail: {
    field: {
      label: string;
      value: string | number | undefined;
      type?: string;
      clickable?: boolean;
    };
    index: number;
  }) {
    const { field } = detail;
    if (field.label === 'Folder') {
      openProjectFolder();
    }
  }

  // Project folder creation workflow
  async function handleCreateProjectFolder() {
    if (!project) {
      logger.error('Cannot create project folder: no project data');
      return;
    }

    try {
      const projectNumber = project.number?.id || '';
      const projectName = project.name_short || project.name || '';

      if (!projectNumber || !projectName) {
        logger.error('Cannot create project folder: missing project number or name');
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
              logApiError('overwrite project folder', error as Error);
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
      logApiError('create project folder', error as Error);
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
  const customActions = $derived([
    {
      handler: handleCreateProjectFolder,
      label: 'Create Project Folder',
      tooltip: 'Create project folder with template files',
      icon: 'M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4',
      disabled: !project
    },
    {
      handler: handleOpenMerge,
      label: 'Merge Into Another Project',
      tooltip: 'Merge this project\'s fee proposals into another project, then delete this one',
      icon: 'M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4',
      disabled: !project
    },
    {
      handler: handleDeleteProject,
      label: 'Delete Project',
      tooltip: 'Permanently delete this project',
      icon: 'M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16',
      disabled: !project || deletingProject
    }
  ]);
</script>

{#if folderError}
  <div class="emittiv-alert emittiv-alert--error" style="margin: 8px 0;">{folderError}</div>
{/if}

<DetailPanel
  {isOpen}
  show={!!project}
  title="project"
  {customActions}
  onedit={handleEdit}
  onclose={handleClose}
>
  <svelte:fragment slot="header">
    {#if project}
      <DetailHeader
        name="{project.number?.id} - {project.name}"
        subtitle="{project.name_short} • {project.area}"
        location="{project.city}, {project.country}"
        stats={[
          { label: 'Proposals', value: projectFees.length },
          { label: 'Accepted', value: projectFees.filter(fee => fee.status === 'Accepted').length },
          { label: 'Pending', value: projectFees.filter(fee => fee.status === 'Sent').length },
          { label: 'Rejected', value: projectFees.filter(fee => fee.status === 'Rejected').length }
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
          { label: 'Created', value: project.time?.created_at, type: 'date' },
          { label: 'Last Updated', value: project.time?.updated_at, type: 'date' },
          { label: 'Record ID', value: extractId(project.id), type: 'id' }
        ]}
        onfieldclick={handleFieldClick}
      />

      <!-- Fee Proposals Section -->
      <section>
        <div class="flex items-center justify-between mb-2">
          <h2 class="emittiv-section-title">Fee Proposals</h2>
          <span
            class="text-xs text-emittiv-light px-2 py-1 rounded-lg"
            style="background-color: #111;"
          >
            {projectFees.length} total
          </span>
        </div>

        {#if projectFees.length === 0}
          <div class="emittiv-empty-state">
            <svg
              class="emittiv-empty-state__icon"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
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
                      <p class="text-sm text-emittiv-lighter">
                        {fee.name}{#if fee.package}
                          - {fee.package}{/if}
                      </p>
                    </div>
                    <div class="mt-2 space-y-1">
                      <div>
                        <h3 class="text-xs font-medium text-emittiv-light">Company:</h3>
                        <p class="text-sm text-emittiv-white">
                          {companyLookup.getCompanyName(fee.company_id) || 'N/A'}
                        </p>
                      </div>
                      {#if fee.staff_name}
                        <div>
                          <h3 class="text-xs font-medium text-emittiv-light">Staff:</h3>
                          <p class="text-sm text-emittiv-white">{fee.staff_name}</p>
                        </div>
                      {/if}
                      <div class="emittiv-card-meta">
                        <span>Rev: {fee.rev}</span>
                        <span>
                          {new Date(
                            fee.issue_date.length === 6
                              ? `20${fee.issue_date.substring(0, 2)}-${fee.issue_date.substring(2, 4)}-${fee.issue_date.substring(4, 6)}`
                              : fee.issue_date
                          ).toLocaleDateString('en-US', {
                            month: 'short',
                            day: 'numeric',
                            year: 'numeric'
                          })}
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
  onclose={() => (warningModal.isOpen = false)}
  checkboxLabel={warningModal.title === 'Delete Project' ? deleteFolderCheckboxLabel : ''}
  bind:checkboxChecked={trashFolderOnDelete}
/>

<!-- Merge Project Modal -->
<MergeProjectModal
  bind:isOpen={showMergeModal}
  sourceProject={project}
  onclose={() => (showMergeModal = false)}
  onmerged={handleMerged}
/>
