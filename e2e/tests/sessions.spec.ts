import { test, expect } from "@playwright/test";

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
    const addButtons = page.getByRole("button", { name: "+ Add" });
    await addButtons.first().click();

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
});
