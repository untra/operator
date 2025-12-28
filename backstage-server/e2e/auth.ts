import { Page, expect } from '@playwright/test';

/**
 * Handle Backstage guest authentication
 *
 * Backstage guest auth stores identity in memory (not cookies/storage),
 * so we need to handle login on each page navigation.
 *
 * This function:
 * 1. Checks if we're on the login page
 * 2. If so, clicks "Enter" to login as guest
 * 3. Waits for the login to complete
 */
export async function loginAsGuest(page: Page): Promise<void> {
  const enterButton = page.getByRole('button', { name: 'Enter' });

  try {
    // Short timeout - we just need to check if login button exists
    await enterButton.waitFor({ state: 'visible', timeout: 3000 });
    await enterButton.click();

    // Wait for the login page to disappear (redirected to content)
    await expect(enterButton).not.toBeVisible({ timeout: 10000 });

    // Give the app time to initialize after login
    await page.waitForTimeout(500);
  } catch {
    // Not on login page - already authenticated or different page
  }
}

/**
 * Navigate to a page and handle authentication
 *
 * Use this instead of bare `page.goto()` for protected pages.
 * Handles the login redirect if needed.
 */
export async function gotoWithAuth(page: Page, path: string): Promise<void> {
  await page.goto(path);
  await loginAsGuest(page);
}
