<!--
  Generic Form Field Component
  
  Renders different field types based on configuration.
  Handles validation, layout, and field-specific logic.
-->
<script lang="ts">
  import FormInput from '../FormInput.svelte';
  import FormSelect from '../FormSelect.svelte';
  import TypeaheadSelect from '../TypeaheadSelect.svelte';
  import type { FormFieldConfig, FieldChangeEvent, TypeaheadOption } from './types';
  import { logger } from '$lib/services/logger';

  let {
    field,
    formData = $bindable({}),
    error = '',
    onfieldchange
  }: {
    field: FormFieldConfig;
    formData?: Record<string, unknown>;
    error?: string;
    onfieldchange?: (event: FieldChangeEvent) => void;
  } = $props();

  // Get/set value from formData (cast to string for form elements)
  const value = $derived((formData[field.name] as string) || '');

  function setValue(newValue: unknown) {
    formData[field.name] = newValue;
    handleValueChange();
  }

  // Internal state for typeahead fields
  let searchText = $state('');
  let options: Array<{ id: string; name: string; [key: string]: unknown }> = $state([]);

  // Handle value changes
  function handleValueChange() {
    onfieldchange?.({
      fieldName: field.name,
      value,
      formData: {} // Will be filled by parent component
    });
  }

  // Handle typeahead search
  async function handleTypeaheadSearch(searchInput: string) {
    if (field.onSearch && searchInput.length >= 1) {
      try {
        options = await field.onSearch(searchInput);
      } catch (error) {
        logger.warn(`Failed to search for ${field.name}`);
        options = [];
      }
    } else {
      options = [];
    }
  }

  // Handle typeahead selection
  function handleTypeaheadSelect(data: {
    id: string;
    option: { id: string; [key: string]: unknown };
  }) {
    setValue(data.id);
    if (field.onSelect) {
      field.onSelect(data.option as TypeaheadOption);
    }
  }

  // Reactive statements
  $effect(() => {
    if (field.type === 'typeahead' && value && typeof value === 'string') {
      searchText = value;
    }
  });
</script>

{#if field.type === 'group'}
  <!-- Group of fields -->
  <div>
    {#if field.groupTitle}
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 8px;">
        {field.groupTitle}
      </h3>
    {/if}

    <div class="grid grid-cols-2" style="gap: 8px;">
      {#each field.fields || [] as groupField}
        <svelte:self field={groupField} bind:formData {error} {onfieldchange} />
      {/each}
    </div>
  </div>
{:else if field.type === 'computed'}
  <!-- Computed/read-only field -->
  <div>
    <label
      class="block font-medium text-emittiv-lighter"
      style="font-size: 12px; margin-bottom: 4px;"
    >
      {field.label}
    </label>
    <div
      class="w-full bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-light flex items-center"
      style="padding: 8px 12px; font-size: 12px; height: 32px; opacity: 0.6;"
    >
      {field.computeFn ? field.computeFn(formData) : value}
    </div>
  </div>
{:else if field.type === 'textarea'}
  <!-- Textarea field -->
  <div class={field.colSpan === 2 ? 'col-span-2' : ''}>
    <label
      class="block font-medium text-emittiv-lighter"
      style="font-size: 12px; margin-bottom: 4px;"
    >
      {field.label}{field.required ? ' *' : ''}
    </label>
    <textarea
      {value}
      placeholder={field.placeholder || ''}
      required={field.required}
      disabled={field.disabled}
      maxlength={field.maxlength}
      class="w-full bg-emittiv-darker border text-emittiv-white rounded resize-none {error
        ? 'border-red-500'
        : 'border-emittiv-dark focus:border-emittiv-splash'} {field.className || ''}"
      style="padding: 8px 12px; font-size: 12px; min-height: 80px; transition: border-color 0.3s cubic-bezier(0.4, 0, 0.2, 1);"
      on:input={e => setValue(e.currentTarget.value)}
    ></textarea>
    {#if error}
      <p class="text-red-400 text-xs mt-1">{error}</p>
    {/if}
  </div>
{:else if field.type === 'select'}
  <!-- Select dropdown -->
  <div class={field.colSpan === 2 ? 'col-span-2' : ''}>
    <FormSelect
      label={field.label}
      bind:value={formData[field.name] as string}
      options={field.options || []}
      placeholder={field.placeholder}
      required={field.required}
      disabled={field.disabled}
      {error}
    />
  </div>
{:else if field.type === 'typeahead'}
  <!-- Typeahead select -->
  <div class={field.colSpan === 2 ? 'col-span-2' : ''}>
    <TypeaheadSelect
      label={field.label}
      {value}
      {searchText}
      {options}
      displayFields={field.displayFields || ['name']}
      placeholder={field.placeholder || 'Search...'}
      required={field.required}
      disabled={field.disabled}
      {error}
      oninput={handleTypeaheadSearch}
      onselect={handleTypeaheadSelect}
    >
      <svelte:fragment slot="option" let:option>
        <div class="flex flex-col">
          <span class="font-medium">{option.name}</span>
          {#if option.name_short && option.name_short !== option.name}
            <span class="text-emittiv-light text-xs">{option.name_short}</span>
          {/if}
          {#if option.abbreviation}
            <span class="text-emittiv-splash text-xs">{option.abbreviation}</span>
          {/if}
        </div>
      </svelte:fragment>
    </TypeaheadSelect>
  </div>
{:else}
  <!-- Standard input fields (text, email, tel, etc.) -->
  <div class={field.colSpan === 2 ? 'col-span-2' : ''}>
    <FormInput
      label={field.label}
      type={field.type}
      bind:value={formData[field.name] as string}
      placeholder={field.placeholder}
      required={field.required}
      disabled={field.disabled}
      maxlength={field.maxlength}
      {error}
      className={field.className}
    />
  </div>
{/if}
