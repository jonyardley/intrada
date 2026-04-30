import { test, expect } from "../fixtures/api-mock";
import { createSeedRoutinesWithStub } from "../fixtures/seed-data";

test.describe("routines page", () => {
  test("shows empty state when no routines exist", async ({ page }) => {
    await page.goto("/routines");

    await expect(
      page.getByRole("heading", { name: "Routines" })
    ).toBeVisible();
    await expect(page.getByText("No saved routines yet")).toBeVisible();

    // Should have a link to create a session
    await expect(
      page.getByRole("link", { name: "New Session" })
    ).toBeVisible();
  });

  test("displays pre-seeded routine with entries", async ({
    page,
    mockApi,
  }) => {
    // Seed a routine before navigating
    mockApi.routines = createSeedRoutinesWithStub();

    await page.goto("/routines");

    await expect(
      page.getByRole("heading", { name: "Routines" })
    ).toBeVisible();

    // Should show the routine name
    await expect(page.getByText("Morning Warm-up")).toBeVisible();

    // Type-breakdown meta line replaces the previous "N items" badge —
    // STUB_ROUTINE has one piece + one exercise.
    await expect(page.getByText("1 piece · 1 exercise")).toBeVisible();

    // The whole row is a tap target linking to the edit screen — Edit /
    // Delete affordances live in the swipe gesture and long-press
    // context menu, not inline buttons. We verify the link target here;
    // gesture-based actions are validated at the core/event layer.
    const row = page.getByRole("link").filter({ hasText: "Morning Warm-up" });
    await expect(row).toHaveAttribute(
      "href",
      /\/routines\/[A-Z0-9]+\/edit$/
    );
  });

  test("save routine from session builder", async ({ page }) => {
    await page.goto("/sessions/new");

    // Click "Custom Session" to enter the setlist builder
    await page.getByRole("button", { name: "Custom Session" }).click();

    // Add an item to the setlist
    await page.getByText("Clair de Lune").click();

    // Expand the "Save as Routine" form
    await page.getByRole("button", { name: "Save as Routine" }).click();

    // Enter a routine name and save
    const routineNameInput = page.getByPlaceholder("e.g. Morning Warm-up");
    await routineNameInput.fill("My New Routine");
    await page.getByRole("button", { name: "Save" }).click();

    // Verify the routine appears on the routines page
    await page.goto("/routines");
    await expect(page.getByText("My New Routine")).toBeVisible();
  });

  test("load routine into session builder", async ({ page, mockApi }) => {
    mockApi.routines = createSeedRoutinesWithStub();

    await page.goto("/sessions/new");

    // Click "Custom Session" to enter the setlist builder
    await page.getByRole("button", { name: "Custom Session" }).click();

    // Should see the saved routine in the "Saved Routines" section
    await expect(page.getByText("Saved Routines")).toBeVisible();
    await expect(page.getByText("Morning Warm-up")).toBeVisible();

    // Load the routine
    await page.getByRole("button", { name: "Load" }).click();

    // Setlist should now have the routine's entries (items also appear in library list)
    // Check that the Start Session button is enabled (proves items were loaded)
    await expect(
      page.getByRole("button", { name: "Start Session" })
    ).toBeEnabled();
  });
});
