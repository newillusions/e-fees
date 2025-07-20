<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { projectsActions } from '$lib/stores';
  import type { Project } from '$lib/../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let project: Project | null = null; // null for create, project object for edit
  export let mode: 'create' | 'edit' = 'create';
  
  // Form data
  let formData = {
    name: '',
    name_short: '',
    area: '',
    city: '',
    country: '',
    folder: '',
    status: 'Draft' as 'Draft' | 'RFP' | 'Active' | 'On Hold' | 'Completed' | 'Cancelled'
  };
  
  // Loading and error states
  let isSaving = false;
  let isDeleting = false;
  let saveMessage = '';
  let formErrors: Record<string, string> = {};
  let showDeleteConfirm = false;
  
  // Update form when project prop changes
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
  } else if (mode === 'create') {
    // Reset form for create mode
    formData = {
      name: '',
      name_short: '',
      area: '',
      city: '',
      country: '',
      folder: '',
      status: 'Draft'
    };
  }
  
  // Reset state when modal closes
  $: if (!isOpen) {
    resetForm();
  }
  
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
    saveMessage = '';
    isSaving = false;
    isDeleting = false;
    showDeleteConfirm = false;
  }
  
  function validateForm(): boolean {
    formErrors = {};
    
    // Required fields
    if (!formData.name.trim()) {
      formErrors.name = 'Project name is required';
    }
    
    if (!formData.name_short.trim()) {
      formErrors.name_short = 'Short name is required';
    }
    
    if (!formData.area.trim()) {
      formErrors.area = 'Area is required';
    }
    
    if (!formData.city.trim()) {
      formErrors.city = 'City is required';
    }
    
    if (!formData.country.trim()) {
      formErrors.country = 'Country is required';
    }
    
    if (!formData.folder.trim()) {
      formErrors.folder = 'Folder name is required';
    }
    
    return Object.keys(formErrors).length === 0;
  }
  
  async function handleSubmit() {
    if (!validateForm()) {
      return;
    }
    
    isSaving = true;
    saveMessage = '';
    
    try {
      if (mode === 'create') {
        const projectData = {
          ...formData,
          time: {
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString()
          }
        };
        await projectsActions.create(projectData);
        saveMessage = 'Project created successfully!';
      } else {
        const projectId = getProjectId(project);
        if (projectId) {
          await projectsActions.update(projectId, formData);
          saveMessage = 'Project updated successfully!';
        } else {
          throw new Error('No valid project ID found for update');
        }
      }
      
      // Auto-close after 1.5 seconds
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      saveMessage = `Error: ${error?.message || error}`;
    } finally {
      isSaving = false;
    }
  }
  
  async function handleDelete() {
    const projectId = getProjectId(project);
    if (!projectId) return;
    
    isDeleting = true;
    saveMessage = '';
    
    try {
      await projectsActions.delete(projectId);
      saveMessage = 'Project deleted successfully!';
      
      // Auto-close after 1.5 seconds
      setTimeout(() => {
        closeModal();
      }, 1500);
      
    } catch (error: any) {
      saveMessage = `Error: ${error?.message || error}`;
    } finally {
      isDeleting = false;
      showDeleteConfirm = false;
    }
  }
  
  function confirmDelete() {
    showDeleteConfirm = true;
  }
  
  function cancelDelete() {
    showDeleteConfirm = false;
  }
  
  function closeModal() {
    isOpen = false;
    resetForm();
    dispatch('close');
  }
  
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }
  
  // Auto-generate short name from project name
  function generateShortName() {
    if (formData.name && !formData.name_short) {
      const words = formData.name.trim().split(/\s+/);
      if (words.length >= 2) {
        formData.name_short = words.slice(0, 2).join(' ');
      } else {
        formData.name_short = formData.name.substring(0, 30);
      }
    }
  }
  
  // Auto-generate folder name from project name
  function generateFolderName() {
    if (formData.name && !formData.folder) {
      // Remove special characters and replace spaces with underscores
      formData.folder = formData.name
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, '')
        .replace(/\s+/g, '_')
        .substring(0, 50);
    }
  }
  
  // Helper function to extract ID from SurrealDB Thing object
  function getProjectId(project: Project | null): string | null {
    if (!project?.id) return null;
    
    if (typeof project.id === 'string') {
      return project.id;
    }
    
    // Handle SurrealDB Thing object format
    if (project.id && typeof project.id === 'object') {
      const thingObj = project.id as any;
      if (thingObj.tb && thingObj.id) {
        if (typeof thingObj.id === 'string') {
          return thingObj.id;
        } else if (thingObj.id.String) {
          return thingObj.id.String;
        }
      }
    }
    
    return null;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Modal Backdrop -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
    on:click={closeModal}
    on:keydown={(e) => e.key === 'Escape' && closeModal()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="project-modal-title"
    tabindex="-1"
  >
    <!-- Modal Content -->
    <div 
      class="bg-emittiv-darker border border-emittiv-dark rounded w-full max-h-[90vh] overflow-y-auto"
      style="padding: 24px; max-width: 700px;"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="presentation"
    >
      <!-- Header -->
      <div class="flex items-center justify-between" style="margin-bottom: 20px;">
        <h2 id="project-modal-title" class="font-semibold text-emittiv-white" style="font-size: 16px;">
          {mode === 'create' ? 'Create New Project' : 'Edit Project'}
        </h2>
        <button 
          on:click={closeModal}
          class="p-1 rounded-lg text-emittiv-light hover:text-emittiv-white hover:bg-emittiv-dark transition-smooth"
          aria-label="Close modal"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <form on:submit|preventDefault={handleSubmit} style="display: flex; flex-direction: column; gap: 24px;">
        
        <!-- Project Basic Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Project Information</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Project Name -->
            <div>
              <label for="project_name" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Project Name *
              </label>
              <input
                id="project_name"
                type="text"
                bind:value={formData.name}
                on:blur={generateShortName}
                on:blur={generateFolderName}
                placeholder="Museum of Future"
                required
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.name ? 'border-red-500' : ''}"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
              {#if formErrors.name}
                <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.name}</p>
              {/if}
            </div>
            
            <!-- Short Name and Status Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="project_short" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Short Name *
                </label>
                <input
                  id="project_short"
                  type="text"
                  bind:value={formData.name_short}
                  placeholder="Museum Future"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.name_short ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.name_short}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.name_short}</p>
                {/if}
              </div>
              
              <div>
                <label for="project_status" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Status *
                </label>
                <select
                  id="project_status"
                  bind:value={formData.status}
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all appearance-none"
                  style="padding: 8px 12px; font-size: 12px; height: 32px; padding-right: 32px; background-image: url('data:image/svg+xml,%3csvg xmlns=\'http://www.w3.org/2000/svg\' fill=\'none\' viewBox=\'0 0 20 20\'%3e%3cpath stroke=\'%23999\' stroke-linecap=\'round\' stroke-linejoin=\'round\' stroke-width=\'1.5\' d=\'M6 8l4 4 4-4\'/%3e%3c/svg%3e'); background-position: right 0.5rem center; background-repeat: no-repeat; background-size: 16px 12px;"
                >
                  <option value="Draft">Draft</option>
                  <option value="RFP">RFP</option>
                  <option value="Active">Active</option>
                  <option value="On Hold">On Hold</option>
                  <option value="Completed">Completed</option>
                  <option value="Cancelled">Cancelled</option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- Location Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Location</h3>
          <div style="display: flex; flex-direction: column; gap: 12px;">
            <!-- Area -->
            <div>
              <label for="project_area" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                Area *
              </label>
              <input
                id="project_area"
                type="text"
                bind:value={formData.area}
                placeholder="Downtown"
                required
                class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.area ? 'border-red-500' : ''}"
                style="padding: 8px 12px; font-size: 12px; height: 32px;"
              />
              {#if formErrors.area}
                <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.area}</p>
              {/if}
            </div>
            
            <!-- City and Country Row -->
            <div class="grid grid-cols-2" style="gap: 12px;">
              <div>
                <label for="project_city" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  City *
                </label>
                <input
                  id="project_city"
                  type="text"
                  bind:value={formData.city}
                  placeholder="Dubai"
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.city ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.city}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.city}</p>
                {/if}
              </div>
              
              <div>
                <label for="project_country" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
                  Country *
                </label>
                <input
                  id="project_country"
                  type="text"
                  bind:value={formData.country}
                  placeholder="U.A.E."
                  required
                  class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.country ? 'border-red-500' : ''}"
                  style="padding: 8px 12px; font-size: 12px; height: 32px;"
                />
                {#if formErrors.country}
                  <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.country}</p>
                {/if}
              </div>
            </div>
          </div>
        </div>

        <!-- Folder Information Section -->
        <div>
          <h3 class="font-medium text-emittiv-white" style="font-size: 14px; margin-bottom: 12px;">Project Folder</h3>
          <div>
            <label for="project_folder" class="block font-medium text-emittiv-lighter" style="font-size: 12px; margin-bottom: 4px;">
              Folder Name *
            </label>
            <input
              id="project_folder"
              type="text"
              bind:value={formData.folder}
              placeholder="museum_of_future"
              required
              class="w-full bg-emittiv-dark border border-emittiv-dark rounded text-emittiv-white placeholder-emittiv-light focus:outline-none focus:border-emittiv-splash focus:ring-1 focus:ring-emittiv-splash transition-all {formErrors.folder ? 'border-red-500' : ''}"
              style="padding: 8px 12px; font-size: 12px; height: 32px;"
            />
            {#if formErrors.folder}
              <p class="text-red-400" style="font-size: 10px; margin-top: 2px;">{formErrors.folder}</p>
            {/if}
            <p class="text-emittiv-light" style="font-size: 10px; margin-top: 4px;">This will be the folder name in your project directory</p>
          </div>
        </div>

        <!-- Save Message -->
        {#if saveMessage}
          <div class="rounded-lg {saveMessage.startsWith('Error') ? 'bg-red-900/20 border border-red-500/30 text-red-300' : 'bg-green-900/20 border border-green-500/30 text-green-300'}" style="padding: 8px; font-size: 11px;">
            {saveMessage}
          </div>
        {/if}

        <!-- Delete Confirmation -->
        {#if showDeleteConfirm}
          <div class="rounded-lg bg-red-900/20 border border-red-500/30 text-red-300" style="padding: 12px; font-size: 12px;">
            <p style="margin-bottom: 8px;">⚠️ Are you sure you want to delete this project?</p>
            <p class="text-red-400" style="font-size: 10px; margin-bottom: 12px;">This action cannot be undone. All related proposals will be affected.</p>
            <div class="flex" style="gap: 8px;">
              <button
                type="button"
                on:click={cancelDelete}
                class="border border-red-500/30 rounded text-red-300 hover:text-red-200 hover:border-red-400 transition-all"
                style="padding: 4px 8px; font-size: 11px; height: 24px;"
                disabled={isDeleting}
              >
                Cancel
              </button>
              <button
                type="button"
                on:click={handleDelete}
                class="bg-red-600 hover:bg-red-700 text-white rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                style="padding: 4px 8px; font-size: 11px; height: 24px; gap: 4px;"
                disabled={isDeleting}
              >
                {#if isDeleting}
                  <div class="border-2 border-white border-t-transparent rounded-full animate-spin" style="width: 10px; height: 10px;"></div>
                  <span>Deleting...</span>
                {:else}
                  <span>Delete Project</span>
                {/if}
              </button>
            </div>
          </div>
        {/if}

        <!-- Action Buttons -->
        <div class="flex {mode === 'edit' ? 'justify-between' : 'justify-end'} border-t border-emittiv-dark" style="gap: 12px; padding-top: 16px; margin-top: 8px;">
          {#if mode === 'edit'}
            <button
              type="button"
              on:click={confirmDelete}
              class="border border-red-500/50 rounded text-red-400 hover:text-red-300 hover:border-red-400 transition-all"
              style="padding: 6px 12px; font-size: 12px; height: 28px;"
              disabled={isSaving || isDeleting || showDeleteConfirm}
            >
              Delete
            </button>
          {/if}
          
          <div class="flex" style="gap: 12px;">
            <button
              type="button"
              on:click={closeModal}
              class="border border-emittiv-dark rounded text-emittiv-light hover:text-emittiv-white hover:border-emittiv-light transition-all"
              style="padding: 6px 12px; font-size: 12px; height: 28px;"
              disabled={isSaving || isDeleting}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="bg-emittiv-splash hover:bg-orange-600 text-emittiv-black rounded transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
              style="padding: 6px 12px; font-size: 12px; height: 28px; gap: 4px;"
              disabled={isSaving || isDeleting || showDeleteConfirm}
            >
              {#if isSaving}
                <div class="border-2 border-emittiv-black border-t-transparent rounded-full animate-spin" style="width: 12px; height: 12px;"></div>
                <span>Saving...</span>
              {:else}
                <span>{mode === 'create' ? 'Create Project' : 'Update Project'}</span>
              {/if}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}