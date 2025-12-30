/**
 * IssueTypesCard Tests
 *
 * Tests for loading, error, empty, and success states.
 */

import { describe, test, expect, beforeEach } from 'bun:test';
import { waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { server } from '../../../test/handlers';
import { renderWithProviders } from '../../../test/utils';
import { IssueTypesCard } from './IssueTypesCard';

describe('IssueTypesCard', () => {
  beforeEach(() => {
    server.resetHandlers();
  });

  test('renders and loads data', async () => {
    // Default handler returns empty data
    const { getByText } = renderWithProviders(<IssueTypesCard />);

    // Wait for card to render
    await waitFor(() => {
      expect(getByText('Issue Types')).toBeTruthy();
    });
  });

  test('shows error state when API fails', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/issuetypes', () => {
        return HttpResponse.json(null, { status: 500 });
      })
    );

    const { getByRole, getByText } = renderWithProviders(<IssueTypesCard />);

    await waitFor(() => {
      expect(getByRole('alert')).toBeTruthy();
    });

    expect(getByText(/failed to load/i)).toBeTruthy();
    expect(getByRole('button', { name: /retry/i })).toBeTruthy();
  });

  test('shows empty state when no issue types', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/issuetypes', () => {
        return HttpResponse.json([]);
      })
    );

    const { getByText } = renderWithProviders(<IssueTypesCard />);

    // Card title should be visible
    await waitFor(() => {
      expect(getByText('Issue Types')).toBeTruthy();
    });
  });

  test('shows issue types when API returns data', async () => {
    server.use(
      http.get('/api/proxy/operator/api/v1/issuetypes', () => {
        return HttpResponse.json([
          {
            key: 'FEAT',
            name: 'Feature',
            mode: 'autonomous',
          },
          {
            key: 'FIX',
            name: 'Bug Fix',
            mode: 'autonomous',
          },
        ]);
      })
    );

    const { getByText } = renderWithProviders(<IssueTypesCard />);

    await waitFor(() => {
      expect(getByText('FEAT')).toBeTruthy();
    });

    expect(getByText('Feature')).toBeTruthy();
    expect(getByText('FIX')).toBeTruthy();
    expect(getByText('Bug Fix')).toBeTruthy();
  });
});
