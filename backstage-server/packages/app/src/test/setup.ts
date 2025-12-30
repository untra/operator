/**
 * Test Setup
 *
 * Global test setup for DOM, MSW and testing utilities.
 * Uses @happy-dom/global-registrator for proper DOM registration.
 */

import { GlobalRegistrator } from '@happy-dom/global-registrator';

// Register happy-dom globals with a base URL for relative path resolution
GlobalRegistrator.register({
  url: 'http://localhost:3000',
});

import { beforeAll, afterEach, afterAll } from 'bun:test';
import { cleanup } from '@testing-library/react';
import { server } from './handlers';

// Start MSW server before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));

// Reset handlers and cleanup DOM after each test
afterEach(() => {
  server.resetHandlers();
  cleanup();
});

// Close MSW server after all tests
afterAll(() => server.close());
