import { test, expect } from "../fixtures/api-mock";
import { createSeedSetsWithStub } from "../fixtures/seed-data";

test.describe("sets page", () => {
  test("shows empty state when no sets exist", async ({ page }) => {
    await page.goto("/routines");

    await expect(
      page.getByRole("heading", { name: "Sets" })
    ).toBeVisible();
    await expect(page.getByText("No saved sets yet")).toBeVisible();

    // Should have a link to create a session
    await expect(
      page.getByRole("link", { name: "New Session" })
    ).toBeVisible();
  });

  test("displays pre-seeded set with entries", async ({
    page,
    mockApi,
  }) => {
    // Seed a set before navigating
    mockApi.sets = createSeedSetsWithStub();

    await page.goto("/routines");

    await expect(
      page.getByRole("heading", { name: "Sets" })
    ).toBeVisible();

    // Should show the set name
    await expect(page.getByText("Morning Warm-up")).toBeVisible();

    // Type-breakdown meta line replaces the previous "N items" badge —
    // STUB_SET has one piece + one exercise.
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

  test("save set from session builder", async ({ page }) => {
    await page.goto("/sessions/new");
    await page.getByRole("button", { name: "Custom Session" }).click();
    await page.getByText("Clair de Lune").click();
    await page.getByRole("button", { name: "Review session" }).click();
    const reviewSheet = page.getByRole("dialog");
    await reviewSheet.getByRole("button", { name: "Save as Set" }).click();
    await reviewSheet.getByPlaceholder("e.g. Morning Warm-up").fill("My New Set");
    await reviewSheet.getByRole("button", { name: "Save" }).click();

    // TODO(#390 pt 2): switch to /library once the URL migration drops
    // /routines and serves Sets from the Library Sets tab.
    await page.goto("/routines");
    await expect(page.getByText("My New Set")).toBeVisible();
  });

  // Skipped: "Load set" UI was removed from the builder during #388
  // and the strip-back kept it out — see #390. The SetLoader component
  // is still in the module tree pending the sets revisit.
  test.skip("load set into session builder", async ({ page, mockApi }) => {
    mockApi.sets = createSeedSetsWithStub();

    await page.goto("/sessions/new");
    await page.getByRole("button", { name: "Custom Session" }).click();

    await expect(page.getByText("Saved Sets")).toBeVisible();
    await expect(page.getByText("Morning Warm-up")).toBeVisible();
    await page.getByRole("button", { name: "Load" }).click();

    await page.getByRole("button", { name: "Review session" }).click();
    await expect(
      page
        .getByRole("dialog")
        .getByRole("button", { name: "Start", exact: true })
    ).toBeEnabled();
  });
});
