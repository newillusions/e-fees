/**
 * ConnectionStatus Component Tests
 * 
 * Tests for the database connection status indicator including
 * connection states, LED visual feedback, and automatic monitoring.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import ConnectionStatus from './ConnectionStatus.svelte';
import { connectionStore } from '../stores';
import type { ConnectionState } from '../stores';

// Mock the API
vi.mock('../api', () => ({
  getConnectionStatus: vi.fn(),
  getDbInfo: vi.fn(),
  getProjects: vi.fn(),
  getCompanies: vi.fn(),
  getContacts: vi.fn(),
  getFees: vi.fn()
}));

// Mock the stores module for loadAllData
vi.mock('../stores', async () => {
  const actual = await vi.importActual('../stores');
  return {
    ...actual,
    loadAllData: vi.fn()
  };
});

// Mock the utils module for getAppVersion
vi.mock('../utils', () => ({
  getAppVersion: vi.fn().mockResolvedValue('0.10.4')
}));

import { getConnectionStatus, getDbInfo, getProjects, getCompanies, getContacts, getFees } from '../api';

describe('ConnectionStatus Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.clearAllTimers();
    vi.useFakeTimers();
    
    // Set up default mock returns
    vi.mocked(getConnectionStatus).mockResolvedValue({
      is_connected: false,
      last_check: undefined,
      error_message: undefined
    });
    vi.mocked(getDbInfo).mockResolvedValue({});
    vi.mocked(getProjects).mockResolvedValue([]);
    vi.mocked(getCompanies).mockResolvedValue([]);
    vi.mocked(getContacts).mockResolvedValue([]);
    vi.mocked(getFees).mockResolvedValue([]);
    
    // Reset connection store to initial state
    const initialState: ConnectionState = {
      isConnected: false,
      status: 'Disconnected'
    };
    connectionStore.set(initialState);
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  describe('Visual States', () => {
    it('should show disconnected state initially', async () => {
      render(ConnectionStatus);

      expect(screen.getByText('Disconnected')).toBeInTheDocument();
      expect(screen.getByText('Disconnected')).toHaveClass('text-xs', 'font-medium');

      // Should show version information (wait for async load)
      await waitFor(() => {
        expect(screen.getByText('v0.10.4')).toBeInTheDocument();
      });
    });

    it('should show connected state', () => {
      const connectedState: ConnectionState = {
        isConnected: true,
        status: 'Connected',
        lastChecked: new Date()
      };
      connectionStore.set(connectedState);
      
      render(ConnectionStatus);
      
      expect(screen.getByText('Connected')).toBeInTheDocument();
      expect(screen.getByText('Connected')).toHaveClass('text-xs', 'font-medium');
      
      // Should show last checked time
      const lastCheckedTime = connectedState.lastChecked?.toLocaleTimeString();
      expect(screen.getByText(lastCheckedTime!)).toBeInTheDocument();
    });

    it('should show connecting state', () => {
      const connectingState: ConnectionState = {
        isConnected: false,
        status: 'Connecting...'
      };
      connectionStore.set(connectingState);
      
      render(ConnectionStatus);
      
      expect(screen.getByText('Connecting...')).toBeInTheDocument();
      expect(screen.getByText('Connecting...')).toHaveClass('text-xs', 'font-medium');
    });

    it('should show error state with message', () => {
      const errorState: ConnectionState = {
        isConnected: false,
        status: 'Connection Error',
        errorMessage: 'Database connection failed'
      };
      connectionStore.set(errorState);
      
      render(ConnectionStatus);
      
      expect(screen.getByText('Connection Error')).toBeInTheDocument();
      expect(screen.getByText('Connection Error')).toHaveClass('text-xs', 'font-medium');
      
      // Should show error message (truncated if long)
      expect(screen.getByTitle('Database connection failed')).toBeInTheDocument();
    });

    it('should truncate long error messages', () => {
      const longErrorMessage = 'This is a very long error message that should be truncated when displayed';
      const errorState: ConnectionState = {
        isConnected: false,
        status: 'Connection Error',
        errorMessage: longErrorMessage
      };
      connectionStore.set(errorState);
      
      render(ConnectionStatus);
      
      // Should show truncated message with ellipsis
      const truncatedText = screen.getByText(/This is a very long error/);
      expect(truncatedText.textContent).toMatch(/\.\.\.$/);
      expect(truncatedText).toHaveAttribute('title', longErrorMessage);
    });
  });

  describe('LED Indicator', () => {
    it('should show disconnected LED state', () => {
      const { container } = render(ConnectionStatus);
      
      const ledElement = container.querySelector('.led-disconnected');
      expect(ledElement).toBeInTheDocument();
      expect(ledElement).toHaveClass('w-1', 'h-1', 'rounded-full');
    });

    it('should show connected LED state with glow', () => {
      const connectedState: ConnectionState = {
        isConnected: true,
        status: 'Connected'
      };
      connectionStore.set(connectedState);
      
      const { container } = render(ConnectionStatus);
      
      const ledElement = container.querySelector('.led-connected');
      expect(ledElement).toBeInTheDocument();
      expect(ledElement).toHaveClass('w-1', 'h-1', 'rounded-full');
      
      // Should show glow effect
      const glowElement = container.querySelector('.led-glow-connected');
      expect(glowElement).toBeInTheDocument();
      expect(glowElement).toHaveClass('animate-pulse');
    });
  });

  describe('Connection Monitoring', () => {
    it('should start with initial delay on mount', async () => {
      render(ConnectionStatus);

      // Should not call immediately
      expect(getConnectionStatus).not.toHaveBeenCalled();

      // Advance timers for initial 2 second delay
      await vi.advanceTimersByTimeAsync(2000);

      await waitFor(() => {
        expect(getConnectionStatus).toHaveBeenCalledTimes(1);
      });
    });

    it('should periodically check connection status', async () => {
      const mockConnectionStatus = {
        is_connected: true,
        last_check: new Date().toISOString()
      };

      vi.mocked(getConnectionStatus).mockResolvedValue(mockConnectionStatus);

      render(ConnectionStatus);

      // Initial check after delay
      await vi.advanceTimersByTimeAsync(2000);
      await waitFor(() => {
        expect(getConnectionStatus).toHaveBeenCalledTimes(1);
      });

      // Periodic check every 30 seconds
      await vi.advanceTimersByTimeAsync(30000);
      await waitFor(() => {
        expect(getConnectionStatus).toHaveBeenCalledTimes(2);
      });
    });

    it('should clean up interval on unmount', async () => {
      const clearIntervalSpy = vi.spyOn(global, 'clearInterval');

      const { unmount } = render(ConnectionStatus);

      // Allow time for interval to be set up
      await vi.advanceTimersByTimeAsync(2000);

      unmount();

      expect(clearIntervalSpy).toHaveBeenCalled();
    });
  });

  describe('Component Structure', () => {
    it('should render with correct container structure', () => {
      const { container } = render(ConnectionStatus);
      
      // Should have main container with flex layout
      const mainContainer = container.querySelector('.flex.items-center.px-6.py-2');
      expect(mainContainer).toBeInTheDocument();
      
      // Should have LED and status text container
      const contentContainer = container.querySelector('.flex.items-center.space-x-3');
      expect(contentContainer).toBeInTheDocument();
    });

    it('should render status text in correct structure', async () => {
      const { container } = render(ConnectionStatus);

      // Should have flex column for status text
      const statusContainer = container.querySelector('.flex.flex-col');
      expect(statusContainer).toBeInTheDocument();

      // Should show version number (wait for async load)
      await waitFor(() => {
        expect(screen.getByText('v0.10.4')).toBeInTheDocument();
      });
    });
  });
});