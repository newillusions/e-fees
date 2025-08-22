<!--
  Generic Form Field Component
  
  Renders different field types based on configuration.
  Handles validation, layout, and field-specific logic.
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import FormInput from '../FormInput.svelte';
  import FormSelect from '../FormSelect.svelte';
  import TypeaheadSelect from '../TypeaheadSelect.svelte';
  import type { FormFieldConfig, FieldChangeEvent } from './types';

  const dispatch = createEventDispatcher<{ fieldChange: FieldChangeEvent }>();

  export let field: FormFieldConfig;
  export let formData: any = {};
  export let error: string = '';

  // Get/set value from formData
  $: value = formData[field.name] || '';
  
  function setValue(newValue: any) {
    formData[field.name] = newValue;
    handleValueChange();
  }

  // Internal state for typeahead fields
  let searchText = '';
  let options: Array<{ id: string; name: string; [key: string]: any }> = [];

  // Handle value changes
  function handleValueChange() {
    dispatch('fieldChange', {
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
        console.warn(`Failed to search for ${field.name}:`, error);
        options = [];
      }
    } else {
      options = [];
    }
  }

  // Handle typeahead selection
  function handleTypeaheadSelect(event: CustomEvent) {
    setValue(event.detail.id);
    if (field.onSelect) {
      field.onSelect(event.detail);
    }
  }

  // Reactive statements
  $: if (field.type === 'typeahead' && value && typeof value === 'string') {
    searchText = value;
  }
</script>

{#if field.type === 'group'}
  <!-- Group of fields -->
  <div>
    {#if field.groupTitle}
      <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">
        {field.groupTitle}
      </h3>
    {/if}
    
    <div style="display: flex; flex-direction: column; gap: 12px;">
      <div class="grid grid-cols-2 gap-3">
        {#each field.fields || [] as groupField}
          <svelte:self 
            field={groupField} 
            bind:formData
            error={error}
            on:fieldChange
          />
        {/each}
      </div>
    </div>
  </div>

{:else if field.type === 'computed'}
  <!-- Computed/read-only field -->
  <div>
    <label class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
      {field.label}
    </label>
    <div class="w-full bg-emittiv-darker border border-emittiv-dark rounded text-emittiv-light flex items-center" 
         style="padding: 8px 12px; font-size: 12px; height: 32px; opacity: 0.6;">
      {field.computeFn ? field.computeFn(formData) : value}
    </div>
  </div>

{:else if field.type === 'textarea'}
  <!-- Textarea field -->
  <div class={field.colSpan === 2 ? 'col-span-2' : ''}>
    <label class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
      {field.label}{field.required ? ' *' : ''}
    </label>
    <textarea
      value={value}
      placeholder={field.placeholder || ''}
      required={field.required}
      disabled={field.disabled}
      maxlength={field.maxlength}
      class="w-full bg-emittiv-darker border text-emittiv-white rounded resize-none {error ? 'border-red-500' : 'border-emittiv-dark focus:border-emittiv-splash'} {field.className || ''}"
      style="padding: 8px 12px; font-size: 12px; min-height: 80px; transition: border-color 0.3s cubic-bezier(0.4, 0, 0.2, 1);"
      on:input={(e) => setValue(e.currentTarget.value)}
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
      value={value}
      options={field.options || []}
      placeholder={field.placeholder}
      required={field.required}
      disabled={field.disabled}
      error={error}
      on:change={(e) => setValue(e.detail)}
    />
  </div>

{:else if field.type === 'typeahead'}
  <!-- Typeahead select -->
  <div class={field.colSpan === 2 ? 'col-span-2' : ''}>
    <TypeaheadSelect
      label={field.label}
      value={value}
      searchText={searchText}
      options={options}
      displayFields={field.displayFields || ['name']}
      placeholder={field.placeholder || 'Search...'}
      required={field.required}
      disabled={field.disabled}
      error={error}
      on:input={(e) => handleTypeaheadSearch(e.detail)}
      on:select={handleTypeaheadSelect}
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
      value={value}
      placeholder={field.placeholder}
      required={field.required}
      disabled={field.disabled}
      maxlength={field.maxlength}
      error={error}
      className={field.className}
      on:input={(e) => setValue(e.detail)}
    />
  </div>
{/if}