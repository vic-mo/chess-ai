import { test, expect } from '@playwright/test';

test('renders and logs', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByText('Chess AI Scaffold')).toBeVisible();
});
