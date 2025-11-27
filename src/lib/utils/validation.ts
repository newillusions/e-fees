/**
 * Generic Form Validation System
 * 
 * This module provides a standardized, type-safe validation system that can be
 * used across all form components. It eliminates duplicate validation logic
 * and provides consistent error handling.
 */

/**
 * Validation rule definition for a form field.
 */
export interface ValidationRule<T> {
  /** The field name to validate */
  field: keyof T;
  /** Whether the field is required */
  required?: boolean;
  /** Minimum length for string fields */
  minLength?: number;
  /** Maximum length for string fields */
  maxLength?: number;
  /** Regular expression pattern to match */
  pattern?: RegExp;
  /** Custom validation function */
  validator?: (value: unknown, formData: T) => string | null;
  /** Custom error message */
  errorMessage?: string;
}

/**
 * Validation errors object mapping field names to error messages.
 */
export type ValidationErrors<T> = Partial<Record<keyof T, string>>;

/**
 * Validates form data against a set of validation rules.
 * 
 * @param formData - The form data to validate
 * @param rules - Array of validation rules to apply
 * @returns Object containing validation errors (empty if no errors)
 * 
 * @example
 * const rules: ValidationRule<Company>[] = [
 *   { field: 'name', required: true, minLength: 2 },
 *   { field: 'email', pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/ }
 * ];
 * 
 * const errors = validateForm(companyData, rules);
 * if (Object.keys(errors).length === 0) {
 *   // Form is valid
 * }
 */
export function validateForm<T extends Record<string, any>>(
  formData: T,
  rules: ValidationRule<T>[]
): ValidationErrors<T> {
  const errors: ValidationErrors<T> = {};

  for (const rule of rules) {
    const value = formData[rule.field];
    const error = validateField(value, rule, formData);
    
    if (error) {
      errors[rule.field] = error;
    }
  }

  return errors;
}

/**
 * Validates a single field against a validation rule.
 * 
 * @param value - The field value to validate
 * @param rule - The validation rule to apply
 * @param formData - The complete form data (for context in custom validators)
 * @returns Error message if validation fails, null if valid
 */
function validateField<T>(
  value: unknown,
  rule: ValidationRule<T>,
  formData: T
): string | null {
  // Check required fields
  if (rule.required && (value === null || value === undefined || value === '')) {
    return rule.errorMessage || `${String(rule.field)} is required`;
  }

  // Skip other validations if field is empty and not required
  if (!rule.required && (value === null || value === undefined || value === '')) {
    return null;
  }

  // Check minimum length
  if (rule.minLength && typeof value === 'string' && value.length < rule.minLength) {
    return rule.errorMessage || `${String(rule.field)} must be at least ${rule.minLength} characters`;
  }

  // Check maximum length
  if (rule.maxLength && typeof value === 'string' && value.length > rule.maxLength) {
    return rule.errorMessage || `${String(rule.field)} must be no more than ${rule.maxLength} characters`;
  }

  // Check pattern matching
  if (rule.pattern && typeof value === 'string' && !rule.pattern.test(value)) {
    return rule.errorMessage || `${String(rule.field)} has invalid format`;
  }

  // Run custom validator
  if (rule.validator) {
    const customError = rule.validator(value, formData);
    if (customError) {
      return customError;
    }
  }

  return null;
}

/**
 * Common validation patterns for reuse.
 */
export const ValidationPatterns = {
  email: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
  phone: /^[\+]?[1-9][\d]{0,15}$/,
  url: /^https?:\/\/.+/,
  alphanumeric: /^[a-zA-Z0-9]+$/,
  numeric: /^\d+$/,
  projectNumber: /^\d{2}-\d{3}\d{2}$/, // Format: YY-CCCNN
  dateYYMMDD: /^\d{6}$/, // Format: YYMMDD
} as const;

/**
 * Common validation rules for different entity types.
 */
export const CommonValidationRules = {
  company: {
    name: { field: 'name' as const, required: true, minLength: 2, maxLength: 100 },
    email: { field: 'email' as const, pattern: ValidationPatterns.email },
    phone: { field: 'phone' as const, pattern: ValidationPatterns.phone },
  },
  
  contact: {
    firstName: { field: 'first_name' as const, required: true, minLength: 1, maxLength: 50 },
    lastName: { field: 'last_name' as const, required: true, minLength: 1, maxLength: 50 },
    email: { field: 'email' as const, required: true, pattern: ValidationPatterns.email },
    phone: { field: 'phone' as const, pattern: ValidationPatterns.phone },
  },
  
  project: {
    name: { field: 'name' as const, required: true, minLength: 2, maxLength: 100 },
    nameShort: { field: 'name_short' as const, required: true, minLength: 1, maxLength: 50 },
    city: { field: 'city' as const, required: true, minLength: 1, maxLength: 50 },
    country: { field: 'country' as const, required: true, minLength: 1, maxLength: 50 },
  },
  
  fee: {
    name: { field: 'name' as const, required: true, minLength: 2, maxLength: 100 },
    number: { field: 'number' as const, required: true, minLength: 5, maxLength: 20 },
    issueDate: { field: 'issue_date' as const, required: true, pattern: ValidationPatterns.dateYYMMDD },
    projectId: { field: 'project_id' as const, required: true },
    companyId: { field: 'company_id' as const, required: true },
    contactId: { field: 'contact_id' as const, required: true },
  },
} as const;

/**
 * Helper function to check if a form has any validation errors.
 * 
 * @param errors - Validation errors object
 * @returns True if there are any errors
 */
export function hasValidationErrors<T>(errors: ValidationErrors<T>): boolean {
  return Object.keys(errors).length > 0;
}

/**
 * Helper function to get the first validation error message.
 * 
 * @param errors - Validation errors object
 * @returns First error message, or null if no errors
 */
export function getFirstValidationError<T>(errors: ValidationErrors<T>): string | null {
  const errorKeys = Object.keys(errors) as (keyof T)[];
  return errorKeys.length > 0 ? errors[errorKeys[0]] || null : null;
}

/**
 * Custom validator for checking if a project number is unique.
 * 
 * @param existingNumbers - Array of existing project numbers
 * @returns Validation function
 */
export function createUniqueProjectNumberValidator(existingNumbers: string[]) {
  return (value: string): string | null => {
    if (existingNumbers.includes(value)) {
      return 'This project number already exists';
    }
    return null;
  };
}

/**
 * Custom validator for checking if an email is unique.
 * 
 * @param existingEmails - Array of existing email addresses
 * @returns Validation function
 */
export function createUniqueEmailValidator(existingEmails: string[]) {
  return (value: string): string | null => {
    if (existingEmails.includes(value.toLowerCase())) {
      return 'This email address is already in use';
    }
    return null;
  };
}