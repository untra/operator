import { test, expect } from '@playwright/test';

test.describe('Issue Types Page', () => {
  test('loads with expected banner', async ({ page }) => {
    await page.goto('/issuetypes');

    // Wait for the page to load
    await expect(page).toHaveURL(/.*issuetypes/);

    // Check for the issue types page banner data-testid
    await expect(page.getByTestId('issuetypes-page-banner')).toBeVisible();
  });

  test('displays Issue Types header', async ({ page }) => {
    await page.goto('/issuetypes');

    // Check for the Issue Types header
    await expect(page.getByRole('heading', { name: 'Issue Types' })).toBeVisible();
  });

  test('displays action buttons', async ({ page }) => {
    await page.goto('/issuetypes');

    // Check for action buttons - may need Operator REST API running
    // These may be in different states depending on API availability
    const createBtn = page.getByRole('button', { name: /Create/i });
    const collectionsBtn = page.getByRole('button', { name: /Collections/i });

    // At least one should be visible when page loads
    await expect(createBtn.or(collectionsBtn)).toBeVisible({ timeout: 10000 });
  });

  test('navigates to collections page', async ({ page }) => {
    await page.goto('/issuetypes/collections');

    await expect(page).toHaveURL(/.*issuetypes\/collections/);
    await expect(page.getByRole('heading', { name: /Collections/i })).toBeVisible();
  });
});
