import { test, expect } from "../fixtures/api-mock";

test.describe("add library item", () => {
  test("add a piece with all fields", async ({ page }) => {
    await page.goto("/");

    // Open Add Item sheet — the CTA is now a button that opens a
    // BottomSheet over the library list (iOS-native pattern). .first()
    // because the library page has an "Add Item" CTA in the header and a
    // second one in the empty state.
    await page.getByRole("button", { name: "Add Item" }).first().click();

    // Piece tab should be active by default. exact: true — the Library page
    // behind the sheet has its own "Pieces" tab; substring match would
    // resolve to two elements.
    await expect(
      page.getByRole("tab", { name: "Piece", exact: true })
    ).toHaveAttribute("aria-selected", "true");

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

    // Should redirect to library and show the new item. Library rows
    // are spans/links post-2026-refresh, not headings — assert against
    // the list contents directly. Library defaults to the Pieces tab,
    // so the new piece is visible without switching.
    await expect(
      page.getByRole("heading", { name: "Library" })
    ).toBeVisible();
    await expect(
      page
        .getByRole("list", { name: "Library items" })
        .getByText("Moonlight Sonata")
    ).toBeVisible();

    // Default tab is Pieces — should show 2 piece rows (stub piece + new).
    // Stub data is 1 piece + 1 exercise; the exercise lives behind the other tab.
    const items = page
      .getByRole("list", { name: "Library items" })
      .locator("li");
    await expect(items).toHaveCount(2);
  });

  test("add an exercise", async ({ page }) => {
    await page.goto("/library/new");

    // Switch to Exercise tab
    await page.getByRole("tab", { name: "Exercise" }).click();
    await expect(page.getByRole("tab", { name: "Exercise" })).toHaveAttribute(
      "aria-selected",
      "true"
    );

    // Fill required + optional fields
    await page.locator("#add-title").fill("Chromatic Scale");
    await page.locator("#add-key").fill("C Major");

    // Submit
    await page.getByRole("button", { name: "Save" }).click();

    // Library defaults to Pieces tab; the new exercise lives behind the
    // Exercises tab. Switch tabs, then assert against the list contents.
    await page.getByRole("tab", { name: "Exercises" }).click();
    await expect(
      page
        .getByRole("list", { name: "Library items" })
        .getByText("Chromatic Scale")
    ).toBeVisible();
  });

  test("composer is required for pieces (native validation)", async ({
    page,
  }) => {
    await page.goto("/library/new");

    // Composer field should have the required attribute on Piece tab
    await expect(page.locator("#add-composer")).toHaveAttribute(
      "required",
      ""
    );

    // Fill title but leave composer empty, then try to submit
    await page.locator("#add-title").fill("Test Piece");
    await page.getByRole("button", { name: "Save" }).click();

    // Should still be on the add form (native validation blocks submission)
    await expect(
      page.getByRole("heading", { name: "Add Library Item" })
    ).toBeVisible();
  });

  test("switching tabs clears validation errors", async ({ page }) => {
    await page.goto("/library/new");

    // Fill required fields, then add an invalid BPM to trigger custom validation
    await page.locator("#add-title").fill("Test Piece");
    await page.locator("#add-composer").fill("Test Composer");
    await page.locator("#add-bpm").fill("9999");
    await page.getByRole("button", { name: "Save" }).click();

    // Custom validation error for BPM should appear
    await expect(page.getByText("BPM must be between")).toBeVisible();

    // Switch to Exercise tab — errors should clear
    await page.getByRole("tab", { name: "Exercise" }).click();
    await expect(page.getByText("BPM must be between")).not.toBeVisible();
  });

});
