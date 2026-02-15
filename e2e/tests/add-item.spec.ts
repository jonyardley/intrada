import { test, expect } from "@playwright/test";

test.describe("add library item", () => {
  test("add a piece with all fields", async ({ page }) => {
    await page.goto("/");

    // Navigate to add form
    await page.getByRole("link", { name: "Add Item" }).click();

    // Piece tab should be active by default
    await expect(page.getByRole("tab", { name: "Piece" })).toHaveAttribute(
      "aria-selected",
      "true"
    );

    // Fill in the form
    await page.locator("#add-title").fill("Moonlight Sonata");
    await page.locator("#add-composer").fill("Ludwig van Beethoven");
    await page.locator("#add-key").fill("C# Minor");
    await page.locator("#add-tempo-marking").fill("Adagio sostenuto");
    await page.locator("#add-bpm").fill("60");
    await page.locator("#add-notes").fill("First movement, Op. 27 No. 2");
    await page.locator("#add-tags").fill("classical, beethoven, sonata");

    // Submit the form
    await page.getByRole("button", { name: "Save" }).click();

    // Should redirect to library and show the new item
    await expect(
      page.getByRole("heading", { name: "Welcome to Intrada" })
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Moonlight Sonata" })
    ).toBeVisible();

    // Should now have 3 items (2 stub + 1 new)
    const items = page
      .getByRole("list", { name: "Library items" })
      .locator("li");
    await expect(items).toHaveCount(3);
  });

  test("add an exercise with category", async ({ page }) => {
    await page.goto("/library/new");

    // Switch to Exercise tab
    await page.getByRole("tab", { name: "Exercise" }).click();
    await expect(page.getByRole("tab", { name: "Exercise" })).toHaveAttribute(
      "aria-selected",
      "true"
    );

    // Category field should now be visible
    await expect(page.locator("#add-category")).toBeVisible();

    // Fill required + optional fields
    await page.locator("#add-title").fill("Chromatic Scale");
    await page.locator("#add-category").fill("Scales");
    await page.locator("#add-key").fill("C Major");

    // Submit
    await page.getByRole("button", { name: "Save" }).click();

    // Should appear in library list
    await expect(
      page.getByRole("heading", { name: "Chromatic Scale" })
    ).toBeVisible();
  });

  test("shows validation error when title is empty", async ({ page }) => {
    await page.goto("/library/new");

    // Submit with empty form
    await page.getByRole("button", { name: "Save" }).click();

    // Should show validation errors and stay on page
    await expect(page.getByText("Title is required")).toBeVisible();
    await expect(page.getByText("Composer is required")).toBeVisible();

    // Should still be on the add form
    await expect(
      page.getByRole("heading", { name: "Add Library Item" })
    ).toBeVisible();
  });

  test("switching tabs clears validation errors", async ({ page }) => {
    await page.goto("/library/new");

    // Trigger validation errors on Piece tab
    await page.getByRole("button", { name: "Save" }).click();
    await expect(page.getByText("Title is required")).toBeVisible();

    // Switch to Exercise tab — errors should clear
    await page.getByRole("tab", { name: "Exercise" }).click();
    await expect(page.getByText("Title is required")).not.toBeVisible();
  });

  test("category field only visible for exercises", async ({ page }) => {
    await page.goto("/library/new");

    // Piece tab — no category field
    await expect(page.locator("#add-category")).not.toBeVisible();

    // Switch to Exercise — category appears
    await page.getByRole("tab", { name: "Exercise" }).click();
    await expect(page.locator("#add-category")).toBeVisible();

    // Switch back to Piece — category disappears
    await page.getByRole("tab", { name: "Piece" }).click();
    await expect(page.locator("#add-category")).not.toBeVisible();
  });
});
