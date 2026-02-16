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

  // Generic form data type - uses unknown for type safety
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type EntityType = Record<string, any>;
  type FormDataType = Record<string, unknown>;
  // Callback type that accepts any entity - allows specific entity types from consumers
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  type DeleteCallback = (entity: any) => Promise<void>;

  const dispatch = createEventDispatcher();

  // Props - EntityType allows any entity object to be passed in
  let { isOpen = $bindable(false), entity = null as EntityType | null, mode = 'create' as 'create' | 'edit', title, fields, validationRules = [] as ValidationRule<FormDataType>[], onSave, onDelete = null as DeleteCallback | null, maxWidth = '500px', customClass = '', zIndex = 100 }: {
    isOpen?: boolean;
    entity?: EntityType | null;
    mode?: 'create' | 'edit';
    title: string;
    fields: FormFieldConfig[];
    validationRules?: ValidationRule<FormDataType>[];
    onSave: (data: FormDataType) => Promise<void>;
    onDelete?: DeleteCallback | null;
    maxWidth?: string;
    customClass?: string;
    zIndex?: number;
  } = $props();

  // Operation state management
  const { store: operationState, actions: operationActions } = useOperationState();

  // Form state - internal form data for editing
  let formData: FormDataType = $state({});
  let formErrors: Partial<Record<string, string>> = $state({});
  let showDeleteConfirm = $state(false);

  // Initialize form data structure based on field configurations
  function initializeFormData(): FormDataType {
    const initialData: FormDataType = {};

    function processField(field: FormFieldConfig, data: FormDataType) {
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

    function processField(field: FormFieldConfig, data: FormDataType, source: EntityType) {
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
    await withLoadingState(
      async () => {
        await onSave(formData);
        operationActions.setMessage(
          mode === 'create' ? 'Created successfully' : 'Updated successfully'
        );
        closeModal();
      },
      operationActions,
      'saving'
    );
  }

  // Handle delete operation
  async function handleDelete() {
    if (!entity || !onDelete || !showDeleteConfirm) return;

    await withLoadingState(
      async () => {
        await onDelete(entity);
        operationActions.setMessage('Deleted successfully');
        closeModal();
      },
      operationActions,
      'deleting'
    );
  }

  // Close modal and reset state
  function closeModal() {
    resetForm();
    dispatch('close');
  }

  // Reactive statements
  $effect(() => {
    if (isOpen) {
      if (mode === 'edit' && entity) {
        loadEntityData();
      } else {
        resetForm();
      }
    } else {
      resetForm();
    }
  });
</script>

<BaseModal {isOpen} {title} {maxWidth} {customClass} {zIndex} on:close={closeModal}>
  <!-- Form -->
  <form on:submit={handleSubmit} style="display: flex; flex-direction: column; gap: 12px;">
    <!-- Dynamic Fields -->
    <div style="display: flex; flex-direction: column; gap: 8px;">
      {#each fields as field}
        <FormField {field} bind:formData error={formErrors[field.name]} on:fieldChange />
      {/each}
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
    {#if showDeleteConfirm && mode === 'edit' && onDelete}
      <div class="emittiv-alert emittiv-alert--error">
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
            variant="danger"
            size="sm"
            className="h-full"
            on:click={() => (showDeleteConfirm = true)}
            disabled={$operationState.saving || $operationState.deleting}
          >
            Delete
          </Button>

          <div class="flex h-full" style="gap: 12px;">
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
            variant="danger"
            size="sm"
            className="h-full"
            on:click={handleDelete}
            disabled={$operationState.deleting}
          >
            {#if $operationState.deleting}
              <div class="emittiv-spinner-sm" style="border-color: white; border-top-color: transparent; margin-right: 6px;"></div>
            {/if}
            Confirm Delete
          </Button>
          <Button
            variant="secondary"
            size="sm"
            className=""
            on:click={() => (showDeleteConfirm = false)}
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
              <div
                class="emittiv-spinner-sm"
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
