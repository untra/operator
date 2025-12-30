/**
 * KanbanBoard Tests
 *
 * Tests for loading, error, empty, and success states.
 */

import { describe, test, expect, beforeEach } from 'bun:test';
import { waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { server } from '../../test/handlers';
import { renderWithProviders } from '../../test/utils';
import { KanbanBoard } from './KanbanBoard';

describe('KanbanBoard', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  test('renders and loads data', async () => {
    // Default handler returns empty data
    const { getByText } = renderWithProviders(<KanbanBoard />);

    // Wait for columns to render after data loads
    await waitFor(() => {
      expect(getByText('Queue')).toBeTruthy();
    });
  });

  test('shows error state when API fails', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/queue/kanban', () => {
        return HttpResponse.json(null, { status: 500 });
      })
    );

    const { getByRole, getByText } = renderWithProviders(<KanbanBoard />);

    // Wait for error state to appear
    await waitFor(() => {
      expect(getByRole('alert')).toBeTruthy();
    });

    expect(getByText(/failed to load board/i)).toBeTruthy();
    expect(getByRole('button', { name: /retry/i })).toBeTruthy();
  });

  test('shows empty state when API returns empty data', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/queue/kanban', () => {
        return HttpResponse.json({
          queue: [],
          running: [],
          awaiting: [],
          done: [],
          total_count: 0,
          last_updated: new Date().toISOString(),
        });
      })
    );

    const { getByText } = renderWithProviders(<KanbanBoard />);

    // Wait for columns to render
    await waitFor(() => {
      expect(getByText('Queue')).toBeTruthy();
    });

    expect(getByText('Running')).toBeTruthy();
    expect(getByText('Awaiting')).toBeTruthy();
    expect(getByText('Done')).toBeTruthy();
  });

  test('shows data when API succeeds', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/queue/kanban', () => {
        return HttpResponse.json({
          queue: [
            {
              id: 'FEAT-1234',
              summary: 'Add new feature',
              ticket_type: 'FEAT',
              project: 'operator',
              status: 'queued',
              step: '',
              priority: 'P2-medium',
              timestamp: '20241229-1430',
            },
          ],
          running: [],
          awaiting: [],
          done: [],
          total_count: 1,
          last_updated: new Date().toISOString(),
        });
      })
    );

    const { getByText } = renderWithProviders(<KanbanBoard />);

    // Wait for ticket to appear
    await waitFor(() => {
      expect(getByText('FEAT-1234')).toBeTruthy();
    });

    expect(getByText('Add new feature')).toBeTruthy();
  });
});
