import { test, expect } from "@playwright/test";

test.describe("sessions page", () => {
  test("shows empty state when no sessions exist", async ({ page }) => {
    await page.goto("/sessions");

    await expect(
      page.getByRole("heading", { name: "Practice Sessions" })
    ).toBeVisible();
    await expect(
      page.getByText("No practice sessions logged yet.")
    ).toBeVisible();
  });

  test("shows sessions after logging one", async ({ page }) => {
    await page.goto("/");

    // Log a session on Clair de Lune
    await page.getByRole("heading", { name: "Clair de Lune" }).click();
    await page.getByRole("button", { name: "Log Session" }).click();
    await page.locator("#log-duration").fill("15");
    await page.getByRole("button", { name: "Save" }).click();

    // Navigate to sessions page
    await page.getByRole("link", { name: "Sessions" }).click();

    // Should show the session
    await expect(
      page.getByRole("heading", { name: "Practice Sessions" })
    ).toBeVisible();
    await expect(page.getByText("15 min")).toBeVisible();
    await expect(page.getByText("Clair de Lune")).toBeVisible();

    // Session count footer
    await expect(page.getByText("1 session(s)")).toBeVisible();
  });
});
