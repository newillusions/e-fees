/**
 * FirstRunSetup Component Tests
 *
 * Regression coverage for the first-run connection-test bug (2026-08-12,
 * fresh Windows install): testConnection() saved settings then called
 * check_db_connection, but that command only reports whether a DB client
 * ALREADY exists - it never attempts a connection. On first run no client
 * has been constructed yet, so Test always failed with "No database client
 * available" even against a reachable server. The fix invokes
 * reconnect_database (which builds a client from the just-saved settings)
 * before checking connection status, and surfaces the real reconnect error
 * instead of the generic failure message when it throws.
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
  reconnectDatabase: vi.fn()
}));

import { checkDbConnection, saveSettings, getSettings, reconnectDatabase } from '../api';

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
    vi.mocked(reconnectDatabase).mockImplementation(async () => {
      callOrder.push('reconnectDatabase');
      return 'Database reconnected successfully';
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

  it('reconnects using the just-saved settings before checking the connection', async () => {
    await fillAndSubmit();

    await waitFor(() => {
      expect(screen.getByText('Connection successful!')).toBeInTheDocument();
    });

    expect(saveSettings).toHaveBeenCalledTimes(1);
    expect(reconnectDatabase).toHaveBeenCalledTimes(1);

    // testConnection's own checkDbConnection call must come AFTER both
    // saveSettings and reconnectDatabase (onMount's own earlier call, if any,
    // is allowed - only the last call matters for the fix under test).
    const saveIdx = callOrder.indexOf('saveSettings');
    const reconnectIdx = callOrder.indexOf('reconnectDatabase');
    const lastCheckIdx = callOrder.lastIndexOf('checkDbConnection');

    expect(reconnectIdx).toBeGreaterThan(saveIdx);
    expect(lastCheckIdx).toBeGreaterThan(reconnectIdx);
  });

  it('surfaces the real reconnect error instead of the generic failure message', async () => {
    vi.mocked(reconnectDatabase).mockImplementation(async () => {
      callOrder.push('reconnectDatabase');
      throw new Error('Unable to connect to ws://10.0.23.11:8000: connection refused');
    });

    await fillAndSubmit();

    await waitFor(() => {
      expect(screen.getByText(/connection refused/)).toBeInTheDocument();
    });

    expect(
      screen.queryByText('Failed to connect. Please check your settings.')
    ).not.toBeInTheDocument();
  });
});
