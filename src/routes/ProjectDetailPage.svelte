<script lang="ts">
  import GenericDetailPage from '$lib/components/GenericDetailPage.svelte';
  import { projectsStore } from '$lib/stores';
  import ProjectDetail from '$lib/components/ProjectDetail.svelte';
  import ProjectModal from '$lib/components/ProjectModal.svelte';

  let { params = { id: '' } }: { params?: { id: string } } = $props();
</script>

<GenericDetailPage
  {params}
  entityType="Project"
  store={projectsStore}
  backRoute="/projects"
  DetailComponent={ProjectDetail}
  ModalComponent={ProjectModal}
  detailProps={(entity, { onedit, onclose }) => ({
    project: entity,
    isOpen: true,
    onedit,
    onclose
  })}
  modalProps={(entity, { onclose }) => ({ isOpen: true, project: entity, mode: 'edit', onclose })}
/>
