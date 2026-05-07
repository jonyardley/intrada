import { test, expect } from "../fixtures/api-mock";
import { createSeedSetsWithStub } from "../fixtures/seed-data";

test.describe("sets in Library", () => {
  test("shows empty state on the Sets tab when no sets exist", async ({
    page,
  }) => {
    // /routines redirects to /?type=set so the Library opens with the
    // Sets tab pre-selected — kept for legacy deep links.
    await page.goto("/?type=set");

    await expect(page.getByRole("heading", { name: "Library" })).toBeVisible();

    // Sets tab is selected via the ?type=set initial filter.
    await expect(page.getByRole("tab", { name: "Sets" })).toHaveAttribute(
      "aria-selected",
      "true"
    );

    await expect(page.getByText("No saved sets yet")).toBeVisible();
  });

  test("displays pre-seeded set on the Sets tab", async ({ page, mockApi }) => {
    mockApi.sets = createSeedSetsWithStub();

    await page.goto("/?type=set");

    await expect(page.getByRole("heading", { name: "Library" })).toBeVisible();
    await expect(page.getByText("Morning Warm-up")).toBeVisible();

    // LibrarySetCard surfaces an item-count label ("N items" / "1 item")
    // — STUB_SET has one piece + one exercise = 2 items.
    await expect(page.getByText("2 items")).toBeVisible();

    // Tap target on the row links to the Set Detail surface, not the
    // edit form (Edit moved to the swipe / context-menu actions).
    const row = page.getByRole("link").filter({ hasText: "Morning Warm-up" });
    await expect(row).toHaveAttribute(
      "href",
      /\/library\/sets\/[A-Z0-9]+$/
    );
  });

  test("legacy /routines URL redirects to the Library Sets tab", async ({
    page,
  }) => {
    await page.goto("/routines");

    // Should land on Library (URL `/?type=set` after the replace
    // navigation, with the Sets tab already selected).
    await expect(page).toHaveURL(/\?type=set$/);
    await expect(page.getByRole("heading", { name: "Library" })).toBeVisible();
    await expect(page.getByRole("tab", { name: "Sets" })).toHaveAttribute(
      "aria-selected",
      "true"
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

    // Sets are now visible on the Library Sets tab; check the saved
    // set surfaces there.
    await page.goto("/?type=set");
    await expect(page.getByText("My New Set")).toBeVisible();
  });

  test("load set into session builder", async ({ page, mockApi }) => {
    mockApi.sets = createSeedSetsWithStub();

    await page.goto("/sessions/new");
    await page.getByRole("button", { name: "Custom Session" }).click();

    await expect(page.getByText("Saved Sets")).toBeVisible();
    await expect(page.getByText("Morning Warm-up")).toBeVisible();
    // Scope by row so the click stays unambiguous if more saved sets are
    // ever seeded — prevents future flakes when the loader has >1 row.
    await page
      .getByRole("listitem")
      .filter({ hasText: "Morning Warm-up" })
      .getByRole("button", { name: "Load" })
      .click();

    await page.getByRole("button", { name: "Review session" }).click();
    await expect(
      page
        .getByRole("dialog")
        .getByRole("button", { name: "Start", exact: true })
    ).toBeEnabled();
  });
});
