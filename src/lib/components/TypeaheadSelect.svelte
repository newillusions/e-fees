<!--
  Typeahead Select Component - Reusable autocomplete dropdown
  Used for complex search and select interactions with add-new functionality
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  let { label = '', value = $bindable(''), searchText = $bindable(''), placeholder = 'Search...', options = [] as Array<{ id: string; [key: string]: unknown }>, displayFields = ['name'], required = false, error = '', showAddButton = false, addButtonLabel = 'Add new', maxHeight = '192px', disabled = false }: {
    label?: string;
    value?: string;
    searchText?: string;
    placeholder?: string;
    options?: Array<{ id: string; [key: string]: unknown }>;
    displayFields?: string[];
    required?: boolean;
    error?: string;
    showAddButton?: boolean;
    addButtonLabel?: string;
    maxHeight?: string;
    disabled?: boolean;
  } = $props();
  
  let dropdownOpen = $state(false);
  let inputElement: HTMLInputElement;
  let selectedIndex = $state(-1); // Track which option is highlighted
  
  // Clear the selection
  function clearSelection() {
    searchText = '';
    value = '';
    dropdownOpen = false;
    selectedIndex = -1;
    dispatch('clear');
  }
  
  // Select an option
  function selectOption(optionId: string) {
    const selectedOption = options.find(opt => opt.id === optionId);
    if (selectedOption) {
      value = optionId;
      searchText = displayFields.map(field => selectedOption[field]).filter(Boolean).join(' - ');
      dropdownOpen = false;
      selectedIndex = -1;
      dispatch('select', { id: optionId, option: selectedOption });
    }
  }
  
  // Handle input events
  function handleInput() {
    dropdownOpen = true;
    selectedIndex = -1; // Reset selection when typing
    
    // Check if the typed text exactly matches any option
    const exactMatch = options.find(opt => 
      displayFields.map(field => opt[field]).filter(Boolean).join(' - ').toLowerCase() === searchText.toLowerCase()
    );
    
    if (exactMatch) {
      // Auto-select exact matches
      selectOption(exactMatch.id);
    } else if (searchText && !options.some(opt => displayFields.map(field => opt[field]).filter(Boolean).join(' - ') === searchText)) {
      value = '';
    }
    
    dispatch('input', searchText);
  }
  
  function handleFocus() {
    dropdownOpen = true;
    selectedIndex = -1; // Reset selection when focusing
    dispatch('focus');
  }
  
  function handleBlur() {
    // Delay closing to allow click events on options
    setTimeout(() => {
      dropdownOpen = false;
      dispatch('blur');
    }, 200);
  }
  
  function handleAddNew() {
    dispatch('add-new');
  }
  
  // Handle keyboard navigation
  function handleKeyDown(event: KeyboardEvent) {
    switch (event.key) {
      case 'Tab':
        // If dropdown is open and we have options, select the highlighted one or first one
        if (dropdownOpen && options.length > 0) {
          event.preventDefault();
          const indexToSelect = selectedIndex >= 0 ? selectedIndex : 0;
          selectOption(options[indexToSelect].id);
        }
        break;
      case 'ArrowDown':
        if (!dropdownOpen || options.length === 0) return;
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, options.length - 1);
        break;
      case 'ArrowUp':
        if (!dropdownOpen || options.length === 0) return;
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, -1);
        break;
      case 'Enter':
      case ' ': // Space key
        if (!dropdownOpen || options.length === 0) return;
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < options.length) {
          selectOption(options[selectedIndex].id);
        }
        break;
      case 'Escape':
        if (!dropdownOpen) return;
        event.preventDefault();
        dropdownOpen = false;
        selectedIndex = -1;
        break;
    }
  }
  
  // Update search text when an option is pre-selected
  $effect(() => {
    if (value && !searchText) {
      const selectedOption = options.find(opt => opt.id === value);
      if (selectedOption) {
        searchText = displayFields.map(field => selectedOption[field]).filter(Boolean).join(' - ');
      }
    }
  });

  // Reset selected index when options change
  $effect(() => {
    options; // track dependency
    selectedIndex = -1;
  });
</script>

<div class="typeahead-select">
  <!-- Label -->
  {#if label}
    <label
      for="typeahead-{label.replace(/\s+/g, '-').toLowerCase()}"
      class="emittiv-label"
      class:emittiv-label--required={required}
    >
      {label}
    </label>
  {/if}
  
  <!-- Input Container -->
  <div class="flex relative" style="gap: 8px;">
    <div class="flex-1 relative">
      <!-- Search Input -->
      <input
        bind:this={inputElement}
        id="typeahead-{label.replace(/\s+/g, '-').toLowerCase()}"
        type="text"
        bind:value={searchText}
        on:input={handleInput}
        on:focus={handleFocus}
        on:blur={handleBlur}
        on:keydown={handleKeyDown}
        {placeholder}
        {disabled}
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        class="emittiv-input {error ? 'emittiv-input--error' : ''}"
        style="padding-right: {searchText ? '30px' : '12px'};"
      />
      
      <!-- Clear Button -->
      {#if searchText && !disabled}
        <button
          type="button"
          on:click={clearSelection}
          class="absolute right-1 top-1/2 transform -translate-y-1/2 text-emittiv-light hover:text-emittiv-white flex items-center justify-center"
          style="width: 16px; height: 16px;"
          aria-label="Clear selection"
        >
          <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}
      
      <!-- Dropdown Options -->
      {#if dropdownOpen && options.length > 0 && !disabled}
        <div 
          class="absolute top-full left-0 right-0 bg-emittiv-darker border border-emittiv-dark rounded-b overflow-y-auto z-50" 
          style="margin-top: 1px; max-height: {maxHeight};"
        >
          {#each options as option, index}
            <button
              type="button"
              on:click={() => selectOption(option.id)}
              on:mouseenter={() => selectedIndex = index}
              class="w-full text-left text-emittiv-white hover:bg-emittiv-dark text-xs border-b border-emittiv-dark last:border-b-0 truncate {selectedIndex === index ? 'bg-emittiv-dark' : ''}"
              style="height: 24px; line-height: 22px; padding: 1px 8px;"
            >
              <slot name="option" {option}>
                {displayFields.map(field => option[field]).filter(Boolean).join(' - ')}
              </slot>
            </button>
          {/each}
        </div>
      {/if}
    </div>
    
    <!-- Add New Button -->
    {#if showAddButton && !disabled}
      <button
        type="button"
        on:click={handleAddNew}
        class="emittiv-fab-sm"
        aria-label={addButtonLabel}
        title={addButtonLabel}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    {/if}
  </div>
  
  <!-- Error Message -->
  {#if error}
    <div class="emittiv-error">{error}</div>
  {/if}
</div>

<style>
  .typeahead-select {
    position: relative;
  }
</style>