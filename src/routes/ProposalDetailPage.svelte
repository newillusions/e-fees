<script lang="ts">
  import GenericDetailPage from '$lib/components/GenericDetailPage.svelte';
  import { feesStore } from '$lib/stores';
  import ProposalDetail from '$lib/components/ProposalDetail.svelte';
  import ProposalModal from '$lib/components/ProposalModal.svelte';

  let { params = { id: '' } }: { params?: { id: string } } = $props();
</script>

<GenericDetailPage
  {params}
  entityType="Proposal"
  store={feesStore}
  backRoute="/proposals"
  DetailComponent={ProposalDetail}
  ModalComponent={ProposalModal}
  detailProps={(entity, { onedit, onclose }) => ({
    proposal: entity,
    isOpen: true,
    onedit,
    onclose
  })}
  modalProps={(entity, { onclose }) => ({ isOpen: true, proposal: entity, mode: 'edit', onclose })}
/>
