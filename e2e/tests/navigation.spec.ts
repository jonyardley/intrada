import { test, expect } from "../fixtures/api-mock";

test.describe("navigation", () => {
  test("header links navigate between library and sessions", async ({
    page,
  }) => {
    await page.goto("/");

    // Navigate to Practice via header nav
    await page.getByRole("link", { name: "Practice" }).click();
    await expect(
      page.getByRole("heading", { name: "Practice" })
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

  test("add item opens bottom sheet and cancel dismisses it", async ({
    page,
  }) => {
    await page.goto("/");

    // CTA opens the Add Item bottom sheet (iOS-native modal pattern)
    await page.getByRole("button", { name: "Add Item" }).first().click();

    // Sheet has its own nav-bar title "Add Item" and a Cancel button
    await expect(
      page.getByRole("heading", { name: "Add Item" })
    ).toBeVisible();

    // Cancel button in the sheet nav dismisses it — sheet slides off-screen
    // (transform), so we check the open class is removed rather than DOM
    // visibility (the element is still technically in the DOM).
    await page.getByRole("button", { name: "Cancel" }).first().click();
    await expect(page.locator(".bottom-sheet")).not.toHaveClass(
      /bottom-sheet--open/
    );
    await expect(
      page.getByRole("heading", { name: "Welcome to Intrada" })
    ).toBeVisible();
  });

  test("non-existent route shows not found", async ({ page }) => {
    await page.goto("/does-not-exist");
    await expect(page.getByText("Page not found")).toBeVisible();
  });
});
