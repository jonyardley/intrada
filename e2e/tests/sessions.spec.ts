import { test, expect } from "../fixtures/api-mock";

test.describe("sessions page", () => {
  test("shows empty state when no sessions exist", async ({ page }) => {
    await page.goto("/sessions");

    await expect(
      page.getByRole("heading", { name: "Practice" })
    ).toBeVisible();

    // Week strip is visible with day cells
    await expect(page.getByText("No sessions on this day")).toBeVisible();

    // Empty state CTA — scope to the empty-state container so we don't
    // pick up the page-header "+" New Session action which has the same
    // accessible name.
    const emptyState = page.locator(".empty-state");
    await expect(
      emptyState.getByRole("link", { name: "New Session" })
    ).toBeVisible();

    // Should have a "Show all sessions" link
    await expect(
      page.getByRole("link", { name: "Show all sessions →" })
    ).toBeVisible();
  });

  test("create a session via the setlist flow", async ({ page }) => {
    await page.goto("/sessions");

    // Click the page-header "New Session" CTA to go to the setlist builder.
    // Use the aria-label to disambiguate from the empty-state CTA which
    // shares the same accessible name.
    await page.getByLabel("New Session").click();

    // Should see the preset selection page
    await expect(
      page.getByRole("heading", { name: "New Session" })
    ).toBeVisible();

    // Click "Custom Session" to enter the setlist builder
    await page.getByRole("button", { name: "Custom Session" }).click();
    await expect(page.getByPlaceholder("Search library...")).toBeVisible();

    // Add "Clair de Lune" — tap the library row toggles selection
    await page.getByText("Clair de Lune").click();

    // Open the review sheet, then start the session from inside it
    await page.getByRole("button", { name: "Review session" }).click();
    const reviewSheet = page.getByRole("dialog");
    await reviewSheet
      .getByRole("button", { name: "Start Session" })
      .click();

    // Should be on the active session page with the timer
    // (Focus mode hides the heading, so check item indicator instead)
    await expect(page.getByText("Item 1 of 1")).toBeVisible();

    // Finish the session (single item = "Finish Session" button)
    await page.getByRole("button", { name: "Finish Session" }).click();

    // Should be on the summary page
    await expect(page.getByText("Session Complete!")).toBeVisible();

    // Save the session
    await page.getByRole("button", { name: "Save Session" }).click();

    // Should redirect to sessions list with the new session
    await expect(
      page.getByRole("heading", { name: "Practice" })
    ).toBeVisible();
    await expect(page.getByText("1 session", { exact: true })).toBeVisible();
  });

  test("multi-item session with skip", async ({ page }) => {
    await page.goto("/sessions/new");

    // Click "Custom Session" to enter the setlist builder
    await page.getByRole("button", { name: "Custom Session" }).click();

    // Add both items to the setlist
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Open the review sheet and start the session from inside it
    await page.getByRole("button", { name: "Review session" }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Start Session" })
      .click();

    // Should show first item
    await expect(page.getByText("Item 1 of 2")).toBeVisible();

    // Skip the first item
    await page.getByRole("button", { name: "Skip" }).click();

    // Should advance to second item
    await expect(page.getByText("Item 2 of 2")).toBeVisible();

    // Finish the session (last item)
    await page.getByRole("button", { name: "Finish Session" }).click();

    // Summary should show both items
    await expect(page.getByText("Session Complete!")).toBeVisible();
    await expect(page.getByText("Items Practiced")).toBeVisible();
  });

  test("cancel building returns to sessions list", async ({ page }) => {
    await page.goto("/sessions/new");

    // Click "Custom Session" to enter the setlist builder
    await page.getByRole("button", { name: "Custom Session" }).click();

    // Should be on the builder
    await expect(
      page.getByRole("heading", { name: "New Session" })
    ).toBeVisible();

    // Click Cancel — scoped to <main> to avoid matching the (closed but
    // still mounted) bottom sheet's own Cancel button.
    await page
      .getByRole("main")
      .getByRole("button", { name: "Cancel" })
      .click();

    // Should redirect to sessions list
    await expect(
      page.getByRole("heading", { name: "Practice" })
    ).toBeVisible();
  });

  test("multi-item session with Next Item navigation", async ({ page }) => {
    await page.goto("/sessions/new");

    // Click "Custom Session" to enter the setlist builder
    await page.getByRole("button", { name: "Custom Session" }).click();

    // Add both items
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Open the review sheet and start the session from inside it
    await page.getByRole("button", { name: "Review session" }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Start Session" })
      .click();

    // First item
    await expect(page.getByText("Item 1 of 2")).toBeVisible();

    // Next Item (not last, so button says "Next Item")
    await page.getByRole("button", { name: "Next Item" }).click();

    // Second item
    await expect(page.getByText("Item 2 of 2")).toBeVisible();

    // Now it's the last item, so button says "Finish Session"
    await page.getByRole("button", { name: "Finish Session" }).click();

    // Summary
    await expect(page.getByText("Session Complete!")).toBeVisible();

    // Save
    await page.getByRole("button", { name: "Save Session" }).click();
    await expect(
      page.getByRole("heading", { name: "Practice" })
    ).toBeVisible();
    await expect(page.getByText("1 session", { exact: true })).toBeVisible();
  });

  test("end session early", async ({ page }) => {
    await page.goto("/sessions/new");

    // Click "Custom Session" to enter the setlist builder
    await page.getByRole("button", { name: "Custom Session" }).click();

    // Add both items
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Open the review sheet and start the session from inside it
    await page.getByRole("button", { name: "Review session" }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Start Session" })
      .click();
    await expect(page.getByText("Item 1 of 2")).toBeVisible();

    // End early
    await page.getByRole("button", { name: "End Early" }).click();

    // Should see summary with "Ended Early" indicator
    await expect(page.getByText("Session Complete!")).toBeVisible();
    await expect(page.getByText("Ended Early")).toBeVisible();
  });
});
