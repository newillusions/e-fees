<script lang="ts">
  import {
    projectsStore,
    projectsActions,
    companiesStore,
    companiesActions,
    contactsStore,
    contactsActions
  } from '$lib/stores';
  import { onMount } from 'svelte';
  import { extractId, findEntityById } from '$lib/utils';
  import { createCompanyLookup } from '$lib/utils/companyLookup';
  import {
    copyProjectTemplate,
    writeFeeToJson,
    writeFeeToJsonSafe,
    checkProjectFolderExists,
    checkVarJsonExists,
    checkVarJsonTemplateExists,
    renameVarJsonWithOldSuffix
  } from '$lib/api';
  import { cloneFeeRevision, getFeesForProject, exportFeeTemplate } from '$lib/api/revisions';
  import { save } from '@tauri-apps/plugin-dialog';
  import DetailPanel from './DetailPanel.svelte';
  import DetailHeader from './DetailHeader.svelte';
  import InfoCard from './InfoCard.svelte';
  import WarningModal from './WarningModal.svelte';
  import { logger, logApiError } from '$lib/services/logger';
  import type { Fee, Project, Company, Contact } from '../../types';

  let {
    isOpen = $bindable(false),
    proposal = null,
    onedit,
    onclose
  }: {
    isOpen?: boolean;
    proposal?: Fee | null;
    onedit?: (proposal: Fee | null) => void;
    onclose?: () => void;
  } = $props();

  // Modal state
  let warningModal: {
    isOpen: boolean;
    title: string;
    message: string;
    confirmText: string;
    cancelText: string;
    onConfirm: (() => void | Promise<void>) | null;
    onCancel: (() => void) | null;
  } = $state({
    isOpen: false,
    title: 'Warning',
    message: '',
    confirmText: 'OK',
    cancelText: '',
    onConfirm: null,
    onCancel: null
  });

  // Revision state
  let projectRevisions: Fee[] = $state([]);
  let loadingRevisions = $state(false);

  // Create optimized company lookup
  const companyLookup = $derived(createCompanyLookup($companiesStore));

  // Find related project using type-safe utility
  const relatedProject = $derived(
    proposal?.project_id ? findEntityById($projectsStore, proposal.project_id) : null
  );

  // Find related company
  const relatedCompany = $derived(proposal ? companyLookup.getCompany(proposal.company_id) : null);

  // Find related contact using type-safe utility
  const relatedContact = $derived(
    proposal?.contact_id ? findEntityById($contactsStore, proposal.contact_id) : null
  );

  // Load related data when component mounts
  onMount(() => {
    projectsActions.load();
    companiesActions.load();
    contactsActions.load();
  });

  function handleEdit() {
    onedit?.(proposal);
  }

  function handleClose() {
    onclose?.();
  }

  function formatIssueDate(dateStr: string): string {
    if (dateStr.length === 6) {
      return new Date(
        `20${dateStr.substring(0, 2)}-${dateStr.substring(2, 4)}-${dateStr.substring(4, 6)}`
      ).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
    }
    return new Date(dateStr).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }

  // Load all revisions for the current proposal's project
  async function loadRevisions() {
    if (!proposal?.project_id) return;
    loadingRevisions = true;
    try {
      const projectId = extractId(proposal.project_id);
      projectRevisions = await getFeesForProject(projectId);
    } catch (error) {
      logApiError('load revisions', error as Error);
      projectRevisions = [];
    } finally {
      loadingRevisions = false;
    }
  }

  // Reload revisions when proposal changes
  $effect(() => {
    if (proposal?.project_id) {
      loadRevisions();
    }
  });

  // Project creation workflow with existence check
  async function handleCreateProject() {
    if (!proposal || !relatedProject) {
      logger.error('Cannot create project: missing proposal or related project data');
      return;
    }

    try {
      const rawProjectNumber = relatedProject.number?.id || '';
      const projectName = relatedProject.name_short || relatedProject.name || '';
      const proposalId = extractId(proposal.id);

      // Strip angle brackets from project number if present
      const projectNumber = rawProjectNumber.replace(/[⟨⟩]/g, '');

      if (!projectNumber || !projectName) {
        logger.error('Cannot create project: missing project number or name');
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
                  logApiError('overwrite JSON file', error as Error);
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
      logApiError('create/update project', error as Error);
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

  // JSON export with safety checks
  async function handleExportToJson() {
    if (!proposal) {
      logger.error('Cannot export: no proposal data');
      return;
    }

    try {
      const proposalId = extractId(proposal.id);
      const result = await writeFeeToJsonSafe(proposalId);

      if (result) {
        // Parse the result to show safety actions taken
        const lines = result.split('\n');
        const filePath = lines[0].replace('Successfully wrote fee proposal data to: ', '');

        let message = `JSON file export completed successfully!\n\nFile: ${filePath}`;

        // Add safety actions if present
        const safetyIndex = lines.findIndex(line => line.includes('Safety actions taken:'));
        if (safetyIndex !== -1) {
          const safetyActions = lines
            .slice(safetyIndex + 1)
            .filter(line => line.trim().startsWith('•'));
          if (safetyActions.length > 0) {
            message += '\n\nSafety actions taken:';
            safetyActions.forEach(action => {
              message += `\n${action.trim()}`;
            });
          }
        }

        warningModal = {
          isOpen: true,
          title: 'Export Successful',
          message,
          confirmText: 'OK',
          cancelText: '',
          onConfirm: null,
          onCancel: null
        };
      } else {
        throw new Error('Export failed - no result returned');
      }
    } catch (error) {
      logApiError('export JSON', error as Error);
      warningModal = {
        isOpen: true,
        title: 'Export Failed',
        message: `Failed to export proposal to JSON:\n\n${error}`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
    }
  }

  // Create new revision from current proposal
  async function handleNewRevision() {
    if (!proposal) return;
    try {
      const feeId = extractId(proposal.id);
      const newFee = await cloneFeeRevision(feeId);
      warningModal = {
        isOpen: true,
        title: 'Revision Created',
        message: `New revision FP-${String(newFee.rev).padStart(2, '0')} created successfully.`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
      // Reload revisions list
      loadRevisions();
    } catch (error) {
      logApiError('create revision', error as Error);
      warningModal = {
        isOpen: true,
        title: 'Error',
        message: `Failed to create revision:\n\n${error}`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
    }
  }

  // Update pricing in project folder (in-place save)
  async function handleUpdatePricing() {
    if (!proposal) return;
    try {
      const feeId = extractId(proposal.id);
      // No outputPath → backend creates next PRI version in project folder
      const path = await exportFeeTemplate(feeId);
      const filename = path.split('/').pop() || path.split('\\').pop() || path;
      warningModal = {
        isOpen: true,
        title: 'Pricing Exported',
        message: `New pricing version saved:\n\n${filename}`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
    } catch (error) {
      logApiError('export pricing', error as Error);
      warningModal = {
        isOpen: true,
        title: 'Export Failed',
        message: `Failed to export pricing:\n\n${error}`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
    }
  }

  // Export pricing as a copy to a chosen location
  async function handleExportTemplateAs() {
    if (!proposal) return;
    try {
      const feeId = extractId(proposal.id);
      const safeNumber = proposal.number.replace(/[/\\:*?"<>|]/g, '-');
      const defaultName = `${safeNumber}-${String(proposal.rev).padStart(2, '0')} Pricing.xlsx`;

      const savePath = await save({
        defaultPath: defaultName,
        filters: [{ name: 'Excel', extensions: ['xlsx'] }]
      });

      if (!savePath) return; // User cancelled

      const path = await exportFeeTemplate(feeId, savePath);
      const filename = path.split('/').pop() || path.split('\\').pop() || path;
      warningModal = {
        isOpen: true,
        title: 'Export Successful',
        message: `Pricing template exported:\n\n${filename}`,
        confirmText: 'OK',
        cancelText: '',
        onConfirm: null,
        onCancel: null
      };
    } catch (error) {
      logApiError('export template', error as Error);
      warningModal = {
        isOpen: true,
        title: 'Export Failed',
        message: `Failed to export pricing template:\n\n${error}`,
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
      handler: handleCreateProject,
      label: 'Create Project Folder',
      tooltip: 'Create project folder with template and populate data',
      icon: 'M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4',
      disabled: !proposal || !relatedProject
    },
    {
      handler: handleExportToJson,
      label: 'Export to JSON',
      tooltip: 'Export proposal data to project JSON file with safety checks',
      icon: 'M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      disabled: !proposal
    },
    {
      handler: handleUpdatePricing,
      label: 'Save Pricing',
      tooltip: proposal?.pricing
        ? 'Save new PRI version to project folder'
        : 'No pricing data — configure pricing first',
      icon: 'M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      disabled: !proposal || !proposal.pricing
    },
    {
      handler: handleExportTemplateAs,
      label: 'Export Pricing As...',
      tooltip: proposal?.pricing
        ? 'Export pricing template to a chosen location'
        : 'No pricing data — configure pricing first',
      icon: 'M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      disabled: !proposal || !proposal.pricing
    },
    {
      handler: handleNewRevision,
      label: 'New Revision',
      tooltip: 'Clone this fee as a new revision (FP-N+1)',
      icon: 'M8 7v8a2 2 0 002 2h6M8 7V5a2 2 0 012-2h4.586a1 1 0 01.707.293l4.414 4.414a1 1 0 01.293.707V15a2 2 0 01-2 2h-2M8 7H6a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2v-2',
      disabled: !proposal
    }
  ]);
</script>

<DetailPanel
  {isOpen}
  show={!!proposal}
  title="proposal"
  canEdit={true}
  {customActions}
  onedit={handleEdit}
  onclose={handleClose}
>
  <svelte:fragment slot="header">
    {#if proposal}
      <DetailHeader
        name="{proposal.number} - {proposal.name}"
        subtitle="{relatedProject?.name || 'Unknown Project'}{proposal.package
          ? ` • ${proposal.package}`
          : ''}"
        location="{relatedCompany?.city || 'Unknown'}, {relatedCompany?.country || 'Unknown'}"
        stats={[
          { label: 'Revision', value: proposal.rev || 0 },
          {
            label: 'Days Active',
            value: proposal.time?.created_at
              ? Math.floor(
                  (new Date().getTime() - new Date(proposal.time.created_at).getTime()) /
                    (1000 * 60 * 60 * 24)
                )
              : 0
          }
        ]}
      />
    {/if}
  </svelte:fragment>

  <svelte:fragment slot="content">
    {#if proposal}
      <!-- Proposal Information Section -->
      <InfoCard
        title="Proposal Information"
        columns={3}
        fields={[
          { label: 'FP Number', value: proposal.number },
          { label: 'Status', value: proposal.status },
          { label: 'Revision', value: (proposal.rev ?? 0).toString() },
          { label: 'Issue Date', value: formatIssueDate(proposal.issue_date) },
          { label: 'Staff Member', value: proposal.staff_name || '—' },
          { label: 'Created', value: proposal.time?.created_at, type: 'date' },
          { label: 'Last Updated', value: proposal.time?.updated_at, type: 'date' },
          { label: 'Record ID', value: extractId(proposal.id), type: 'id' }
        ]}
      />

      <!-- Revision History Section -->
      {#if projectRevisions.length > 1}
        <section>
          <h2 class="emittiv-section-title">Revision History</h2>
          <div class="bg-emittiv-black/50 rounded-xl border border-emittiv-dark/50 overflow-hidden">
            {#each projectRevisions as rev}
              <div
                class="flex items-center justify-between px-4 py-2 border-b border-emittiv-dark/30 last:border-b-0 {extractId(
                  rev.id
                ) === extractId(proposal.id)
                  ? 'bg-emittiv-splash/10 border-l-2 border-l-emittiv-splash'
                  : 'hover:bg-emittiv-darker/50'}"
              >
                <div class="flex items-center gap-3">
                  <span class="text-xs font-mono text-emittiv-lighter"
                    >FP-{String(rev.rev ?? 0).padStart(2, '0')}</span
                  >
                  <span class="text-xs text-emittiv-light">{rev.status}</span>
                </div>
                <div class="flex items-center gap-2">
                  <span class="text-xs text-emittiv-light"
                    >{rev.issue_date ? formatIssueDate(rev.issue_date) : '—'}</span
                  >
                  {#if extractId(rev.id) === extractId(proposal.id)}
                    <span class="text-xxs text-emittiv-splash font-medium">Current</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <!-- Project Information Section -->
      {#if relatedProject}
        <InfoCard
          title="Related Project"
          columns={3}
          fields={[
            { label: 'Project Number', value: relatedProject.number?.id || '—' },
            { label: 'Project Name', value: relatedProject.name },
            { label: 'Short Name', value: relatedProject.name_short || '—' },
            { label: 'Status', value: relatedProject.status },
            { label: 'Area', value: relatedProject.area || '—' },
            { label: 'Location', value: `${relatedProject.city}, ${relatedProject.country}` }
          ]}
        />
      {:else}
        <section>
          <h2 class="emittiv-section-title">Related Project</h2>
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
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
            <p class="text-emittiv-light text-sm">Project not found</p>
          </div>
        </section>
      {/if}

      <!-- Company Information Section -->
      {#if relatedCompany}
        <InfoCard
          title="Client Company"
          columns={3}
          fields={[
            { label: 'Company Name', value: relatedCompany.name },
            { label: 'Short Name', value: relatedCompany.name_short || '—' },
            { label: 'Abbreviation', value: relatedCompany.abbreviation || '—' },
            { label: 'Location', value: `${relatedCompany.city}, ${relatedCompany.country}` },
            { label: 'Registration', value: relatedCompany.reg_no || '—' },
            { label: 'Tax Number', value: relatedCompany.tax_no || '—' }
          ]}
        />
      {:else}
        <section>
          <h2 class="emittiv-section-title">Client Company</h2>
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
                d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"
              />
            </svg>
            <p class="text-emittiv-light text-sm">Company not found</p>
          </div>
        </section>
      {/if}

      <!-- Contact Information Section -->
      {#if relatedContact}
        <InfoCard
          title="Primary Contact"
          columns={3}
          fields={[
            { label: 'Full Name', value: relatedContact.full_name },
            { label: 'Position', value: relatedContact.position || '—' },
            { label: 'Email', value: relatedContact.email },
            { label: 'Phone', value: relatedContact.phone || '—' }
          ]}
        />
      {:else}
        <section>
          <h2 class="emittiv-section-title">Primary Contact</h2>
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
                d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
              />
            </svg>
            <p class="text-emittiv-light text-sm">Contact not found</p>
          </div>
        </section>
      {/if}
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
/>
