import { test, expect } from "../fixtures/api-mock";

test.describe("sessions page", () => {
  test("shows empty state when no sessions exist", async ({ page }) => {
    await page.goto("/sessions");

    await expect(
      page.getByRole("heading", { name: "Practice Sessions" })
    ).toBeVisible();
    await expect(
      page.getByText("No practice sessions recorded yet.")
    ).toBeVisible();

    // Should have a "New Session" link
    await expect(
      page.getByRole("link", { name: "New Session" })
    ).toBeVisible();
  });

  test("create a session via the setlist flow", async ({ page }) => {
    await page.goto("/sessions");

    // Click "New Session" to go to the setlist builder
    await page.getByRole("link", { name: "New Session" }).click();

    // Should see the setlist builder page
    await expect(
      page.getByRole("heading", { name: "New Practice Session" })
    ).toBeVisible();
    await expect(page.getByText("Your Setlist")).toBeVisible();

    // Add "Clair de Lune" from the library items list
    // (026-drag-drop-builder: whole library row is now the click target)
    await page.getByText("Clair de Lune").click();

    // Start the session
    await page.getByRole("button", { name: "Start Session" }).click();

    // Should be on the active session page with the timer
    await expect(
      page.getByRole("heading", { name: "Practice Session" })
    ).toBeVisible();
    await expect(page.getByText("Item 1 of 1")).toBeVisible();

    // Finish the session (single item = "Finish Session" button)
    await page.getByRole("button", { name: "Finish Session" }).click();

    // Should be on the summary page
    await expect(page.getByText("Session Complete!")).toBeVisible();

    // Save the session
    await page.getByRole("button", { name: "Save Session" }).click();

    // Should redirect to sessions list with the new session
    await expect(
      page.getByRole("heading", { name: "Practice Sessions" })
    ).toBeVisible();
    await expect(page.getByText("1 session", { exact: true })).toBeVisible();
  });

  test("multi-item session with skip", async ({ page }) => {
    await page.goto("/sessions/new");

    // Add both items to the setlist
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Start the session
    await page.getByRole("button", { name: "Start Session" }).click();

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

    // Should be on the builder
    await expect(
      page.getByRole("heading", { name: "New Practice Session" })
    ).toBeVisible();

    // Click Cancel
    await page.getByRole("button", { name: "Cancel" }).click();

    // Should redirect to sessions list
    await expect(
      page.getByRole("heading", { name: "Practice Sessions" })
    ).toBeVisible();
  });

  test("multi-item session with Next Item navigation", async ({ page }) => {
    await page.goto("/sessions/new");

    // Add both items
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Start session
    await page.getByRole("button", { name: "Start Session" }).click();

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
      page.getByRole("heading", { name: "Practice Sessions" })
    ).toBeVisible();
    await expect(page.getByText("1 session", { exact: true })).toBeVisible();
  });

  test("end session early", async ({ page }) => {
    await page.goto("/sessions/new");

    // Add both items
    await page.getByText("Clair de Lune").click();
    await page.getByText("Hanon No. 1").click();

    // Start session
    await page.getByRole("button", { name: "Start Session" }).click();
    await expect(page.getByText("Item 1 of 2")).toBeVisible();

    // End early
    await page.getByRole("button", { name: "End Early" }).click();

    // Should see summary with "Ended Early" indicator
    await expect(page.getByText("Session Complete!")).toBeVisible();
    await expect(page.getByText("Ended Early")).toBeVisible();
  });
});
