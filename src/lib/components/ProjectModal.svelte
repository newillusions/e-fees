<!--
  Refactored Project Modal using BaseModal, FormInput, and FormSelect components
  Reduced from ~519 lines to ~320 lines using base components and utilities
-->
<script lang="ts">
  import { projectsActions, feesStore } from '$lib/stores';
  import { getEntityId, compareSurrealIds } from '$lib/utils/surrealdb';
  import { validateForm, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import { logger } from '$lib/services/logger';
  import { PROJECT_STATUS_OPTIONS, type ProjectStatus } from '$lib/constants';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import Button from './Button.svelte';
  import StatusChangeModal from './StatusChangeModal.svelte';
  import type { Project, Fee } from '../../types';
  
  let { isOpen = $bindable(false), project = null as Project | null, mode = 'create' as 'create' | 'edit', onclose }: {
    isOpen?: boolean;
    project?: Project | null;
    mode?: 'create' | 'edit';
    onclose?: () => void;
  } = $props();
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Status change modal state
  let showStatusChangeModal = $state(false);
  let pendingStatusChange = $state('');
  let originalProject: Project | null = $state(null);
  
  // Form data with better typing
  interface ProjectFormData {
    name: string;
    name_short: string;
    area: string;
    city: string;
    country: string;
    folder: string;
    status: ProjectStatus;
  }
  
  let formData: ProjectFormData = $state({
    name: '',
    name_short: '',
    area: '',
    city: '',
    country: '',
    folder: '',
    status: 'Lead'
  });
  
  // Validation setup
  const validationRules = [
    { field: 'name' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 255 },
    { field: 'name_short' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'area' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 100 },
    { field: 'city' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'country' as keyof ProjectFormData, required: true, minLength: 1, maxLength: 50 },
    { field: 'status' as keyof ProjectFormData, required: true, minLength: 1 }
  ];
  
  // Form validation state
  let formErrors: Record<string, string> = $state({});

  // UI state
  let showDeleteConfirm = $state(false);
  
  
  // Form submission handler
  function handleSubmit(event: Event) {
    event.preventDefault();
    
    // Validate using the validation system
    const errors = validateForm(formData, validationRules);
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
  
  // Create project with loading state
  async function handleCreate() {
    await withLoadingState(async () => {
      const timestamp = new Date().toISOString();
      const projectData = {
        ...formData,
        time: {
          created_at: timestamp,
          updated_at: timestamp
        }
      };
      
      const result = await projectsActions.create(projectData);
      operationActions.setMessage('Project created successfully');
      resetForm();
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Update project with loading state  
  async function handleUpdate() {
    if (!project || !originalProject) return;
    
    // Check if status has changed
    const statusChanged = formData.status !== originalProject.status;
    
    if (statusChanged) {
      // Show status change confirmation modal
      pendingStatusChange = formData.status;
      showStatusChangeModal = true;
      return;
    }
    
    // If no status change, proceed with normal update
    await performProjectUpdate();
  }
  
  // Perform the actual project update
  async function performProjectUpdate() {
    if (!project) return;
    
    await withLoadingState(async () => {
      const projectId = getEntityId(project);
      if (!projectId) {
        logger.error('Failed to extract project ID', { project });
        throw new Error('Invalid project ID');
      }
      
      const projectData = {
        ...formData,
        time: {
          created_at: project.time?.created_at || new Date().toISOString(),
          updated_at: new Date().toISOString()
        }
      };
      
      const result = await projectsActions.update(projectId, projectData);
      operationActions.setMessage('Project updated successfully');
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Handle status change confirmation
  async function handleStatusChangeConfirm(detail: Record<string, unknown>) {
    const { project: projectData, newStatus, folderChangeRequired, affectedFees, feesToUpdate, suggestedFeeStatus } = detail as any;
    
    // Update the form data with the new status
    formData.status = newStatus;
    
    // Close the status change modal
    showStatusChangeModal = false;
    
    await withLoadingState(async () => {
      let folderResult = null;
      
      // Perform folder movement if required
      if (folderChangeRequired && project?.number?.id) {
        const { moveProjectFolder } = await import('$lib/api/folderManagement');
        try {
          folderResult = await moveProjectFolder(project.number.id, newStatus);
        } catch (error) {
          logger.error('Folder movement failed', { error });
          folderResult = {
            success: false,
            message: `Failed to move folder: ${error}`,
            old_path: undefined,
            new_path: undefined
          };
        }
      }
      
      // Perform the project update
      await performProjectUpdate();
      
      // Update selected fee proposals if any
      let updatedCount = 0;
      if (feesToUpdate && feesToUpdate.length > 0) {
        for (const feeUpdate of feesToUpdate) {
          try {
            const { feesActions } = await import('$lib/stores');
            await feesActions.updateStatus(feeUpdate.id, feeUpdate.newStatus);
            updatedCount++;
          } catch (error) {
            logger.error('Failed to update fee', { feeId: feeUpdate.id, error });
          }
        }
      }
      
      // Build success message
      let message = 'Project updated successfully';
      if (folderResult && folderResult.success) {
        message += ` and ${folderResult.message}`;
      }
      if (updatedCount > 0) {
        message += `. ${updatedCount} proposal(s) updated to ${suggestedFeeStatus}`;
      }
      
      operationActions.setMessage(message);
      return true;
    }, operationActions, 'saving');
  }
  
  // Handle status change cancellation
  function handleStatusChangeCancel() {
    // Revert the status back to original
    formData.status = (originalProject?.status as ProjectStatus) || 'Lead';
    showStatusChangeModal = false;
    pendingStatusChange = '';
  }
  
  // Delete project with loading state
  async function handleDelete() {
    if (!project || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      const projectId = getEntityId(project);
      if (!projectId) {
        logger.error('Failed to extract project ID', { project });
        throw new Error('Invalid project ID');
      }
      
      const result = await projectsActions.delete(projectId);
      operationActions.setMessage('Project deleted successfully');
      closeModal();
      return result;
    }, operationActions, 'deleting');
  }
  
  // Form management
  function resetForm() {
    formData = {
      name: '',
      name_short: '',
      area: '',
      city: '',
      country: '',
      folder: '',
      status: 'Lead'
    };
    formErrors = {};
    showDeleteConfirm = false;
  }
  
  function closeModal() {
    resetForm();
    operationActions.reset();
    onclose?.();
  }
  
  // Get related fees for impact analysis (uses utility for SurrealDB ID comparison)
  const relatedFees = $derived(project
    ? $feesStore.filter(fee => fee.project_id && project?.id && compareSurrealIds(fee.project_id, project.id))
    : []);

  // Load form data when project changes
  $effect(() => {
    if (project && mode === 'edit') {
      // Store original project for comparison
      originalProject = { ...project };

      formData = {
        name: project.name || '',
        name_short: project.name_short || '',
        area: project.area || '',
        city: project.city || '',
        country: project.country || '',
        folder: project.folder || '',
        status: (project.status as ProjectStatus) || 'Lead'
      };
    }
  });
</script>

<BaseModal 
  {isOpen} 
  title={mode === 'create' ? 'New Project' : 'Edit Project'}
  maxWidth="500px"
  onclose={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} class="emittiv-form-section emittiv-form-section--wide">
    
    <!-- PROJECT INFORMATION SECTION -->
    <div>
      <h3 class="emittiv-form-section__title">
        Project Information
      </h3>
      <div class="emittiv-form-section">
        
        <!-- Project Name -->
        <FormInput
          label="Project Name"
          bind:value={formData.name}
          placeholder="Full project name"
          required
          error={formErrors.name}
        />
        
        <!-- Short Name -->
        <FormInput
          label="Short Name"
          bind:value={formData.name_short}
          placeholder="Short name"
          required
          error={formErrors.name_short}
        />
        
        <!-- Location: Country, City, Area -->
        <div class="emittiv-form-grid">
          <FormInput
            label="Country"
            bind:value={formData.country}
            placeholder="Country"
            required
            error={formErrors.country}
          />
          
          <FormInput
            label="City"
            bind:value={formData.city}
            placeholder="City"
            required
            error={formErrors.city}
          />
        </div>
        
        <FormInput
          label="Area"
          bind:value={formData.area}
          placeholder="Area/District"
          required
          error={formErrors.area}
        />
        
        <!-- Status and Folder -->
        <div class="emittiv-form-grid">
          <FormSelect
            label="Status"
            bind:value={formData.status}
            required
            error={formErrors.status}
            options={PROJECT_STATUS_OPTIONS}
          />
          
          <FormInput
            label="Folder Path"
            bind:value={formData.folder}
            placeholder="Optional folder path"
            error={formErrors.folder}
          />
        </div>
      </div>
    </div>
    
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
      <div class="emittiv-alert emittiv-alert--error">
        <p class="font-medium mb-2">Are you sure you want to delete this project?</p>
        <p class="text-xs opacity-80">This action cannot be undone.</p>
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
            disabled={$operationState.saving || $operationState.deleting}
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
              disabled={$operationState.saving || $operationState.deleting}
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
        <!-- Delete Confirmation: Confirm/Cancel delete buttons -->
        <div class="emittiv-modal__actions-group" style="justify-content: center;">
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
            disabled={$operationState.saving}
          >
            {#if $operationState.saving}
              <div class="emittiv-spinner-sm"></div>
            {/if}
            Create Project
          </Button>
        </div>
      {/if}
    </div>
  </form>
</BaseModal>

<!-- Status Change Confirmation Modal -->
<StatusChangeModal
  isOpen={showStatusChangeModal}
  project={originalProject}
  newStatus={pendingStatusChange}
  relatedFees={relatedFees}
  mode="project-primary"
  onconfirm={handleStatusChangeConfirm}
  oncancel={handleStatusChangeCancel}
/>