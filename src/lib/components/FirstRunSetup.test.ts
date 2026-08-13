/**
 * FirstRunSetup Component Tests
 *
 * Regression coverage for two stacked first-run connection-test bugs.
 *
 * Bug 1 (2026-08-12, fixed in v0.17.0): testConnection() saved settings then
 * called check_db_connection, but that command only reports whether a DB
 * client ALREADY exists - it never attempts a connection. Fixed by calling
 * reconnect_database first.
 *
 * Bug 2 (2026-08-13, this fix): reconnect_database re-runs initialize() with
 * the DatabaseManager's EXISTING self.config - it never re-reads the
 * settings the user just saved. On first run that config is empty, so Test
 * failed with "Reconnection failed: Invalid URL: http://" even with correct
 * credentials (restarting the app worked, because startup builds config
 * fresh from the settings file - proving save was fine and only reconnect's
 * config was stale). Fixed by calling reload_database_config instead, which
 * already re-reads the settings file into a fresh DatabaseConfig via
 * DatabaseManager::reconfigure() before reconnecting - mirroring the app's
 * own startup config-loading path (src-tauri/src/lib.rs
 * load_database_config_from_settings) rather than duplicating it.
 *
 * reload_database_config has a UX-relevant quirk covered below: on a failed
 * reconnect it does NOT throw - it resolves with a string like "Database
 * configuration reloaded but connection failed: <reason>". testConnection()
 * must surface that embedded reason rather than falling through to the
 * generic message.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import FirstRunSetup from './FirstRunSetup.svelte';

// jsdom doesn't implement the Web Animations API that Svelte's transition:fade
// / transition:slide rely on internally. This component is the first in the
// repo with tests to use transitions, so no other test file has needed this
// yet - scoped here rather than in the shared test setup.
if (typeof Element !== 'undefined' && !Element.prototype.animate) {
  Element.prototype.animate = () =>
    ({
      finished: Promise.resolve(),
      cancel: () => {},
      play: () => {},
      pause: () => {},
      addEventListener: () => {},
      removeEventListener: () => {}
    }) as unknown as Animation;
}

vi.mock('../api', () => ({
  checkDbConnection: vi.fn(),
  saveSettings: vi.fn(),
  getSettings: vi.fn(),
  reloadDatabaseConfig: vi.fn()
}));

import { checkDbConnection, saveSettings, getSettings, reloadDatabaseConfig } from '../api';

describe('FirstRunSetup - testConnection', () => {
  const callOrder: string[] = [];

  beforeEach(() => {
    vi.clearAllMocks();
    callOrder.length = 0;

    vi.mocked(getSettings).mockResolvedValue({
      surrealdb_user: 'placeholder',
      has_password: false
    } as unknown as Awaited<ReturnType<typeof getSettings>>);

    vi.mocked(saveSettings).mockImplementation(async () => {
      callOrder.push('saveSettings');
      return 'Settings saved';
    });
    vi.mocked(reloadDatabaseConfig).mockImplementation(async () => {
      callOrder.push('reloadDatabaseConfig');
      return 'Database configuration reloaded and connected successfully';
    });
    vi.mocked(checkDbConnection).mockImplementation(async () => {
      callOrder.push('checkDbConnection');
      return true;
    });
  });

  async function fillAndSubmit() {
    render(FirstRunSetup, { isOpen: true });

    await fireEvent.input(screen.getByLabelText('Database URL'), {
      target: { value: 'ws://10.0.23.11:8000' }
    });
    await fireEvent.input(screen.getByLabelText('Namespace'), {
      target: { value: 'emittiv' }
    });
    await fireEvent.input(screen.getByLabelText('Database'), {
      target: { value: 'projects' }
    });
    await fireEvent.input(screen.getByLabelText('Username'), {
      target: { value: 'root' }
    });
    await fireEvent.input(screen.getByLabelText('Password'), {
      target: { value: 'secret' }
    });

    await fireEvent.click(screen.getByText('Test & Continue'));
  }

  it('reloads config from the just-saved settings before checking the connection', async () => {
    await fillAndSubmit();

    await waitFor(() => {
      expect(screen.getByText('Connection successful!')).toBeInTheDocument();
    });

    expect(saveSettings).toHaveBeenCalledTimes(1);
    expect(reloadDatabaseConfig).toHaveBeenCalledTimes(1);

    // testConnection's own checkDbConnection call must come AFTER both
    // saveSettings and reloadDatabaseConfig (onMount's own earlier call, if
    // any, is allowed - only the last call matters for the fix under test).
    const saveIdx = callOrder.indexOf('saveSettings');
    const reloadIdx = callOrder.indexOf('reloadDatabaseConfig');
    const lastCheckIdx = callOrder.lastIndexOf('checkDbConnection');

    expect(reloadIdx).toBeGreaterThan(saveIdx);
    expect(lastCheckIdx).toBeGreaterThan(reloadIdx);
  });

  it('surfaces the real failure reason when reload resolves but the reconnect inside it failed', async () => {
    // This is the exact shape of Martin's live bug: reload_database_config
    // does NOT throw when the inner reconnect fails - it resolves with the
    // reason embedded in the string, and the client never got constructed
    // (so checkDbConnection still correctly reports false afterward).
    vi.mocked(reloadDatabaseConfig).mockImplementation(async () => {
      callOrder.push('reloadDatabaseConfig');
      return 'Database configuration reloaded but connection failed: Reconnection failed: Invalid URL: http://';
    });
    vi.mocked(checkDbConnection).mockImplementation(async () => {
      callOrder.push('checkDbConnection');
      return false;
    });

    await fillAndSubmit();

    await waitFor(() => {
      expect(screen.getByText(/Invalid URL/)).toBeInTheDocument();
    });

    expect(
      screen.queryByText('Failed to connect. Please check your settings.')
    ).not.toBeInTheDocument();
  });

  it('surfaces a genuine invoke-level error if reloadDatabaseConfig itself throws', async () => {
    vi.mocked(reloadDatabaseConfig).mockImplementation(async () => {
      callOrder.push('reloadDatabaseConfig');
      throw new Error('IPC error: reload_database_config not registered');
    });

    await fillAndSubmit();

    await waitFor(() => {
      expect(screen.getByText(/reload_database_config not registered/)).toBeInTheDocument();
    });

    expect(
      screen.queryByText('Failed to connect. Please check your settings.')
    ).not.toBeInTheDocument();
  });
});
