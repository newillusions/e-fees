<!--
  Refactored Project Modal using BaseModal, FormInput, and FormSelect components
  Reduced from ~519 lines to ~320 lines using base components and utilities
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { projectsActions } from '$lib/stores';
  import { extractSurrealId } from '$lib/utils/surrealdb';
  import { validateForm, hasValidationErrors } from '$lib/utils/validation';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import BaseModal from './BaseModal.svelte';
  import FormInput from './FormInput.svelte';
  import FormSelect from './FormSelect.svelte';
  import Button from './Button.svelte';
  import type { Project } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let project: Project | null = null;
  export let mode: 'create' | 'edit' = 'create';
  
  // Use the new operation state utility
  const { store: operationState, actions: operationActions } = useOperationState();
  
  // Form data with better typing
  interface ProjectFormData {
    name: string;
    name_short: string;
    area: string;
    city: string;
    country: string;
    folder: string;
    status: 'Draft' | 'RFP' | 'Active' | 'Awarded' | 'Completed' | 'Lost' | 'Cancelled' | 'On Hold' | 'Revised';
  }
  
  let formData: ProjectFormData = {
    name: '',
    name_short: '',
    area: '',
    city: '',
    country: '',
    folder: '',
    status: 'Draft'
  };
  
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
  let formErrors: Record<string, string> = {};
  
  // UI state
  let showDeleteConfirm = false;
  
  // Status options
  const statusOptions = [
    { value: 'Draft', label: 'Draft' },
    { value: 'RFP', label: 'RFP' },
    { value: 'Active', label: 'Active' },
    { value: 'Awarded', label: 'Awarded' },
    { value: 'Completed', label: 'Completed' },
    { value: 'Lost', label: 'Lost' },
    { value: 'Cancelled', label: 'Cancelled' },
    { value: 'On Hold', label: 'On Hold' },
    { value: 'Revised', label: 'Revised' }
  ];
  
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
    if (!project) return;
    
    await withLoadingState(async () => {
      // Try to extract ID from project.id first, then from project itself
      const projectId = extractSurrealId(project.id) || extractSurrealId(project);
      if (!projectId) {
        console.error('Failed to extract project ID from:', project);
        throw new Error('Invalid project ID');
      }
      
      const projectData = {
        ...formData,
        time: {
          created_at: project.time.created_at,
          updated_at: new Date().toISOString()
        }
      };
      
      const result = await projectsActions.update(projectId, projectData);
      operationActions.setMessage('Project updated successfully');
      closeModal();
      return result;
    }, operationActions, 'saving');
  }
  
  // Delete project with loading state
  async function handleDelete() {
    if (!project || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      // Try to extract ID from project.id first, then from project itself
      const projectId = extractSurrealId(project.id) || extractSurrealId(project);
      if (!projectId) {
        console.error('Failed to extract project ID from:', project);
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
      status: 'Draft'
    };
    formErrors = {};
    showDeleteConfirm = false;
  }
  
  function closeModal() {
    resetForm();
    operationActions.reset();
    dispatch('close');
  }
  
  // Load form data when project changes
  $: if (project && mode === 'edit') {
    formData = {
      name: project.name || '',
      name_short: project.name_short || '',
      area: project.area || '',
      city: project.city || '',
      country: project.country || '',
      folder: project.folder || '',
      status: project.status || 'Draft'
    };
  }
</script>

<BaseModal 
  {isOpen} 
  title={mode === 'create' ? 'New Project' : 'Edit Project'}
  maxWidth="500px"
  on:close={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
    
    <!-- PROJECT INFORMATION SECTION -->
    <div>
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        Project Information
      </h3>
      <div style="display: flex; flex-direction: column; gap: 12px;">
        
        <!-- Project Name -->
        <FormInput
          label="Project Name"
          bind:value={formData.name}
          placeholder="Full project name"
          required
          error={formErrors.name}
        />
        
        <!-- Short Name and Area -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="Short Name"
            bind:value={formData.name_short}
            placeholder="Short name"
            required
            error={formErrors.name_short}
          />
          
          <FormInput
            label="Area"
            bind:value={formData.area}
            placeholder="Area/District"
            required
            error={formErrors.area}
          />
        </div>
        
        <!-- Location -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormInput
            label="City"
            bind:value={formData.city}
            placeholder="City"
            required
            error={formErrors.city}
          />
          
          <FormInput
            label="Country"
            bind:value={formData.country}
            placeholder="Country"
            required
            error={formErrors.country}
          />
        </div>
        
        <!-- Status and Folder -->
        <div class="grid grid-cols-2" style="gap: 12px;">
          <FormSelect
            label="Status"
            bind:value={formData.status}
            required
            error={formErrors.status}
            options={statusOptions}
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
      <div class="text-red-400 text-sm bg-red-900/20 border border-red-500/30 rounded p-3">
        <p class="font-medium mb-2">Are you sure you want to delete this project?</p>
        <p class="text-xs opacity-80">This action cannot be undone.</p>
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
            className="!bg-red-600 !text-white hover:!bg-red-700 !border !border-red-500 h-full !py-2 !flex !items-center !justify-center"
            on:click={() => showDeleteConfirm = true}
            disabled={$operationState.saving || $operationState.deleting}
          >
            Delete
          </Button>
          
          <div class="flex h-full" style="gap: 12px;">
            <Button
              variant="secondary"
              size="sm"
              className="h-full !py-2 !flex !items-center !justify-center"
              on:click={closeModal}
              disabled={$operationState.saving || $operationState.deleting}
            >
              Cancel
            </Button>
            
            <Button
              type="submit"
              variant="primary"
              size="sm"
              className="h-full !py-2 !flex !items-center !justify-center"
              disabled={$operationState.saving || $operationState.deleting}
            >
              {#if $operationState.saving}
                <div 
                  class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
                  style="width: 14px; height: 14px; margin-right: 6px;"
                ></div>
              {/if}
              Update Project
            </Button>
          </div>
        </div>
      {:else if mode === 'edit' && showDeleteConfirm}
        <!-- Delete Confirmation: Confirm/Cancel delete buttons -->
        <div class="flex justify-center items-stretch h-full" style="gap: 12px;">
          <Button
            variant="ghost"
            size="sm"
            className="!bg-red-600 !text-white hover:!bg-red-700 !border !border-red-500 h-full !py-2 !flex !items-center !justify-center"
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
            className="h-full !py-2 !flex !items-center !justify-center"
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
            className="h-full !py-2 !flex !items-center !justify-center"
            on:click={closeModal}
            disabled={$operationState.saving}
          >
            Cancel
          </Button>
          
          <Button
            type="submit"
            variant="primary"
            size="sm"
            className="h-full !py-2 !flex !items-center !justify-center"
            disabled={$operationState.saving}
          >
            {#if $operationState.saving}
              <div 
                class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
                style="width: 14px; height: 14px; margin-right: 6px;"
              ></div>
            {/if}
            Create Project
          </Button>
        </div>
      {/if}
    </div>
  </form>
</BaseModal>