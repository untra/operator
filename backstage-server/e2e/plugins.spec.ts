import { test, expect } from '@playwright/test';

test.describe('Plugins Page', () => {
  test('loads with expected banner', async ({ page }) => {
    await page.goto('/plugins');

    // Wait for the page to load
    await expect(page).toHaveURL(/.*plugins/);

    // Check for the plugins page banner data-testid
    await expect(page.getByTestId('plugins-page-banner')).toBeVisible();
  });

  test('displays Installed Plugins header', async ({ page }) => {
    await page.goto('/plugins');

    // Check for the Installed Plugins header
    await expect(page.getByRole('heading', { name: 'Installed Plugins' })).toBeVisible();
  });

  test('displays plugin table with installed plugins', async ({ page }) => {
    await page.goto('/plugins');

    // Check that the plugins table is visible
    await expect(page.getByRole('table')).toBeVisible();

    // Check that known plugins are listed
    await expect(page.getByText('catalog')).toBeVisible();
    await expect(page.getByText('search')).toBeVisible();
    await expect(page.getByText('issuetypes')).toBeVisible();
  });

  test('displays plugin metadata', async ({ page }) => {
    await page.goto('/plugins');

    // Check for plugin descriptions
    await expect(page.getByText('Software catalog')).toBeVisible();
    await expect(page.getByText('Full-text search')).toBeVisible();
    await expect(page.getByText('Manage issue types')).toBeVisible();
  });
});
