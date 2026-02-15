import { test, expect } from "@playwright/test";

test.describe("navigation", () => {
  test("header links navigate between library and sessions", async ({
    page,
  }) => {
    await page.goto("/");

    // Navigate to Sessions via header nav
    await page.getByRole("link", { name: "Sessions" }).click();
    await expect(
      page.getByRole("heading", { name: "Practice Sessions" })
    ).toBeVisible();

    // Navigate back to Library via header nav
    await page.getByRole("link", { name: "Library" }).click();
    await expect(
      page.getByRole("heading", { name: "Welcome to Intrada" })
    ).toBeVisible();
  });

  test("clicking a library item navigates to detail view", async ({
    page,
  }) => {
    await page.goto("/");

    // Click the first stub item (Clair de Lune)
    await page.getByRole("heading", { name: "Clair de Lune" }).click();

    // Should show the detail view with the item title
    await expect(
      page.getByRole("heading", { name: "Clair de Lune", level: 2 })
    ).toBeVisible();

    // Should show composer as subtitle
    await expect(page.getByText("Claude Debussy")).toBeVisible();

    // Back link returns to library
    await page.getByRole("link", { name: "Back to Library" }).click();
    await expect(
      page.getByRole("heading", { name: "Welcome to Intrada" })
    ).toBeVisible();
  });

  test("add item page is reachable and has cancel link", async ({ page }) => {
    await page.goto("/");

    await page.getByRole("link", { name: "Add Item" }).click();
    await expect(
      page.getByRole("heading", { name: "Add Library Item" })
    ).toBeVisible();

    // Cancel navigates back to library
    await page.getByRole("link", { name: "Cancel" }).click();
    await expect(
      page.getByRole("heading", { name: "Welcome to Intrada" })
    ).toBeVisible();
  });

  test("non-existent route shows not found", async ({ page }) => {
    await page.goto("/does-not-exist");
    await expect(page.getByText("Page not found")).toBeVisible();
  });
});
