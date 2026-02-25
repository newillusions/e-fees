<!--
  Venue Modal using Generic CrudModal Component

  Handles create and edit operations for venue records.
  Flattens nested location fields for the generic form system.
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { venuesActions } from '$lib/stores';
  import { getEntityId } from '$lib/utils/surrealdb';
  import { CommonValidationRules } from '$lib/utils/validation';
  import { searchCountries } from '$lib/api';
  import CrudModal from './base/CrudModal.svelte';
  import type { Venue, VenueCreate } from '../../types';
  import type { FormFieldConfig } from './base/types';

  const dispatch = createEventDispatcher();

  let { isOpen = $bindable(false), venue = null, mode = 'create', zIndex = 100 }: {
    isOpen?: boolean;
    venue?: Venue | null;
    mode?: 'create' | 'edit';
    zIndex?: number;
  } = $props();

  // Transform venue entity for the form (flatten nested location fields)
  const formEntity = $derived(venue ? {
    ...venue,
    city: venue.location?.city || '',
    country: venue.location?.country || '',
    area: venue.location?.area || '',
    tags: (venue.tags || []).join(', '),
  } : null);

  // Form field configuration
  const fields: FormFieldConfig[] = [
    {
      type: 'group',
      name: 'venue_info',
      groupTitle: 'Venue Information',
      fields: [
        {
          type: 'text',
          name: 'name',
          label: 'Venue Name',
          placeholder: 'Full venue name',
          required: true,
          colSpan: 2
        },
        {
          type: 'text',
          name: 'name_short',
          label: 'Short Name',
          placeholder: 'Short name (optional)',
          required: false,
          colSpan: 1
        },
        {
          type: 'typeahead',
          name: 'country',
          label: 'Country',
          placeholder: 'Search countries...',
          required: false,
          colSpan: 1,
          displayFields: ['name'],
          onSearch: async (searchText: string) => {
            if (!searchText || searchText.length < 2) return [];
            try {
              const countries = await searchCountries(searchText);
              return countries.map(country => ({
                id: country.name,
                name: country.name,
                dial_code: country.dial_code
              }));
            } catch (error) {
              console.warn('Failed to search countries:', error);
              return [];
            }
          }
        },
        {
          type: 'text',
          name: 'city',
          label: 'City',
          placeholder: 'City (optional)',
          required: false,
          colSpan: 1
        },
        {
          type: 'text',
          name: 'area',
          label: 'Area / District',
          placeholder: 'Area or district (optional)',
          required: false,
          colSpan: 1
        },
        {
          type: 'text',
          name: 'tags',
          label: 'Tags',
          placeholder: 'Comma-separated tags (e.g. hotel, resort, mixed-use)',
          required: false,
          colSpan: 2
        },
        {
          type: 'textarea',
          name: 'notes',
          label: 'Notes',
          placeholder: 'Additional notes (optional)',
          required: false,
          colSpan: 2
        }
      ]
    }
  ];

  // Validation rules
  const validationRules = [
    { field: 'name', required: true, minLength: 2, maxLength: 200 },
    { field: 'name_short', required: false, maxLength: 50 },
    { field: 'country', required: false, maxLength: 100 },
    { field: 'city', required: false, maxLength: 100 },
    { field: 'area', required: false, maxLength: 100 },
    { field: 'tags', required: false, maxLength: 500 },
    { field: 'notes', required: false, maxLength: 2000 }
  ];

  // Save handler
  async function handleSave(formData: Record<string, unknown>) {
    const tagsRaw = (formData.tags as string || '').split(',').map((t: string) => t.trim()).filter(Boolean);

    if (mode === 'create') {
      const venueData = {
        name: formData.name as string,
        name_short: formData.name_short as string || '',
        location: {
          city: formData.city as string || '',
          country: formData.country as string || '',
          area: formData.area as string || '',
        },
        tags: tagsRaw,
        notes: formData.notes as string || '',
      };
      await venuesActions.create(venueData as unknown as Omit<Venue, 'id'>);
    } else if (venue) {
      const venueId = getEntityId(venue);
      if (!venueId) {
        throw new Error('Invalid venue ID');
      }

      const venueData = {
        name: formData.name as string,
        name_short: formData.name_short as string || '',
        location: {
          city: formData.city as string || '',
          country: formData.country as string || '',
          area: formData.area as string || '',
        },
        tags: tagsRaw,
        notes: formData.notes as string || '',
      };
      await venuesActions.update(venueId, venueData as Partial<Venue>);
    }
  }

  // Delete handler
  async function handleDelete(entity: Venue) {
    const venueId = getEntityId(entity);
    if (!venueId) {
      throw new Error('Invalid venue ID');
    }
    await venuesActions.delete(venueId);
  }

  // Close handler
  function handleClose() {
    dispatch('close');
  }
</script>

<CrudModal
  {isOpen}
  entity={formEntity}
  {mode}
  title={mode === 'create' ? 'New Venue' : 'Edit Venue'}
  {fields}
  {validationRules}
  onSave={handleSave}
  onDelete={mode === 'edit' ? handleDelete : null}
  maxWidth="500px"
  customClass="venue-modal"
  {zIndex}
  on:close={handleClose}
/>
