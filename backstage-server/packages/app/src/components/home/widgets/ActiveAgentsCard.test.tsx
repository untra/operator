/**
 * ActiveAgentsCard Tests
 *
 * Tests for loading, error, empty, and success states.
 */

import { describe, test, expect, beforeEach } from 'bun:test';
import { waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { server } from '../../../test/handlers';
import { renderWithProviders } from '../../../test/utils';
import { ActiveAgentsCard } from './ActiveAgentsCard';

describe('ActiveAgentsCard', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  test('renders and loads data', async () => {
    // Default handler returns empty data
    const { getByText } = renderWithProviders(<ActiveAgentsCard />);

    // Wait for card to render
    await waitFor(() => {
      expect(getByText('Active Agents')).toBeTruthy();
    });
  });

  test('shows error state when API fails', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/agents/active', () => {
        return HttpResponse.json(null, { status: 500 });
      })
    );

    const { getByRole, getByText } = renderWithProviders(<ActiveAgentsCard />);

    await waitFor(() => {
      expect(getByRole('alert')).toBeTruthy();
    });

    expect(getByText(/failed to load/i)).toBeTruthy();
    expect(getByRole('button', { name: /retry/i })).toBeTruthy();
  });

  test('shows empty state when no agents running', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/agents/active', () => {
        return HttpResponse.json({ agents: [], count: 0 });
      })
    );

    const { getByText } = renderWithProviders(<ActiveAgentsCard />);

    await waitFor(() => {
      expect(getByText('No agents running')).toBeTruthy();
    });
  });

  test('shows agents when API returns data', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/agents/active', () => {
        return HttpResponse.json({
          agents: [
            {
              id: 'agent-1',
              ticket_id: 'FEAT-1234',
              ticket_type: 'FEAT',
              project: 'operator',
              status: 'running',
              mode: 'autonomous',
              started_at: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
              current_step: 'implement',
            },
          ],
          count: 1,
        });
      })
    );

    const { getByText } = renderWithProviders(<ActiveAgentsCard />);

    await waitFor(() => {
      expect(getByText('FEAT-1234')).toBeTruthy();
    });

    expect(getByText('operator')).toBeTruthy();
    expect(getByText('Auto')).toBeTruthy();
  });
});
