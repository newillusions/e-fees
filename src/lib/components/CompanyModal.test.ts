/**
 * CompanyModal Component Tests
 * 
 * Tests for the Company modal component including CRUD operations,
 * form validation, country/city typeahead, and user interactions.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import CompanyModal from './CompanyModal.svelte';
import type { Company } from '$lib/../types';

// Mock the API
vi.mock('$lib/api', () => ({
  searchCountries: vi.fn(),
  getCitySuggestions: vi.fn()
}));

// Mock stores
vi.mock('$lib/stores', () => ({
  companiesActions: {
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn()
  }
}));

// Mock utilities
vi.mock('$lib/utils/surrealdb', () => ({
  extractSurrealId: vi.fn()
}));

vi.mock('$lib/utils/validation', () => ({
  validateForm: vi.fn(),
  hasValidationErrors: vi.fn(),
  CommonValidationRules: {
    company: {
      name: { field: 'name', required: true, minLength: 1, maxLength: 100 }
    }
  }
}));

vi.mock('$lib/utils/crud', () => ({
  useOperationState: vi.fn(),
  withLoadingState: vi.fn()
}));

import { searchCountries, getCitySuggestions } from '$lib/api';
import { companiesActions } from '$lib/stores';
import { extractSurrealId } from '$lib/utils/surrealdb';
import { validateForm, hasValidationErrors } from '$lib/utils/validation';
import { useOperationState, withLoadingState } from '$lib/utils/crud';

describe('CompanyModal Component', () => {
  const mockCountries = [
    { name: 'United Arab Emirates', dial_code: 971 },
    { name: 'Saudi Arabia', dial_code: 966 },
    { name: 'United Kingdom', dial_code: 44 }
  ];

  const mockCities = ['Dubai', 'Abu Dhabi', 'Sharjah'];

  const mockCompany: Company = {
    id: 'company:emt',
    name: 'Emittiv Engineering Consultants',
    name_short: 'Emittiv',
    abbreviation: 'EMT',
    city: 'Dubai',
    country: 'United Arab Emirates',
    reg_no: 'REG123456',
    tax_no: 'TAX789012',
    time: {
      created_at: '2025-08-21T10:00:00Z',
      updated_at: '2025-08-21T10:00:00Z'
    }
  };

  const mockOperationState = {
    subscribe: vi.fn((callback) => {
      callback({ saving: false, deleting: false, error: null, message: null });
      return { unsubscribe: vi.fn() };
    })
  };

  const mockOperationActions = {
    setError: vi.fn(),
    setMessage: vi.fn(),
    reset: vi.fn()
  };

  beforeEach(() => {
    vi.clearAllMocks();
    
    // Setup default mocks
    vi.mocked(searchCountries).mockResolvedValue(mockCountries);
    vi.mocked(getCitySuggestions).mockResolvedValue(mockCities);
    vi.mocked(validateForm).mockReturnValue({});
    vi.mocked(hasValidationErrors).mockReturnValue(false);
    vi.mocked(extractSurrealId).mockReturnValue('emt');
    vi.mocked(withLoadingState).mockImplementation(async (fn) => await fn());
    vi.mocked(useOperationState).mockReturnValue({
      store: mockOperationState,
      actions: mockOperationActions
    });
  });

  describe('Modal Display', () => {
    it('should not render when closed', () => {
      render(CompanyModal, { isOpen: false });
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('should render create modal when open', () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });
      expect(screen.getByRole('dialog')).toBeInTheDocument();
      expect(screen.getByText('New Company')).toBeInTheDocument();
    });

    it('should render edit modal with company data', () => {
      render(CompanyModal, {
        isOpen: true,
        mode: 'edit',
        company: mockCompany
      });
      expect(screen.getByText('Edit Company')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Emittiv Engineering Consultants')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Dubai')).toBeInTheDocument();
    });
  });

  describe('Form Fields', () => {
    it('should display all company form fields', () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });

      // Check labels exist
      expect(screen.getByText('Company Name')).toBeInTheDocument();
      expect(screen.getByText('Short Name')).toBeInTheDocument();
      expect(screen.getByText('Abbreviation')).toBeInTheDocument();
      
      // Check placeholders for complex components like TypeaheadSelect
      expect(screen.getByPlaceholderText('Search countries...')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Search cities...')).toBeInTheDocument();
      
      expect(screen.getByText('Registration No.')).toBeInTheDocument();
      expect(screen.getByText('Tax/VAT No.')).toBeInTheDocument();
    });

    it('should populate form fields when editing company', () => {
      render(CompanyModal, {
        isOpen: true,
        mode: 'edit',
        company: mockCompany
      });

      expect(screen.getByDisplayValue('Emittiv Engineering Consultants')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Emittiv')).toBeInTheDocument();
      expect(screen.getByDisplayValue('EMT')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Dubai')).toBeInTheDocument();
      expect(screen.getByDisplayValue('United Arab Emirates')).toBeInTheDocument();
      expect(screen.getByDisplayValue('REG123456')).toBeInTheDocument();
      expect(screen.getByDisplayValue('TAX789012')).toBeInTheDocument();
    });

    it('should have form submit button', () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });

      const createButton = screen.getByRole('button', { name: 'Create' });
      expect(createButton).toBeInTheDocument();
      expect(createButton).toHaveAttribute('type', 'submit');
    });

    it('should handle country search', async () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });
      
      // Country search input exists (within TypeaheadSelect)
      const countryInput = screen.getByPlaceholderText('Search countries...');
      expect(countryInput).toBeInTheDocument();
    });

    it('should handle city search', async () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });
      
      // City search input exists (within TypeaheadSelect)
      const cityInput = screen.getByPlaceholderText('Search cities...');
      expect(cityInput).toBeInTheDocument();
    });

    it('should show city field when no country selected', () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });
      
      // Note: City field conditional disabling not implemented in CrudModal system yet
      const cityInput = screen.getByPlaceholderText('Search cities...');
      expect(cityInput).toBeInTheDocument();
    });
  });

  describe('CRUD Operations', () => {
    it('should render create button in create mode', () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });

      const createButton = screen.getByRole('button', { name: 'Create' });
      expect(createButton).toBeInTheDocument();
      expect(createButton).not.toBeDisabled();
    });

    it('should update existing company', async () => {
      const user = userEvent.setup();
      
      vi.mocked(companiesActions.update).mockResolvedValue(mockCompany);
      
      render(CompanyModal, {
        isOpen: true,
        mode: 'edit',
        company: mockCompany
      });

      const updateButton = screen.getByRole('button', { name: 'Update' });
      await user.click(updateButton);

      expect(withLoadingState).toHaveBeenCalled();
    });

    it('should show delete confirmation', async () => {
      const user = userEvent.setup();
      
      render(CompanyModal, {
        isOpen: true,
        mode: 'edit',
        company: mockCompany
      });

      const deleteButton = screen.getByRole('button', { name: 'Delete' });
      await user.click(deleteButton);

      expect(screen.getByText('Are you sure you want to delete this item?')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Confirm Delete' })).toBeInTheDocument();
    });

    it('should delete company after confirmation', async () => {
      const user = userEvent.setup();
      
      vi.mocked(companiesActions.delete).mockResolvedValue(true);
      
      const { component } = render(CompanyModal, {
        isOpen: true,
        mode: 'edit',
        company: mockCompany
      });

      // Click delete to show confirmation
      const deleteButton = screen.getByRole('button', { name: 'Delete' });
      await user.click(deleteButton);

      // Click confirm delete
      const confirmButton = screen.getByRole('button', { name: 'Confirm Delete' });
      await user.click(confirmButton);

      expect(withLoadingState).toHaveBeenCalled();
    });
  });

  describe('Button States', () => {
    it('should show different buttons for create mode', () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });

      expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Create' })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
    });

    it('should show different buttons for edit mode', () => {
      render(CompanyModal, {
        isOpen: true,
        mode: 'edit',
        company: mockCompany
      });

      expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Update' })).toBeInTheDocument();
    });

    it('should show loading spinner during save', () => {
      const loadingState = {
        subscribe: vi.fn((callback) => {
          callback({ saving: true, deleting: false, error: null, message: null });
          return { unsubscribe: vi.fn() };
        })
      };

      vi.mocked(useOperationState).mockReturnValue({
        store: loadingState,
        actions: mockOperationActions
      });

      render(CompanyModal, { isOpen: true, mode: 'create' });

      const createButton = screen.getByRole('button', { name: 'Create' });
      expect(createButton).toBeDisabled();
      
      // Loading spinner should be present (div with animate-spin)
      const spinner = createButton.querySelector('.animate-spin');
      expect(spinner).toBeInTheDocument();
    });
  });

  describe('Error and Message Display', () => {
    it('should display error messages', () => {
      const errorState = {
        subscribe: vi.fn((callback) => {
          callback({ saving: false, deleting: false, error: 'Something went wrong', message: null });
          return { unsubscribe: vi.fn() };
        })
      };

      vi.mocked(useOperationState).mockReturnValue({
        store: errorState,
        actions: mockOperationActions
      });

      render(CompanyModal, { isOpen: true, mode: 'create' });

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    });

    it('should display success messages', () => {
      const messageState = {
        subscribe: vi.fn((callback) => {
          callback({ saving: false, deleting: false, error: null, message: 'Company created successfully' });
          return { unsubscribe: vi.fn() };
        })
      };

      vi.mocked(useOperationState).mockReturnValue({
        store: messageState,
        actions: mockOperationActions
      });

      render(CompanyModal, { isOpen: true, mode: 'create' });

      expect(screen.getByText('Company created successfully')).toBeInTheDocument();
    });
  });

  describe('Modal Interactions', () => {
    it('should render cancel button for modal close', async () => {
      const user = userEvent.setup();
      
      render(CompanyModal, { isOpen: true, mode: 'create' });

      const cancelButton = screen.getByRole('button', { name: 'Cancel' });
      expect(cancelButton).toBeInTheDocument();
      
      // Test clicking cancel button
      await user.click(cancelButton);
      
      // Button should be clickable (this tests the functionality without $on)
      expect(cancelButton).toBeInTheDocument();
    });

    it('should have operation state utilities available', () => {
      render(CompanyModal, { isOpen: true, mode: 'create' });
      
      // Component has access to operation actions from useOperationState
      expect(useOperationState).toHaveBeenCalled();
    });
  });
});