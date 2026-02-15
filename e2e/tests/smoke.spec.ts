import { test, expect } from "@playwright/test";

test("app renders with library list", async ({ page }) => {
  await page.goto("/");

  // Verify page heading is visible
  await expect(page.getByRole("heading", { name: "Welcome to Intrada" })).toBeVisible();

  // Verify library list renders with at least one item (stub data is seeded on first load)
  await expect(page.getByRole("list", { name: "Library items" })).toBeVisible();
  const items = page.getByRole("list", { name: "Library items" }).locator("li");
  await expect(items).not.toHaveCount(0);
});
