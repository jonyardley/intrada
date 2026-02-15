import { test, expect } from "@playwright/test";

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

    // Type badge
    await expect(page.getByText("Piece", { exact: true })).toBeVisible();

    // Key, Tempo, Notes
    await expect(page.getByText("Db Major")).toBeVisible();
    await expect(page.getByText("Andante très expressif")).toBeVisible();
    await expect(
      page.getByText("Third movement of Suite bergamasque")
    ).toBeVisible();

    // Tags
    await expect(page.getByText("impressionist")).toBeVisible();
    await expect(page.getByText("piano")).toBeVisible();

    // Action buttons
    await expect(page.getByRole("link", { name: "Edit" })).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Log Session" })
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

    // Cancel — should dismiss confirmation
    await page.getByRole("button", { name: "Cancel" }).click();
    await expect(
      page.getByText("Are you sure you want to delete this item?")
    ).not.toBeVisible();

    // Click Delete again and confirm
    await page.getByRole("button", { name: "Delete" }).click();
    await page.getByRole("button", { name: "Confirm Delete" }).click();

    // Should redirect to library
    await expect(
      page.getByRole("heading", { name: "Welcome to Intrada" })
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

  test("log a manual practice session", async ({ page }) => {
    await page.goto("/");

    // Navigate to Clair de Lune
    await page.getByRole("heading", { name: "Clair de Lune" }).click();

    // No sessions yet
    await expect(page.getByText("No practice sessions logged")).toBeVisible();

    // Open log session form
    await page.getByRole("button", { name: "Log Session" }).click();
    await expect(
      page.getByRole("heading", { name: "Log Practice Session" })
    ).toBeVisible();

    // Fill in and submit
    await page.locator("#log-duration").fill("25");
    await page.locator("#log-notes").fill("Focused on the arpeggios");
    await page.getByRole("button", { name: "Save" }).click();

    // Session should appear in history
    await expect(page.getByText("25 min")).toBeVisible();
    await expect(page.getByText("Focused on the arpeggios")).toBeVisible();

    // Practice summary updates on re-navigation (detail view captures item at mount)
    await page.getByRole("link", { name: "Back to Library" }).click();
    await page.getByRole("heading", { name: "Clair de Lune" }).click();
    await expect(page.getByText("1 session, 25 min total")).toBeVisible();
  });
});
