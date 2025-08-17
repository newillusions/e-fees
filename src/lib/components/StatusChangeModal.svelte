<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BaseModal from './BaseModal.svelte';
  import Button from './Button.svelte';
  import { moveProjectFolder, getFolderForStatus, getStatusForFolder, type FolderOperationResult } from '$lib/api/folderManagement';
  import type { Project, Fee } from '../../types';
  
  const dispatch = createEventDispatcher();
  
  export let isOpen = false;
  export let project: Project | null = null;
  export let proposal: Fee | null = null;
  export let newStatus: string = '';
  export let relatedFees: Fee[] = [];
  export let mode: 'project-primary' | 'proposal-primary' = 'project-primary';
  
  let isProcessing = false;
  let operationResult: FolderOperationResult | null = null;
  let selectedFeeUpdates: Record<string, boolean> = {};
  let selectedProjectUpdate: boolean = false;
  let suggestedFeeStatus: string = '';
  let suggestedProjectStatus: string = '';
  
  // Determine primary and secondary entities based on mode
  $: primaryEntity = mode === 'project-primary' ? project : proposal;
  $: secondaryEntities = mode === 'project-primary' ? relatedFees : (project ? [project] : []);
  $: primaryType = mode === 'project-primary' ? 'project' : 'proposal';
  $: secondaryType = mode === 'project-primary' ? 'proposal' : 'project';
  
  $: currentFolder = project?.status ? getFolderForStatus(project.status) : '';
  $: newFolder = mode === 'project-primary' 
    ? (newStatus ? getFolderForStatus(newStatus) : '')
    : (suggestedProjectStatus ? getFolderForStatus(suggestedProjectStatus) : '');
  $: folderChangeRequired = currentFolder !== newFolder && currentFolder && newFolder;
  $: affectedFees = relatedFees.filter(fee => 
    // Filter fees that might be affected by status change
    fee.status !== 'Completed' && fee.status !== 'Cancelled'
  );
  
  // Determine suggested statuses based on primary entity change
  $: {
    if (mode === 'project-primary') {
      // Project is changing, suggest fee status updates
      if (newStatus === 'Active' || newStatus === 'Awarded') {
        suggestedFeeStatus = 'Awarded';
      } else if (newStatus === 'Completed') {
        suggestedFeeStatus = 'Completed';
      } else if (newStatus === 'Cancelled' || newStatus === 'Lost') {
        suggestedFeeStatus = 'Lost';
      } else if (newStatus === 'On Hold') {
        suggestedFeeStatus = 'On Hold';
      } else {
        suggestedFeeStatus = '';
      }
      
      // Auto-select relevant fees for update
      affectedFees.forEach(fee => {
        if (fee.id && suggestedFeeStatus && fee.status !== suggestedFeeStatus) {
          selectedFeeUpdates[fee.id] = true;
        }
      });
    } else {
      // Proposal is changing, suggest project status update
      if (newStatus === 'Awarded') {
        suggestedProjectStatus = 'Active';
      } else if (newStatus === 'Completed') {
        suggestedProjectStatus = 'Completed';
      } else if (newStatus === 'Lost') {
        suggestedProjectStatus = 'Lost';
      } else if (newStatus === 'Cancelled') {
        suggestedProjectStatus = 'Cancelled';
      } else if (newStatus === 'On Hold') {
        suggestedProjectStatus = 'On Hold';
      } else if (newStatus === 'Sent' || newStatus === 'Negotiation') {
        suggestedProjectStatus = 'RFP';
      } else {
        suggestedProjectStatus = '';
      }
      
      // Auto-select project for update if suggestion exists
      if (project && suggestedProjectStatus && project.status !== suggestedProjectStatus) {
        selectedProjectUpdate = true;
      }
    }
  }
  
  function handleCancel() {
    isOpen = false;
    operationResult = null;
    dispatch('cancel');
  }
  
  async function handleConfirm() {
    if (!primaryEntity || !newStatus) return;
    
    isProcessing = true;
    
    try {
      if (mode === 'project-primary') {
        // Collect fees that should be updated
        const feesToUpdate = affectedFees.filter(fee => 
          fee.id && selectedFeeUpdates[fee.id]
        ).map(fee => ({
          id: fee.id,
          newStatus: suggestedFeeStatus
        }));
        
        // Dispatch the status change event - parent will handle folder operations
        dispatch('confirm', {
          project,
          newStatus,
          folderChangeRequired,
          affectedFees,
          feesToUpdate,
          suggestedFeeStatus
        });
      } else {
        // Proposal-primary mode: collect project update if selected
        const projectToUpdate = selectedProjectUpdate && project ? [{
          id: project.id,
          newStatus: suggestedProjectStatus
        }] : [];
        
        // Dispatch the status change event - parent will handle folder operations
        dispatch('confirm', {
          proposal,
          newStatus,
          folderChangeRequired,
          affectedProject: project,
          projectToUpdate,
          suggestedProjectStatus
        });
      }
      
    } catch (error) {
      console.error('Status change failed:', error);
    } finally {
      isProcessing = false;
    }
  }
  
  function getStatusChangeDescription(): string {
    if (!project?.status || !newStatus) return '';
    
    const statusChanges = {
      'RFP->Active': 'Project awarded - moving to current projects',
      'RFP->Completed': 'Project completed directly from RFP stage',
      'RFP->Cancelled': 'RFP cancelled - moving to inactive',
      'Active->Completed': 'Project completed - moving to archive',
      'Active->Cancelled': 'Active project cancelled - moving to inactive',
      'Active->On Hold': 'Project temporarily on hold',
      'Completed->Active': 'Reopening completed project',
      'Cancelled->RFP': 'Reactivating cancelled project'
    };
    
    const key = `${project.status}->${newStatus}`;
    return statusChanges[key] || `Changing status from ${project.status} to ${newStatus}`;
  }
  
  function getImpactWarning(): string {
    if (affectedFees.length === 0) return '';
    
    const warnings = [];
    
    if (newStatus === 'Cancelled' || newStatus === 'Lost') {
      warnings.push(`${affectedFees.length} related proposal(s) may need status updates`);
    }
    
    if (newStatus === 'Completed') {
      warnings.push(`${affectedFees.length} proposal(s) should be marked as completed or invoiced`);
    }
    
    if (newStatus === 'On Hold') {
      warnings.push(`${affectedFees.length} proposal(s) may need to be marked as on hold`);
    }
    
    return warnings.join('. ');
  }
</script>

<BaseModal 
  {isOpen}
  title="Confirm Status Change"
  maxWidth="600px"
  showCloseButton={false}
  on:close={handleCancel}
>
  {#if primaryEntity}
    <div class="space-y-1.5">
      <!-- Primary Entity Information -->
      <div class="bg-emittiv-black/30 rounded px-2 py-1.5 border border-emittiv-dark/50">
        <h3 class="text-sm font-medium text-emittiv-white mb-0.5">
          {#if mode === 'project-primary'}
            {project?.number?.id} - {project?.name}
          {:else}
            {proposal?.number} - {proposal?.name}
          {/if}
        </h3>
        <div class="grid grid-cols-2 gap-2 text-xs">
          <div>
            <span class="text-emittiv-light">Current Status:</span>
            <span class="text-emittiv-white ml-2">{mode === 'project-primary' ? project?.status : proposal?.status}</span>
          </div>
          <div>
            <span class="text-emittiv-light">New Status:</span>
            <span class="text-emittiv-splash ml-2 font-medium">{newStatus}</span>
          </div>
        </div>
      </div>
      
      <!-- Status Change Description -->
      <div class="bg-blue-500/10 border border-blue-500/20 rounded px-2 py-1">
        <div class="flex items-start gap-1.5">
          <svg class="w-3 h-3 text-blue-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div>
            <h4 class="text-xs font-medium text-blue-400">Status Change</h4>
            <p class="text-xs text-emittiv-lighter">{getStatusChangeDescription()}</p>
          </div>
        </div>
      </div>
      
      <!-- Folder Movement Information -->
      {#if folderChangeRequired}
        <div class="bg-orange-500/10 border border-orange-500/20 rounded px-2 py-1">
          <div class="flex items-start gap-1.5">
            <svg class="w-3 h-3 text-orange-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2V7z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5a2 2 0 012-2h4a2 2 0 012 2" />
            </svg>
            <div class="flex-1">
              <h4 class="text-xs font-medium text-orange-400">Folder Movement</h4>
              <p class="text-xs text-emittiv-lighter mb-0.5">
                Project folder will be moved from <span class="font-mono text-emittiv-white">{currentFolder}</span> 
                to <span class="font-mono text-emittiv-white">{newFolder}</span>
              </p>
              {#if newFolder === '11 Current'}
                <p class="text-xs text-emittiv-light">
                  ℹ️ Awarded project templates will be copied automatically
                </p>
              {/if}
            </div>
          </div>
        </div>
      {/if}
      
      <!-- Downstream Impact -->
      {#if (mode === 'project-primary' && affectedFees.length > 0) || (mode === 'proposal-primary' && project)}
        <div class="bg-yellow-500/10 border border-yellow-500/20 rounded px-2 py-1">
          <div class="flex items-start gap-1.5">
            <svg class="w-3 h-3 text-yellow-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.081 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
            <div class="flex-1">
              <h4 class="text-xs font-medium text-yellow-400">Downstream Impact</h4>
              
              {#if mode === 'project-primary'}
                <!-- Project changing → suggest fee updates -->
                {#if suggestedFeeStatus}
                  <p class="text-xs text-emittiv-lighter mb-0.5">
                    Suggested action: Update proposals to <span class="font-medium text-emittiv-white">{suggestedFeeStatus}</span>
                  </p>
                {:else}
                  <p class="text-xs text-emittiv-lighter mb-0.5">{getImpactWarning()}</p>
                {/if}
                
                <div class="space-y-0.5">
                  <h5 class="text-xs font-medium text-emittiv-light uppercase tracking-wider">Select proposals to update:</h5>
                  {#each affectedFees as fee}
                    <label class="flex items-center gap-2 py-0.5 px-1 bg-emittiv-black/30 rounded text-xs cursor-pointer hover:bg-emittiv-black/40">
                      <input
                        type="checkbox"
                        bind:checked={selectedFeeUpdates[fee.id]}
                        disabled={!suggestedFeeStatus || fee.status === suggestedFeeStatus}
                        class="w-3 h-3 rounded border-emittiv-light bg-emittiv-darker text-emittiv-splash focus:ring-1 focus:ring-emittiv-splash"
                      />
                      <div class="flex-1 flex items-center justify-between">
                        <span class="text-emittiv-white">{fee.number}</span>
                        <div class="flex items-center gap-2">
                          <span class="text-emittiv-lighter">{fee.status}</span>
                          {#if suggestedFeeStatus && fee.status !== suggestedFeeStatus && selectedFeeUpdates[fee.id]}
                            <span class="text-emittiv-splash">→ {suggestedFeeStatus}</span>
                          {/if}
                        </div>
                      </div>
                    </label>
                  {/each}
                </div>
                
                {#if Object.values(selectedFeeUpdates).some(v => v)}
                  <p class="text-xs text-emittiv-splash mt-0.5">
                    ✓ Selected proposals will be automatically updated
                  </p>
                {/if}
              {:else}
                <!-- Proposal changing → suggest project update -->
                {#if suggestedProjectStatus}
                  <p class="text-xs text-emittiv-lighter mb-0.5">
                    Suggested action: Update project to <span class="font-medium text-emittiv-white">{suggestedProjectStatus}</span>
                  </p>
                {/if}
                
                <div class="space-y-0.5">
                  <h5 class="text-xs font-medium text-emittiv-light uppercase tracking-wider">Update related project:</h5>
                  <label class="flex items-center gap-2 py-0.5 px-1 bg-emittiv-black/30 rounded text-xs cursor-pointer hover:bg-emittiv-black/40">
                    <input
                      type="checkbox"
                      bind:checked={selectedProjectUpdate}
                      disabled={!suggestedProjectStatus || project?.status === suggestedProjectStatus}
                      class="w-3 h-3 rounded border-emittiv-light bg-emittiv-darker text-emittiv-splash focus:ring-1 focus:ring-emittiv-splash"
                    />
                    <div class="flex-1 flex items-center justify-between">
                      <span class="text-emittiv-white">{project?.number?.id} - {project?.name}</span>
                      <div class="flex items-center gap-2">
                        <span class="text-emittiv-lighter">{project?.status}</span>
                        {#if suggestedProjectStatus && project?.status !== suggestedProjectStatus && selectedProjectUpdate}
                          <span class="text-emittiv-splash">→ {suggestedProjectStatus}</span>
                        {/if}
                      </div>
                    </div>
                  </label>
                </div>
                
                {#if selectedProjectUpdate}
                  <p class="text-xs text-emittiv-splash mt-0.5">
                    ✓ Project will be automatically updated
                  </p>
                {/if}
              {/if}
            </div>
          </div>
        </div>
      {/if}
      
    </div>
  {/if}
  
  <!-- Actions -->
  <div class="flex justify-end gap-2 mt-3 pt-2 border-t border-emittiv-dark/50">
    <Button 
      variant="ghost" 
      size="sm"
      className="!py-1 !px-2.5 !text-xs"
      on:click={handleCancel}
      disabled={isProcessing}
    >
      Cancel
    </Button>
    <Button 
      variant="primary" 
      size="sm"
      className="!py-1 !px-3 !text-xs"
      on:click={handleConfirm}
      disabled={isProcessing || !project || !newStatus}
      loading={isProcessing}
    >
      {isProcessing ? 'Processing...' : 'Confirm Status Change'}
    </Button>
  </div>
</BaseModal>