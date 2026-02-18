import { test, expect } from "../fixtures/api-mock";

test.describe("edit library item", () => {
  test("navigate to edit form and see pre-populated fields for a piece", async ({
    page,
  }) => {
    await page.goto("/");

    // Navigate to Clair de Lune detail
    await page.getByRole("heading", { name: "Clair de Lune" }).click();
    await expect(
      page.getByRole("heading", { name: "Clair de Lune", level: 2 })
    ).toBeVisible();

    // Click Edit link
    await page.getByRole("link", { name: "Edit" }).click();

    // Should be on the edit form
    await expect(
      page.getByRole("heading", { name: "Edit Library Item" })
    ).toBeVisible();

    // Fields should be pre-populated
    await expect(page.locator("#edit-title")).toHaveValue("Clair de Lune");
    await expect(page.locator("#edit-composer")).toHaveValue("Claude Debussy");
    await expect(page.locator("#edit-key")).toHaveValue("Db Major");
    await expect(page.locator("#edit-tempo-marking")).toHaveValue(
      "Andante très expressif"
    );
    await expect(page.locator("#edit-bpm")).toHaveValue("66");
    await expect(page.locator("#edit-notes")).toHaveValue(
      "Third movement of Suite bergamasque"
    );
  });

  test("edit piece title and save", async ({ page }) => {
    await page.goto("/");

    // Navigate to piece detail then edit
    await page.getByRole("heading", { name: "Clair de Lune" }).click();
    await page.getByRole("link", { name: "Edit" }).click();
    await expect(
      page.getByRole("heading", { name: "Edit Library Item" })
    ).toBeVisible();

    // Clear and change the title
    await page.locator("#edit-title").fill("Clair de Lune (Revised)");

    // Save
    await page.getByRole("button", { name: "Save" }).click();

    // Should redirect back to detail with updated title
    await expect(
      page.getByRole("heading", { name: "Clair de Lune (Revised)", level: 2 })
    ).toBeVisible();
  });

  test("edit exercise category field", async ({ page }) => {
    await page.goto("/");

    // Navigate to Hanon No. 1 detail then edit
    await page.getByRole("heading", { name: "Hanon No. 1" }).click();
    await page.getByRole("link", { name: "Edit" }).click();
    await expect(
      page.getByRole("heading", { name: "Edit Library Item" })
    ).toBeVisible();

    // Category field should be visible and pre-populated for exercises
    await expect(page.locator("#edit-category")).toHaveValue("Technique");

    // Change it
    await page.locator("#edit-category").fill("Finger Independence");

    // Save
    await page.getByRole("button", { name: "Save" }).click();

    // Should redirect back to detail
    await expect(
      page.getByRole("heading", { name: "Hanon No. 1", level: 2 })
    ).toBeVisible();
  });

  test("cancel edit returns to detail without changes", async ({ page }) => {
    await page.goto("/");

    // Navigate to piece detail then edit
    await page.getByRole("heading", { name: "Clair de Lune" }).click();
    await page.getByRole("link", { name: "Edit" }).click();
    await expect(
      page.getByRole("heading", { name: "Edit Library Item" })
    ).toBeVisible();

    // Change the title but cancel
    await page.locator("#edit-title").fill("CHANGED TITLE");
    await page.getByRole("link", { name: "Cancel" }).click();

    // Should be back on the detail page with the ORIGINAL title
    await expect(
      page.getByRole("heading", { name: "Clair de Lune", level: 2 })
    ).toBeVisible();
  });
});
