/**
 * Contacts API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  getContacts,
  getContactsPage,
  getContactById,
  createContact,
  updateContact,
  deleteContact
} from './contacts';
import type { Contact, ContactCreate, ContactUpdate, PaginatedResponse } from '../../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

const mockContact: Contact = {
  id: 'contacts:test',
  first_name: 'John',
  last_name: 'Doe',
  full_name: 'John Doe',
  email: 'john@example.com',
  phone: '+971501234567',
  position: 'Manager',
  company: 'company:TEST',
  time: { created_at: '2025-01-18T00:00:00Z', updated_at: '2025-01-18T00:00:00Z' }
};

describe('Contacts API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('getContacts', () => {
    it('should call invoke with correct command', async () => {
      mockInvoke.mockResolvedValueOnce([mockContact]);

      const result = await getContacts();

      expect(mockInvoke).toHaveBeenCalledWith('get_contacts');
      expect(result).toEqual([mockContact]);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await expect(getContacts()).rejects.toThrow('Database error');
    });
  });

  describe('getContactsPage', () => {
    it('should call invoke with pagination params', async () => {
      const mockResponse: PaginatedResponse<Contact> = {
        items: [mockContact],
        total: 1,
        page: 1,
        page_size: 50,
        has_more: false
      };
      mockInvoke.mockResolvedValueOnce(mockResponse);

      const result = await getContactsPage(1, 50);

      expect(mockInvoke).toHaveBeenCalledWith('get_contacts_page', { page: 1, pageSize: 50 });
      expect(result).toEqual(mockResponse);
    });
  });

  describe('getContactById', () => {
    it('should call invoke with contact id', async () => {
      mockInvoke.mockResolvedValueOnce(mockContact);

      const result = await getContactById('test');

      expect(mockInvoke).toHaveBeenCalledWith('get_contact_by_id', { id: 'test' });
      expect(result).toEqual(mockContact);
    });
  });

  describe('createContact', () => {
    it('should call invoke with contact data', async () => {
      mockInvoke.mockResolvedValueOnce(mockContact);

      const contactData: ContactCreate = {
        first_name: 'Jane',
        last_name: 'Smith',
        email: 'jane@example.com',
        phone: '+971509876543',
        position: 'Director',
        company: 'company:TEST'
      };

      const result = await createContact(contactData);

      expect(mockInvoke).toHaveBeenCalledWith('create_contact', {
        contact: expect.objectContaining({
          first_name: 'Jane',
          last_name: 'Smith',
          id: null
        })
      });
      expect(result).toEqual(mockContact);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Create failed'));

      await expect(createContact({} as ContactCreate)).rejects.toThrow('Create failed');
    });
  });

  describe('updateContact', () => {
    it('should call invoke with id and update data', async () => {
      mockInvoke.mockResolvedValueOnce(mockContact);

      const update: ContactUpdate = { position: 'Senior Manager' };
      const result = await updateContact('test', update);

      expect(mockInvoke).toHaveBeenCalledWith('update_contact', {
        id: 'test',
        contactUpdate: update
      });
      expect(result).toEqual(mockContact);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Update failed'));

      await expect(updateContact('test', {})).rejects.toThrow('Update failed');
    });
  });

  describe('deleteContact', () => {
    it('should call invoke with contact id', async () => {
      mockInvoke.mockResolvedValueOnce(mockContact);

      const result = await deleteContact('test');

      expect(mockInvoke).toHaveBeenCalledWith('delete_contact', { id: 'test' });
      expect(result).toEqual(mockContact);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Delete failed'));

      await expect(deleteContact('test')).rejects.toThrow('Delete failed');
    });
  });
});
