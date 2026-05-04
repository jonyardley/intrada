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

    // Open the review sheet — Save as Routine + Start Session live inside it
    await page.getByRole("button", { name: "Review session" }).click();
    const reviewSheet = page.getByRole("dialog");

    // Expand the "Save as Routine" form
    await reviewSheet.getByRole("button", { name: "Save as Routine" }).click();

    // Enter a routine name and save
    await reviewSheet.getByPlaceholder("e.g. Morning Warm-up").fill("My New Routine");
    await reviewSheet.getByRole("button", { name: "Save" }).click();

    // Verify the routine appears on the routines page
    await page.goto("/routines");
    await expect(page.getByText("My New Routine")).toBeVisible();
  });

  // Skipped: "Load routine" UI was intentionally removed from the new
  // builder for #388 ("Routines load is out of scope here per design
  // conversation"). The RoutineLoader component is still in the module
  // tree pending the planned routines revisit — re-enable this test when
  // that lands.
  test.skip("load routine into session builder", async ({ page, mockApi }) => {
    mockApi.routines = createSeedRoutinesWithStub();

    await page.goto("/sessions/new");
    await page.getByRole("button", { name: "Custom Session" }).click();

    await expect(page.getByText("Saved Routines")).toBeVisible();
    await expect(page.getByText("Morning Warm-up")).toBeVisible();
    await page.getByRole("button", { name: "Load" }).click();

    await page.getByRole("button", { name: "Review session" }).click();
    await expect(
      page
        .getByRole("dialog")
        .getByRole("button", { name: "Start Session" })
    ).toBeEnabled();
  });
});
