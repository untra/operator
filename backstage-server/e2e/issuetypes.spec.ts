import { test, expect } from '@playwright/test';
import { gotoWithAuth } from './auth';

test.describe('Issue Types Page', () => {
  test('loads issue types page', async ({ page }) => {
    await gotoWithAuth(page, '/issuetypes');

    // Wait for the page to load
    await expect(page).toHaveURL(/.*issuetypes/);

    // Check for main Issue Types heading
    await expect(page.getByRole('heading', { name: 'Issue Types', level: 1 })).toBeVisible();
  });

  test('displays Issue Types header', async ({ page }) => {
    await gotoWithAuth(page, '/issuetypes');

    // Check for the Issue Types header (h1, not h2 subtitle)
    await expect(page.getByRole('heading', { name: 'Issue Types', level: 1 })).toBeVisible();
  });

  test('displays action buttons', async ({ page }) => {
    await gotoWithAuth(page, '/issuetypes');

    // Check for action buttons
    const createBtn = page.getByRole('button', { name: 'Create Issue Type' });
    const collectionsBtn = page.getByRole('button', { name: 'Collections' });

    // Both should be visible when page loads
    await expect(createBtn).toBeVisible({ timeout: 10000 });
    await expect(collectionsBtn).toBeVisible({ timeout: 10000 });
  });

  test('navigates to collections page', async ({ page }) => {
    await gotoWithAuth(page, '/issuetypes/collections');

    await expect(page).toHaveURL(/.*issuetypes\/collections/);
    // Check for main Collections heading (h1)
    await expect(page.getByRole('heading', { name: 'Collections', level: 1 })).toBeVisible();
  });
});
