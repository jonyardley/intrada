import { test, expect } from "../fixtures/api-mock";

test.describe("sessions page", () => {
  test("shows empty state when no sessions exist", async ({ page }) => {
    await page.goto("/sessions");

    await expect(
      page.getByRole("heading", { name: "Practice" })
    ).toBeVisible();

    // Week strip is visible with day cells
    await expect(page.getByText("No practices on this day")).toBeVisible();

    // Should have a "New Practice" link
    await expect(
      page.getByRole("link", { name: "New Practice" })
    ).toBeVisible();

    // Should have a "Show all practices" link
    await expect(
      page.getByRole("link", { name: "Show all practices →" })
    ).toBeVisible();
  });

  test("create a session via the setlist flow", async ({ page }) => {
    await page.goto("/sessions");

    // Click "New Practice" to go to the setlist builder
    await page.getByRole("link", { name: "New Practice" }).click();

    // Should see the setlist builder page
    await expect(
      page.getByRole("heading", { name: "New Practice" })
    ).toBeVisible();
    await expect(page.getByText("Your Setlist")).toBeVisible();

    // Add "Clair de Lune" from the library items list
    // (026-drag-drop-builder: whole library row is now the click target)
    await page.getByText("Clair de Lune").click();

    // Start the practice
    await page.getByRole("button", { name: "Start Practice" }).click();

    // Should be on the active practice page with the timer
    // (Focus mode hides the "Practice" heading, so check item indicator instead)
    await expect(page.getByText("Item 1 of 1")).toBeVisible();

    // Finish the practice (single item = "Finish Practice" button)
    await page.getByRole("button", { name: "Finish Practice" }).click();

    // Should be on the summary page
    await expect(page.getByText("Practice Complete!")).toBeVisible();

    // Save the practice
    await page.getByRole("button", { name: "Save Practice" }).click();

    // Should redirect to practice list with the new practice
    await expect(
      page.getByRole("heading", { name: "Practice" })
    ).toBeVisible();
    await expect(page.getByText("1 practice", { exact: true })).toBeVisible();
  });

  test("multi-item session with skip", async ({ page }) => {
    await page.goto("/sessions/new");

    // Add both items to the setlist
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Start the practice
    await page.getByRole("button", { name: "Start Practice" }).click();

    // Should show first item
    await expect(page.getByText("Item 1 of 2")).toBeVisible();

    // Skip the first item
    await page.getByRole("button", { name: "Skip" }).click();

    // Should advance to second item
    await expect(page.getByText("Item 2 of 2")).toBeVisible();

    // Finish the practice (last item)
    await page.getByRole("button", { name: "Finish Practice" }).click();

    // Summary should show both items
    await expect(page.getByText("Practice Complete!")).toBeVisible();
    await expect(page.getByText("Items Practiced")).toBeVisible();
  });

  test("cancel building returns to sessions list", async ({ page }) => {
    await page.goto("/sessions/new");

    // Should be on the builder
    await expect(
      page.getByRole("heading", { name: "New Practice" })
    ).toBeVisible();

    // Click Cancel
    await page.getByRole("button", { name: "Cancel" }).click();

    // Should redirect to practice list
    await expect(
      page.getByRole("heading", { name: "Practice" })
    ).toBeVisible();
  });

  test("multi-item session with Next Item navigation", async ({ page }) => {
    await page.goto("/sessions/new");

    // Add both items
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Start practice
    await page.getByRole("button", { name: "Start Practice" }).click();

    // First item
    await expect(page.getByText("Item 1 of 2")).toBeVisible();

    // Next Item (not last, so button says "Next Item")
    await page.getByRole("button", { name: "Next Item" }).click();

    // Second item
    await expect(page.getByText("Item 2 of 2")).toBeVisible();

    // Now it's the last item, so button says "Finish Practice"
    await page.getByRole("button", { name: "Finish Practice" }).click();

    // Summary
    await expect(page.getByText("Practice Complete!")).toBeVisible();

    // Save
    await page.getByRole("button", { name: "Save Practice" }).click();
    await expect(
      page.getByRole("heading", { name: "Practice" })
    ).toBeVisible();
    await expect(page.getByText("1 practice", { exact: true })).toBeVisible();
  });

  test("end session early", async ({ page }) => {
    await page.goto("/sessions/new");

    // Add both items
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Start practice
    await page.getByRole("button", { name: "Start Practice" }).click();
    await expect(page.getByText("Item 1 of 2")).toBeVisible();

    // End early
    await page.getByRole("button", { name: "End Early" }).click();

    // Should see summary with "Ended Early" indicator
    await expect(page.getByText("Practice Complete!")).toBeVisible();
    await expect(page.getByText("Ended Early")).toBeVisible();
  });
});
