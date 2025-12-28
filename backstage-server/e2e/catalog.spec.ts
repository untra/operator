import { test, expect } from '@playwright/test';

test.describe('Catalog Page', () => {
  test('loads with expected banner', async ({ page }) => {
    await page.goto('/catalog');

    // Wait for the page to load
    await expect(page).toHaveURL(/.*catalog/);

    // Check for the catalog page banner data-testid
    await expect(page.getByTestId('catalog-page-banner')).toBeVisible();
  });

  test('displays Repositories header', async ({ page }) => {
    await page.goto('/catalog');

    // Check for the Repositories header (from OperatorCatalogPage)
    await expect(page.getByRole('heading', { name: 'Repositories' })).toBeVisible();
  });

  test('can toggle between Operator and Backstage views', async ({ page }) => {
    await page.goto('/catalog');

    // Check for view toggle buttons
    const operatorBtn = page.getByRole('button', { name: /Operator/i });
    const backstageBtn = page.getByRole('button', { name: /Backstage/i });

    await expect(operatorBtn).toBeVisible();
    await expect(backstageBtn).toBeVisible();

    // Click Backstage view
    await backstageBtn.click();
    await expect(page).toHaveURL(/.*view=backstage/);

    // Click Operator view
    await operatorBtn.click();
    await expect(page).not.toHaveURL(/.*view=backstage/);
  });
});
