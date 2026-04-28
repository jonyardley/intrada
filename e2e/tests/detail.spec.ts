import { test, expect } from "../fixtures/api-mock";

test.describe("detail view", () => {
  test("displays all fields for a stub piece", async ({ page }) => {
    await page.goto("/");

    // Navigate to Clair de Lune detail
    await page.getByRole("heading", { name: "Clair de Lune" }).click();

    // Title and composer
    await expect(
      page.getByRole("heading", { name: "Clair de Lune", level: 2 })
    ).toBeVisible();
    await expect(page.getByText("Claude Debussy")).toBeVisible();

    // Type badge — scope to <main> to avoid matching the "Piece" tab button
    // inside the (DOM-present-but-off-screen) Edit sheet's TypeTabs.
    await expect(
      page.getByRole("main").getByText("Piece", { exact: true })
    ).toBeVisible();

    // Key, Tempo, Notes
    await expect(page.getByText("Db Major")).toBeVisible();
    await expect(page.getByText("Andante très expressif")).toBeVisible();
    await expect(
      page.getByText("Third movement of Suite bergamasque")
    ).toBeVisible();

    // Tags — scope to <main> to skip the duplicate TagInput inside the
    // off-screen Edit sheet (which is mounted but translated out of view).
    await expect(
      page.getByRole("main").getByText("impressionist")
    ).toBeVisible();
    await expect(page.getByRole("main").getByText("piano")).toBeVisible();

    // Action buttons (Edit and Delete — "Log Session" removed in setlist model).
    // Edit is now a <button> that opens an inline BottomSheet (iOS-native
    // pattern); .first() because the sheet's Cancel button below also
    // matches role=button when it's later opened.
    await expect(
      page.getByRole("button", { name: "Edit" })
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Delete" })
    ).toBeVisible();
  });

  test("delete item with confirmation", async ({ page }) => {
    await page.goto("/");

    // Navigate to Hanon No. 1
    await page.getByRole("heading", { name: "Hanon No. 1" }).click();
    await expect(
      page.getByRole("heading", { name: "Hanon No. 1", level: 2 })
    ).toBeVisible();

    // Click Delete — should show confirmation
    await page.getByRole("button", { name: "Delete" }).click();
    await expect(
      page.getByText("Are you sure you want to delete this item?")
    ).toBeVisible();

    // Cancel — should dismiss confirmation. Scope to <main> because the
    // off-screen Edit sheet also has Cancel buttons (sheet nav + form).
    await page.getByRole("main").getByRole("button", { name: "Cancel" }).click();
    await expect(
      page.getByText("Are you sure you want to delete this item?")
    ).not.toBeVisible();

    // Click Delete again and confirm
    await page.getByRole("button", { name: "Delete" }).click();
    await page.getByRole("button", { name: "Confirm Delete" }).click();

    // Should redirect to library
    await expect(
      page.getByRole("heading", { name: "Library" })
    ).toBeVisible();

    // Hanon No. 1 should be gone
    await expect(
      page.getByRole("heading", { name: "Hanon No. 1" })
    ).not.toBeVisible();

    // Only 1 item remaining
    const items = page
      .getByRole("list", { name: "Library items" })
      .locator("li");
    await expect(items).toHaveCount(1);
  });
});
