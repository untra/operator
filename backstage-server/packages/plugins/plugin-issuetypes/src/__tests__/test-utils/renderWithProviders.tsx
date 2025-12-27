/**
 * Test utilities for rendering components with all required providers.
 *
 * NOTE: These utilities require @testing-library/react and @backstage/test-utils
 * to be installed. They are intended for future integration tests in a browser-like
 * environment. For now, use the simpler bun:test smoke tests.
 *
 * Usage (when dependencies are installed):
 *   import { renderWithProviders } from './test-utils';
 *   const { getByText } = renderWithProviders(<MyComponent />, { route: '/issuetypes' });
 */

import React from 'react';

/**
 * Type definitions for test utilities.
 * These mirror the APIs from @testing-library/react and @backstage/test-utils.
 */
export interface RenderOptions {
  route?: string;
  mockApi?: unknown;
  routes?: React.ReactNode;
}

export interface RenderResult {
  container: HTMLElement;
  getByText: (text: string) => HTMLElement;
  queryByText: (text: string) => HTMLElement | null;
  findByText: (text: string) => Promise<HTMLElement>;
}

/**
 * Placeholder for renderWithProviders.
 *
 * This function requires @testing-library/react and @backstage/test-utils.
 * Install them to enable integration testing:
 *   bun add -d @testing-library/react @backstage/test-utils
 */
export function renderWithProviders(
  _ui: React.ReactElement,
  _options: RenderOptions = {},
): RenderResult {
  throw new Error(
    'renderWithProviders requires @testing-library/react and @backstage/test-utils. ' +
      'Install them with: bun add -d @testing-library/react @backstage/test-utils',
  );
}

/**
 * Placeholder for renderWithRoutes.
 *
 * This function requires @testing-library/react and @backstage/test-utils.
 */
export function renderWithRoutes(
  _routeConfig: Array<{ path: string; element: React.ReactElement }>,
  _options: Omit<RenderOptions, 'routes'> = {},
): RenderResult {
  throw new Error(
    'renderWithRoutes requires @testing-library/react and @backstage/test-utils. ' +
      'Install them with: bun add -d @testing-library/react @backstage/test-utils',
  );
}
