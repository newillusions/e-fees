<!--
  Generic CRUD Modal Component
  
  Replaces CompanyModal, ContactModal, and ProposalModal with a unified interface.
  Supports any entity type with configurable fields and validation.
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { useOperationState, withLoadingState } from '$lib/utils/crud';
  import { validateForm, hasValidationErrors, type ValidationRule } from '$lib/utils/validation';
  import BaseModal from '../BaseModal.svelte';
  import FormField from './FormField.svelte';
  import Button from '../Button.svelte';
  import type { FormFieldConfig } from './types';

  // Generic types
  type T = Record<string, any>;

  const dispatch = createEventDispatcher();

  // Props
  export let isOpen = false;
  export let entity: any | null = null;
  export let mode: 'create' | 'edit' = 'create';
  export let title: string;
  export let fields: FormFieldConfig[];
  export let validationRules: any[] = [];
  export let onSave: (data: any) => Promise<void>;
  export let onDelete: ((entity: any) => Promise<void>) | null = null;
  export let maxWidth = "500px";
  export let customClass = "";

  // Operation state management
  const { store: operationState, actions: operationActions } = useOperationState();

  // Form state
  let formData: T = {};
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;

  // Initialize form data structure based on field configurations
  function initializeFormData() {
    const initialData: T = {};
    
    function processField(field: FormFieldConfig, data: any) {
      if (field.type === 'group' && field.fields) {
        // For group fields, process nested fields
        field.fields.forEach(subField => processField(subField, data));
      } else {
        // For regular fields, set default value
        data[field.name] = field.defaultValue ?? '';
      }
    }
    
    fields.forEach(field => processField(field, initialData));
    return initialData;
  }

  // Reset form to initial state
  function resetForm() {
    formData = initializeFormData();
    formErrors = {};
    showDeleteConfirm = false;
    operationActions.reset();
  }

  // Load entity data into form (for edit mode)
  function loadEntityData() {
    if (!entity || mode !== 'edit') return;
    
    formData = { ...initializeFormData() };
    
    function processField(field: FormFieldConfig, data: any, source: any) {
      if (field.type === 'group' && field.fields) {
        // For group fields, process nested fields
        field.fields.forEach(subField => processField(subField, data, source));
      } else if (source[field.name] !== undefined) {
        // For regular fields, copy value from entity
        data[field.name] = source[field.name];
      }
    }
    
    fields.forEach(field => processField(field, formData, entity));
    formErrors = {};
  }

  // Handle form submission
  async function handleSubmit(event: Event) {
    event.preventDefault();
    
    // Validate form
    const errors = validateForm(formData, validationRules);
    formErrors = errors;
    
    if (hasValidationErrors(errors)) {
      operationActions.setError('Please fix the validation errors above.');
      return;
    }

    // Perform save operation
    await withLoadingState(async () => {
      await onSave(formData);
      operationActions.setMessage(
        mode === 'create' ? 'Created successfully' : 'Updated successfully'
      );
      closeModal();
    }, operationActions, 'saving');
  }

  // Handle delete operation
  async function handleDelete() {
    if (!entity || !onDelete || !showDeleteConfirm) return;
    
    await withLoadingState(async () => {
      await onDelete(entity);
      operationActions.setMessage('Deleted successfully');
      closeModal();
    }, operationActions, 'deleting');
  }

  // Close modal and reset state
  function closeModal() {
    resetForm();
    dispatch('close');
  }

  // Reactive statements
  $: if (isOpen) {
    if (mode === 'edit' && entity) {
      loadEntityData();
    } else {
      resetForm();
    }
  }

  // Clear form when modal closes
  $: if (!isOpen) {
    resetForm();
  }
</script>

<BaseModal 
  {isOpen} 
  {title}
  {maxWidth}
  {customClass}
  on:close={closeModal}
>
  <!-- Form -->
  <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 16px;">
    
    <!-- Dynamic Fields -->
    <div style="display: flex; flex-direction: column; gap: 12px;">
      {#each fields as field}
        <FormField 
          {field}
          bind:formData
          error={formErrors[field.name]}
          on:fieldChange
        />
      {/each}
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
    {#if showDeleteConfirm && mode === 'edit' && onDelete}
      <div class="text-red-400 text-sm bg-red-900/20 border border-red-500/30 rounded p-3">
        <p class="font-medium mb-2">Are you sure you want to delete this item?</p>
        <p class="text-xs opacity-80">This action cannot be undone.</p>
      </div>
    {/if}
    
    <!-- Actions -->
    <div class="w-full" style="height: 40px;">
      {#if mode === 'edit' && onDelete && !showDeleteConfirm}
        <!-- Edit Mode: Delete button on left, Cancel/Update on right -->
        <div class="flex justify-between items-stretch h-full" style="gap: 12px;">
          <Button
            variant="ghost"
            size="sm"
            className="!bg-red-600 !text-white hover:!bg-red-700 !border !border-red-500 h-full !py-1 !flex !items-center !justify-center"
            on:click={() => showDeleteConfirm = true}
            disabled={$operationState.saving || $operationState.deleting}
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
              disabled={$operationState.saving || $operationState.deleting}
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
      {:else if mode === 'edit' && onDelete && showDeleteConfirm}
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
            disabled={$operationState.saving}
          >
            {#if $operationState.saving}
              <div 
                class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin"
                style="width: 14px; height: 14px; margin-right: 6px;"
              ></div>
            {/if}
            Create
          </Button>
        </div>
      {/if}
    </div>
  </form>
</BaseModal>