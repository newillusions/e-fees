/**
 * Activity Logger Service Tests
 *
 * Regression coverage for the 2026-09-27 activity-feed logging-gap fix:
 * proves each entity logger builds a schema-valid ActivityLogCreate payload
 * and calls through to `create_activity_log`. Complements the live-DB Rust
 * tests in `src-tauri/src/db/tests.rs`
 * (test_get_activity_logs_filters_backfill_prefixed_users,
 * test_create_activity_log_merge_action_round_trips), which prove the
 * payloads these functions build actually persist against a real
 * `activity_log` table.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  projectLogger,
  companyLogger,
  contactLogger,
  feeLogger,
  logMerge,
  getEntityDisplayName
} from './activityLogger';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('activityLogger service', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue({ id: 'activity_log:test' });
  });

  describe('projectLogger', () => {
    it('logs create with a schema-valid action/entity_type', async () => {
      await projectLogger.onCreate({ id: 'projects:26_97104', name: 'Test Project' });

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', {
        log: expect.objectContaining({
          action: 'create',
          entity_type: 'project',
          entity_id: 'projects:26_97104',
          entity_name: 'Test Project'
        })
      });
    });

    it('logs status_change with old/new values', async () => {
      await projectLogger.onStatusChange(
        { id: 'projects:26_97104', name: 'Test Project' },
        'RFP',
        'Design'
      );

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', {
        log: expect.objectContaining({
          action: 'status_change',
          entity_type: 'project',
          old_value: 'RFP',
          new_value: 'Design'
        })
      });
    });
  });

  describe('feeLogger', () => {
    it('logs create (the clone_fee_revision "New Revision" write path)', async () => {
      await feeLogger.onCreate({ id: 'fee:abc123', number: 'FP-02' });

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', {
        log: expect.objectContaining({
          action: 'create',
          entity_type: 'fee',
          entity_id: 'fee:abc123'
        })
      });
    });

    it('logs status_change to Superseded (the source fee of a new revision)', async () => {
      await feeLogger.onStatusChange(
        { id: 'fee:source1', number: 'FP-01' },
        'Sent',
        'Superseded'
      );

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', {
        log: expect.objectContaining({
          action: 'status_change',
          entity_type: 'fee',
          old_value: 'Sent',
          new_value: 'Superseded'
        })
      });
    });

    it('logs update (the pricing-modal write path)', async () => {
      await feeLogger.onUpdate({ id: 'fee:abc123', number: 'FP-01' }, [
        'pricing',
        'payment_schedule'
      ]);

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', {
        log: expect.objectContaining({
          action: 'update',
          entity_type: 'fee',
          entity_id: 'fee:abc123',
          description: expect.stringContaining('pricing, payment_schedule')
        })
      });
    });
  });

  describe('companyLogger / contactLogger', () => {
    it('companyLogger.onDelete logs a valid delete', async () => {
      await companyLogger.onDelete('company:abc', 'DELETE ME - Test Co');

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', {
        log: expect.objectContaining({
          action: 'delete',
          entity_type: 'company',
          entity_id: 'company:abc'
        })
      });
    });

    it('contactLogger.onCreate logs a valid create', async () => {
      await contactLogger.onCreate({ id: 'contacts:abc', full_name: 'Jane Doe' });

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', {
        log: expect.objectContaining({
          action: 'create',
          entity_type: 'contact',
          entity_id: 'contacts:abc'
        })
      });
    });
  });

  describe('logMerge', () => {
    it("builds action:'merge' (schema-valid only after migration v008)", async () => {
      await logMerge('projects:target1', 'Target Project', 'Source Project');

      expect(mockInvoke).toHaveBeenCalledWith('create_activity_log', {
        log: expect.objectContaining({
          action: 'merge',
          entity_type: 'project',
          entity_id: 'projects:target1',
          old_value: 'Source Project',
          new_value: 'Target Project'
        })
      });
    });
  });

  describe('getEntityDisplayName', () => {
    it('falls back through name -> project_number -> Unknown Project', () => {
      expect(getEntityDisplayName('project', { name: 'Named' })).toBe('Named');
      expect(getEntityDisplayName('project', { project_number: '26-97104' })).toBe('26-97104');
      expect(getEntityDisplayName('project', {})).toBe('Unknown Project');
    });
  });
});
