/**
 * Type definitions for the generic CRUD modal system.
 */

import type { ValidationRule } from '$lib/utils/validation';

/** Type for typeahead search result items */
export interface TypeaheadOption {
  id: string;
  name: string;
  [key: string]: unknown;
}

/** Type for form field values */
export type FormFieldValue = string | number | boolean | null | undefined;

/**
 * Configuration for a form field in the CRUD modal.
 */
export interface FormFieldConfig {
  /** Field name/key in the form data object */
  name: string;
  /** Display label for the field */
  label?: string;
  /** Field type (determines which input component to use) */
  type: 'text' | 'textarea' | 'email' | 'tel' | 'select' | 'typeahead' | 'computed' | 'group';
  /** Placeholder text */
  placeholder?: string;
  /** Whether the field is required */
  required?: boolean;
  /** Maximum length for text inputs */
  maxlength?: number;
  /** Default value for the field */
  defaultValue?: FormFieldValue;
  /** Whether the field is disabled */
  disabled?: boolean;
  /** Additional CSS classes */
  className?: string;
  /** Grid column span (for responsive layouts) */
  colSpan?: 1 | 2;

  // Select field options
  /** Options for select fields (value/label for FormSelect) */
  options?: Array<{ value: string; label: string }>;

  // Typeahead field options
  /** Fields to display in typeahead options */
  displayFields?: string[];
  /** Search handler for typeahead fields */
  onSearch?: (searchText: string) => Promise<TypeaheadOption[]>;
  /** Selection handler for typeahead fields */
  onSelect?: (selected: TypeaheadOption) => void;

  // Computed field options
  /** Computation function for computed fields */
  computeFn?: (formData: Record<string, unknown>) => string;

  // Group field options
  /** Child fields for group fields */
  fields?: FormFieldConfig[];
  /** Group title */
  groupTitle?: string;
}

/**
 * Props interface for the CrudModal component.
 */
export interface CrudModalProps<T> {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Entity being edited (null for create mode) */
  entity?: T | null;
  /** Modal mode */
  mode: 'create' | 'edit';
  /** Modal title */
  title: string;
  /** Field configurations */
  fields: FormFieldConfig[];
  /** Validation rules */
  validationRules?: ValidationRule<T>[];
  /** Save handler */
  onSave: (data: T) => Promise<void>;
  /** Delete handler (optional) */
  onDelete?: (entity: T) => Promise<void>;
  /** Maximum modal width */
  maxWidth?: string;
  /** Custom CSS class */
  customClass?: string;
}

/**
 * Field change event data.
 */
export interface FieldChangeEvent {
  fieldName: string;
  value: unknown;
  formData: Record<string, unknown>;
}
