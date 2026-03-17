import { test, expect } from "../fixtures/api-mock";
import { createSeedRoutinesWithStub } from "../fixtures/seed-data";

test.describe("routines page", () => {
  test("shows empty state when no routines exist", async ({ page }) => {
    await page.goto("/routines");

    await expect(
      page.getByRole("heading", { name: "Routines" })
    ).toBeVisible();
    await expect(page.getByText("No saved routines yet.")).toBeVisible();

    // Should have a link to create a practice
    await expect(
      page.getByRole("link", { name: "New Practice" })
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

    // Should show the item count badge
    await expect(page.getByText("2 items")).toBeVisible();

    // Should show Edit and Delete controls
    await expect(page.getByRole("link", { name: "Edit" })).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Delete" })
    ).toBeVisible();
  });

  test("delete routine with confirmation", async ({ page, mockApi }) => {
    mockApi.routines = createSeedRoutinesWithStub();

    await page.goto("/routines");
    await expect(page.getByText("Morning Warm-up")).toBeVisible();

    // Click Delete — should show confirmation
    await page.getByRole("button", { name: "Delete" }).click();
    await expect(
      page.getByText("Delete this routine? This cannot be undone.")
    ).toBeVisible();

    // Cancel — should dismiss
    await page.getByRole("button", { name: "Cancel" }).click();
    await expect(
      page.getByText("Delete this routine? This cannot be undone.")
    ).not.toBeVisible();

    // Delete again and confirm
    await page.getByRole("button", { name: "Delete" }).click();
    await page.getByRole("button", { name: "Confirm Delete" }).click();

    // Should show empty state
    await expect(page.getByText("No saved routines yet.")).toBeVisible();
  });

  test("save routine from session builder", async ({ page }) => {
    await page.goto("/sessions/new");

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

    // Should see the saved routine in the "Saved Routines" section
    await expect(page.getByText("Saved Routines")).toBeVisible();
    await expect(page.getByText("Morning Warm-up")).toBeVisible();

    // Load the routine
    await page.getByRole("button", { name: "Load" }).click();

    // Setlist should now have the routine's entries (items also appear in library list)
    // Check that the Start Practice button is enabled (proves items were loaded)
    await expect(
      page.getByRole("button", { name: "Start Practice" })
    ).toBeEnabled();
  });
});
