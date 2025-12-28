import { test, expect } from '@playwright/test';
import { gotoWithAuth } from './auth';

test.describe('Plugins Page', () => {
  test('loads plugins page', async ({ page }) => {
    await gotoWithAuth(page, '/plugins');

    // Wait for the page to load
    await expect(page).toHaveURL(/.*plugins/);

    // Check for the Installed Plugins header
    await expect(page.getByRole('heading', { name: 'Installed Plugins' })).toBeVisible();
  });

  test('displays Installed Plugins header', async ({ page }) => {
    await gotoWithAuth(page, '/plugins');

    // Check for the Installed Plugins header
    await expect(page.getByRole('heading', { name: 'Installed Plugins' })).toBeVisible();
  });

  test('displays plugin table with installed plugins', async ({ page }) => {
    await gotoWithAuth(page, '/plugins');

    // Check that the plugins table is visible
    await expect(page.getByRole('table')).toBeVisible();

    // Check that known plugins are listed (using table cells for specificity)
    await expect(page.getByRole('cell', { name: 'catalog', exact: true })).toBeVisible();
    await expect(page.getByRole('cell', { name: 'search', exact: true })).toBeVisible();
    await expect(page.getByRole('cell', { name: 'issuetypes', exact: true })).toBeVisible();
  });

  test('displays plugin metadata', async ({ page }) => {
    await gotoWithAuth(page, '/plugins');

    // Check for plugin descriptions in table cells
    await expect(page.getByRole('cell', { name: /Software catalog/i })).toBeVisible();
    await expect(page.getByRole('cell', { name: /Full-text search/i })).toBeVisible();
    await expect(page.getByRole('cell', { name: /Manage issue types/i })).toBeVisible();
  });
});
