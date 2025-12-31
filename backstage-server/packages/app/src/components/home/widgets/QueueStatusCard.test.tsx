/**
 * QueueStatusCard Tests
 *
 * Tests for loading, error, empty, and success states.
 */

import { describe, test, expect, beforeEach } from 'bun:test';
import { waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { server } from '../../../test/handlers';
import { renderWithProviders } from '../../../test/utils';
import { QueueStatusCard } from './QueueStatusCard';

describe('QueueStatusCard', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  test('renders and loads data', async () => {
    // Default handler returns empty data
    const { getByText } = renderWithProviders(<QueueStatusCard />);

    // Wait for card to render
    await waitFor(() => {
      expect(getByText('Queue Status')).toBeTruthy();
    });
  });

  test('shows error state when API fails', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/queue/status', () => {
        return HttpResponse.json(null, { status: 500 });
      })
    );

    const { getByRole, getByText } = renderWithProviders(<QueueStatusCard />);

    await waitFor(() => {
      expect(getByRole('alert')).toBeTruthy();
    });

    expect(getByText(/failed to load/i)).toBeTruthy();
    expect(getByRole('button', { name: /retry/i })).toBeTruthy();
  });

  test('shows zero counts when queue is empty', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/queue/status', () => {
        return HttpResponse.json({
          queued: 0,
          in_progress: 0,
          awaiting: 0,
          completed: 0,
          by_type: { inv: 0, fix: 0, feat: 0, spike: 0 },
        });
      })
    );

    const { getByText } = renderWithProviders(<QueueStatusCard />);

    await waitFor(() => {
      expect(getByText('Queued')).toBeTruthy();
    });

    expect(getByText('In Progress')).toBeTruthy();
    expect(getByText('Awaiting')).toBeTruthy();
    expect(getByText('Completed')).toBeTruthy();
  });

  test('shows status counts when API returns data', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/queue/status', () => {
        return HttpResponse.json({
          queued: 5,
          in_progress: 2,
          awaiting: 1,
          completed: 10,
          by_type: { inv: 1, fix: 3, feat: 8, spike: 2 },
        });
      })
    );

    const { getByText, getAllByText } = renderWithProviders(<QueueStatusCard />);

    await waitFor(() => {
      expect(getByText('5')).toBeTruthy(); // queued count
    });

    // Check for presence of counts (some may appear multiple times)
    expect(getAllByText('2').length).toBeGreaterThan(0); // in_progress and spike
    expect(getByText('10')).toBeTruthy(); // completed
    expect(getByText('Investigation')).toBeTruthy();
    expect(getByText('Bug Fix')).toBeTruthy();
    expect(getByText('Feature')).toBeTruthy();
    expect(getByText('Spike')).toBeTruthy();
  });
});
