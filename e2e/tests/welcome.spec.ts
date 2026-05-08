import { test, expect } from "../fixtures/api-mock";

test.describe("welcome carousel", () => {
  test("shows welcome on first visit, tap through to CTA", async ({ page }) => {
    // Ensure localStorage is clean (no welcome-seen flag)
    await page.goto("/");

    // Carousel should be visible
    const carousel = page.getByRole("region", { name: "Welcome" });
    await expect(carousel).toBeVisible();

    // Card 1 content
    await expect(
      page.getByText("Knowing how to practise well is hard")
    ).toBeVisible();

    // Tap to advance through cards 2-4
    // Click in the centre of the viewport to avoid the Skip button (top-right)
    // and the progress dots (bottom). The carousel overlay is fixed inset-0.
    await carousel.click({ position: { x: 200, y: 400 } });
    await page.waitForTimeout(150); // allow 50ms fade-out + render swap
    await expect(
      page.getByText("Build a library of pieces and exercises")
    ).toBeVisible();

    await carousel.click({ position: { x: 200, y: 400 } });
    await page.waitForTimeout(150);
    await expect(
      page.getByText("Plan each session with intention")
    ).toBeVisible();

    await carousel.click({ position: { x: 200, y: 400 } });
    await page.waitForTimeout(150);
    await expect(
      page.getByText("Run focused, timed sessions")
    ).toBeVisible();

    await carousel.click({ position: { x: 200, y: 400 } });
    await page.waitForTimeout(150);
    // Card 5 — final card with CTA
    await expect(
      page.getByText("Track your progress, achieve your goals")
    ).toBeVisible();

    // Use regex to match the button regardless of the → Unicode arrow suffix
    const cta = page.getByRole("button", { name: /Add your first piece/ });
    await expect(cta).toBeVisible();

    // Click CTA — should navigate to /library/new
    await cta.click();
    await expect(carousel).not.toBeVisible();
    await expect(page).toHaveURL(/\/library\/new/);
  });

  test("skip dismisses carousel and lands on library", async ({ page }) => {
    await page.goto("/");

    const carousel = page.getByRole("region", { name: "Welcome" });
    await expect(carousel).toBeVisible();

    // Click Skip
    await page.getByRole("button", { name: "Skip" }).click();

    await expect(carousel).not.toBeVisible();
    // Should be on the library page (root)
    await expect(page).toHaveURL(/\/$/);

    // Reload — carousel should NOT reappear (localStorage flag set)
    await page.reload();
    await expect(
      page.getByRole("region", { name: "Welcome" })
    ).not.toBeVisible();
  });

  test("does not show welcome when localStorage flag is set", async ({
    page,
  }) => {
    // Prime localStorage before navigating
    await page.addInitScript(() => {
      localStorage.setItem("intrada:welcome-seen", "1");
    });

    await page.goto("/");

    // Carousel should not appear
    await expect(
      page.getByRole("region", { name: "Welcome" })
    ).not.toBeVisible();

    // Library content should be visible
    await expect(page.getByRole("main")).toBeVisible();
  });
});
