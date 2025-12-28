import { test, expect } from '@playwright/test';

test.describe('Health Check', () => {
  test('server health endpoint responds', async ({ request }) => {
    const response = await request.get('/health');
    expect(response.ok()).toBeTruthy();

    const body = await response.json();
    expect(body.status).toBe('ok');
  });

  test('API status endpoint responds', async ({ request }) => {
    const response = await request.get('/api/status');
    expect(response.ok()).toBeTruthy();

    const body = await response.json();
    expect(body.status).toBe('running');
    expect(body.mode).toBe('standalone');
  });
});
