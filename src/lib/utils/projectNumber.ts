/**
 * Project number parsing and payload building utilities.
 *
 * Extracted from NewProjectModal.handleCreate() to enable unit testing.
 * Project number format: YY-CCCNN (e.g., 25-97105 = year 2025, UAE 971, seq 05)
 */

import type { ProjectNumber } from '../../types';

export interface ProjectFormFields {
  name: string;
  name_short: string;
  status: string;
  area: string;
  city: string;
  country: string;
  folder: string;
}

export function parseProjectNumber(raw: string): ProjectNumber {
  const parts = raw.split('-');
  if (parts.length !== 2 || !parts[0] || !parts[1]) {
    throw new Error('Invalid project number format');
  }

  const year = parseInt(parts[0]);
  const countryCode = parts[1].substring(0, 3);
  const seq = parseInt(parts[1].substring(3));

  return {
    year,
    country: parseInt(countryCode),
    seq,
    id: raw,
  };
}

export function buildProjectPayload(
  formData: ProjectFormFields,
  projectNumber: ProjectNumber
): {
  name: string;
  name_short: string;
  status: string;
  area: string;
  city: string;
  country: string;
  number: ProjectNumber;
  folder: string;
  time: { created_at: string; updated_at: string };
} {
  const timestamp = new Date().toISOString();

  return {
    name: formData.name,
    name_short: formData.name_short,
    status: formData.status,
    area: formData.area,
    city: formData.city,
    country: formData.country,
    number: projectNumber,
    folder: formData.folder,
    time: {
      created_at: timestamp,
      updated_at: timestamp,
    },
  };
}
