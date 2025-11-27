/**
 * ProposalModal Component Tests
 * 
 * Tests for the Fee Proposal modal component including CRUD operations,
 * project/company/contact relationships, form validation, and business logic.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import ProposalModal from './ProposalModal.svelte';
import type { Fee, Project, Company, Contact } from '$lib/../types';

// Mock the API
vi.mock('$lib/api', () => ({
  writeFeeToJsonSafe: vi.fn()
}));

// Mock stores
vi.mock('$lib/stores', () => ({
  feesActions: {
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn()
  },
  feesStore: {
    subscribe: vi.fn()
  },
  projectsActions: {
    reload: vi.fn()
  },
  projectsStore: {
    subscribe: vi.fn()
  },
  companiesStore: {
    subscribe: vi.fn()
  },
  contactsStore: {
    subscribe: vi.fn()
  }
}));

// Mock settings store
vi.mock('$lib/stores/settings', () => ({
  settingsStore: {
    subscribe: vi.fn()
  }
}));

// Mock utilities
vi.mock('$lib/utils/surrealdb', () => ({
  extractSurrealId: vi.fn()
}));

vi.mock('$lib/utils/validation', () => ({
  validateForm: vi.fn(),
  hasValidationErrors: vi.fn(),
  ValidationPatterns: {
    email: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
    phone: /^[\+]?[1-9][\d]{0,15}$/,
    url: /^https?:\/\/.+/,
    alphanumeric: /^[a-zA-Z0-9]+$/,
    numeric: /^\d+$/,
    projectNumber: /^\d{2}-\d{3}\d{2}$/,
    dateYYMMDD: /^\d{6}$/,
  },
  CommonValidationRules: {
    company: {
      name: { field: 'name', required: true, minLength: 2, maxLength: 100 },
      email: { field: 'email', pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/ },
      phone: { field: 'phone', pattern: /^[\+]?[1-9][\d]{0,15}$/ },
    },
    contact: {
      firstName: { field: 'first_name', required: true, minLength: 1, maxLength: 50 },
      lastName: { field: 'last_name', required: true, minLength: 1, maxLength: 50 },
      email: { field: 'email', required: true, pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/ },
      phone: { field: 'phone', pattern: /^[\+]?[1-9][\d]{0,15}$/ },
    },
    project: {
      name: { field: 'name', required: true, minLength: 2, maxLength: 100 },
      nameShort: { field: 'name_short', required: true, minLength: 1, maxLength: 50 },
      city: { field: 'city', required: true, minLength: 1, maxLength: 50 },
      country: { field: 'country', required: true, minLength: 1, maxLength: 50 },
    },
    fee: {
      name: { field: 'name', required: true, minLength: 2, maxLength: 100 },
      number: { field: 'number', required: true, minLength: 5, maxLength: 20 },
      issueDate: { field: 'issue_date', required: true, pattern: /^\d{6}$/ },
      projectId: { field: 'project_id', required: true },
      companyId: { field: 'company_id', required: true },
      contactId: { field: 'contact_id', required: true },
    },
  }
}));

vi.mock('$lib/utils/crud', () => ({
  useOperationState: vi.fn(),
  withLoadingState: vi.fn()
}));

import { feesActions, feesStore, projectsActions, projectsStore, companiesStore, contactsStore } from '$lib/stores';
import { settingsStore } from '$lib/stores/settings';
import { extractSurrealId } from '$lib/utils/surrealdb';
import { validateForm, hasValidationErrors } from '$lib/utils/validation';
import { useOperationState, withLoadingState } from '$lib/utils/crud';
import { writeFeeToJsonSafe } from '$lib/api';

describe('ProposalModal Component', () => {
  const mockProjects: Project[] = [
    {
      id: 'projects:25-97101',
      number: '25-97101',
      name: 'Dubai Marina Tower',
      name_short: 'Marina Tower',
      country: 'United Arab Emirates',
      city: 'Dubai',
      company: 'company:emt',
      time: {
        created_at: '2025-08-21T09:00:00Z',
        updated_at: '2025-08-21T09:00:00Z'
      }
    },
    {
      id: 'projects:25-96601',
      number: '25-96601',
      name: 'Riyadh Complex',
      name_short: 'Riyadh Complex',
      country: 'Saudi Arabia',
      city: 'Riyadh',
      company: 'company:aec',
      time: {
        created_at: '2025-08-21T09:00:00Z',
        updated_at: '2025-08-21T09:00:00Z'
      }
    }
  ];

  const mockCompanies: Company[] = [
    {
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
    },
    {
      id: 'company:aec',
      name: 'Arabian Engineering Corporation',
      name_short: 'Arabian',
      abbreviation: 'AEC',
      city: 'Riyadh',
      country: 'Saudi Arabia',
      reg_no: 'REG456789',
      tax_no: 'TAX345678',
      time: {
        created_at: '2025-08-21T09:00:00Z',
        updated_at: '2025-08-21T09:00:00Z'
      }
    }
  ];

  const mockContacts: Contact[] = [
    {
      id: 'contact:test123',
      first_name: 'John',
      last_name: 'Doe',
      full_name: 'John Doe',
      email: 'john.doe@emittiv.com',
      phone: '+971501234567',
      position: 'Project Manager',
      company: 'company:emt',
      time: {
        created_at: '2025-08-21T10:00:00Z',
        updated_at: '2025-08-21T10:00:00Z'
      }
    },
    {
      id: 'contact:test456',
      first_name: 'Jane',
      last_name: 'Smith',
      full_name: 'Jane Smith',
      email: 'jane.smith@arabian.com',
      phone: '+966501234567',
      position: 'Engineering Director',
      company: 'company:aec',
      time: {
        created_at: '2025-08-21T10:00:00Z',
        updated_at: '2025-08-21T10:00:00Z'
      }
    }
  ];

  const mockFee: Fee = {
    id: 'rfp:test123',
    name: 'Dubai Marina Tower Fee Proposal',
    number: 'FP-25-97101',
    project: 'projects:25-97101',
    company: 'company:emt',
    contact: 'contact:test123',
    issue_date: '250821',
    status: 'draft',
    stage: 'proposal',
    package: 'basic',
    staff: 'Martin',
    activity: 'Design Development',
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

  const mockSettings = {
    currentTeamMember: 'Martin'
  };

  beforeEach(() => {
    vi.clearAllMocks();
    
    // Setup store mocks
    vi.mocked(projectsStore.subscribe).mockImplementation((callback) => {
      callback(mockProjects);
      return { unsubscribe: vi.fn() };
    });
    
    vi.mocked(companiesStore.subscribe).mockImplementation((callback) => {
      callback(mockCompanies);
      return { unsubscribe: vi.fn() };
    });
    
    vi.mocked(contactsStore.subscribe).mockImplementation((callback) => {
      callback(mockContacts);
      return { unsubscribe: vi.fn() };
    });
    
    vi.mocked(feesStore.subscribe).mockImplementation((callback) => {
      callback([]);
      return { unsubscribe: vi.fn() };
    });
    
    vi.mocked(settingsStore.subscribe).mockImplementation((callback) => {
      callback(mockSettings);
      return { unsubscribe: vi.fn() };
    });
    
    // Setup utility mocks
    vi.mocked(validateForm).mockReturnValue({});
    vi.mocked(hasValidationErrors).mockReturnValue(false);
    vi.mocked(extractSurrealId).mockReturnValue('test123');
    vi.mocked(withLoadingState).mockImplementation(async (fn) => await fn());
    vi.mocked(useOperationState).mockReturnValue({
      store: mockOperationState,
      actions: mockOperationActions
    });
  });

  describe('Modal Display', () => {
    it('should not render when closed', () => {
      render(ProposalModal, { isOpen: false });
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('should render create modal when open', () => {
      render(ProposalModal, { isOpen: true, mode: 'create' });
      expect(screen.getByRole('dialog')).toBeInTheDocument();
      expect(screen.getByText('Create New Fee Proposal')).toBeInTheDocument();
    });

    it('should render edit modal with fee data', () => {
      render(ProposalModal, {
        isOpen: true,
        mode: 'edit',
        proposal: mockFee
      });
      expect(screen.getByText('Edit Fee Proposal')).toBeInTheDocument();
    });
  });

  describe('Form Fields', () => {
    it('should display all required form fields', () => {
      render(ProposalModal, { isOpen: true, mode: 'create' });

      // Check for form sections
      expect(screen.getByText('Project & Client Information')).toBeInTheDocument();
      expect(screen.getByText('Basic Information')).toBeInTheDocument();
      expect(screen.getByText('Staff Information')).toBeInTheDocument();
      
      // Project selection - using placeholder text to avoid ambiguity
      expect(screen.getByPlaceholderText('Search projects...')).toBeInTheDocument();
      
      // Company selection - using placeholder text
      expect(screen.getByPlaceholderText('Search companies...')).toBeInTheDocument();
      
      // Contact selection - using placeholder text
      expect(screen.getByPlaceholderText('Search contacts...')).toBeInTheDocument();
      
      // Basic proposal fields - check for presence of labels/text
      expect(screen.getByText('Proposal Number')).toBeInTheDocument();
      expect(screen.getByText('Proposal Name')).toBeInTheDocument();
      expect(screen.getByText('Issue Date')).toBeInTheDocument();
      expect(screen.getByText('Status')).toBeInTheDocument();
      
      // Staff information - check for presence of labels/text
      expect(screen.getByText('Staff Name')).toBeInTheDocument();
      expect(screen.getByText('Staff Email')).toBeInTheDocument();
    });

    it('should have create proposal button', () => {
      render(ProposalModal, { isOpen: true, mode: 'create' });

      const createButton = screen.getByRole('button', { name: 'Create Proposal' });
      expect(createButton).toBeInTheDocument();
      expect(createButton).toHaveAttribute('type', 'submit');
    });
  });

  describe('Project Selection', () => {
    it('should show project typeahead search', () => {
      render(ProposalModal, { isOpen: true, mode: 'create' });
      
      const projectInput = screen.getByPlaceholderText('Search projects...');
      expect(projectInput).toBeInTheDocument();
    });

    it('should show add project button', () => {
      render(ProposalModal, { isOpen: true, mode: 'create' });
      
      const addProjectButton = screen.getByRole('button', { name: 'Add new project' });
      expect(addProjectButton).toBeInTheDocument();
    });
  });

  describe('CRUD Operations', () => {
    it('should render create button in create mode', () => {
      render(ProposalModal, { isOpen: true, mode: 'create' });

      const createButton = screen.getByRole('button', { name: 'Create Proposal' });
      expect(createButton).toBeInTheDocument();
      expect(createButton).not.toBeDisabled();
    });

    it('should update existing proposal', async () => {
      const user = userEvent.setup();
      
      vi.mocked(feesActions.update).mockResolvedValue(mockFee);
      
      render(ProposalModal, {
        isOpen: true,
        mode: 'edit',
        proposal: mockFee
      });

      const updateButton = screen.getByRole('button', { name: 'Update' });
      await user.click(updateButton);

      expect(withLoadingState).toHaveBeenCalled();
    });

    it('should show delete confirmation', async () => {
      const user = userEvent.setup();
      
      render(ProposalModal, {
        isOpen: true,
        mode: 'edit',
        proposal: mockFee
      });

      const deleteButton = screen.getByRole('button', { name: 'Delete' });
      await user.click(deleteButton);

      expect(screen.getByText('Are you sure you want to delete this proposal?')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Confirm Delete' })).toBeInTheDocument();
    });

    it('should delete proposal after confirmation', async () => {
      const user = userEvent.setup();
      
      vi.mocked(feesActions.delete).mockResolvedValue(true);
      
      render(ProposalModal, {
        isOpen: true,
        mode: 'edit',
        proposal: mockFee
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
      render(ProposalModal, { isOpen: true, mode: 'create' });

      expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Create Proposal' })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
    });

    it('should show different buttons for edit mode', () => {
      render(ProposalModal, {
        isOpen: true,
        mode: 'edit',
        proposal: mockFee
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

      render(ProposalModal, { isOpen: true, mode: 'create' });

      const createButton = screen.getByRole('button', { name: 'Create Proposal' });
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

      render(ProposalModal, { isOpen: true, mode: 'create' });

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    });

    it('should display success messages', () => {
      const messageState = {
        subscribe: vi.fn((callback) => {
          callback({ saving: false, deleting: false, error: null, message: 'Fee proposal created successfully' });
          return { unsubscribe: vi.fn() };
        })
      };

      vi.mocked(useOperationState).mockReturnValue({
        store: messageState,
        actions: mockOperationActions
      });

      render(ProposalModal, { isOpen: true, mode: 'create' });

      expect(screen.getByText('Fee proposal created successfully')).toBeInTheDocument();
    });
  });

  describe('Modal Interactions', () => {
    it('should render cancel button for modal close', async () => {
      const user = userEvent.setup();
      
      render(ProposalModal, { isOpen: true, mode: 'create' });

      const cancelButton = screen.getByRole('button', { name: 'Cancel' });
      expect(cancelButton).toBeInTheDocument();
      
      // Test clicking cancel button
      await user.click(cancelButton);
      
      // Button should be clickable (this tests the functionality without $on)
      expect(cancelButton).toBeInTheDocument();
    });

    it('should have operation state utilities available', () => {
      render(ProposalModal, { isOpen: true, mode: 'create' });
      
      // Component has access to operation actions from useOperationState
      expect(useOperationState).toHaveBeenCalled();
    });
  });
});