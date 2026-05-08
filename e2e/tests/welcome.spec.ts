import { test, expect } from "../fixtures/api-mock";

test.describe("welcome carousel", () => {
  test("shows welcome on first visit, tap through to CTA", async ({ page }) => {
    // The shared fixture primes intrada:welcome-seen by default so other
    // tests don't see the carousel. Clear it here so we DO see it.
    // This addInitScript is registered AFTER the fixture's, so it runs
    // last on every page load and wins.
    await page.addInitScript(() =>
      localStorage.removeItem("intrada:welcome-seen")
    );
    await page.goto("/");

    // Carousel should be visible
    const carousel = page.getByRole("region", { name: "Welcome" });
    await expect(carousel).toBeVisible();

    // Card 1 content
    await expect(
      page.getByText("Knowing how to practise well is hard")
    ).toBeVisible();

    // Tap to advance through cards 2-4. Each card now has a distinct
    // anchor phrase (the bold serif heading) — the test matches that
    // since it's the most reliable per-card signal.
    // Click in the centre of the viewport to avoid the Skip button (top-right)
    // and the progress dots (bottom). The carousel overlay is fixed inset-0.
    await carousel.click({ position: { x: 200, y: 400 } });
    await page.waitForTimeout(150); // allow 50ms fade-out + render swap
    // Card 2 anchor — "Build a library"
    await expect(page.getByText("Build a library")).toBeVisible();

    await carousel.click({ position: { x: 200, y: 400 } });
    await page.waitForTimeout(150);
    // Card 3 anchor — "Practise with intention"
    await expect(page.getByText("Practise with intention")).toBeVisible();

    await carousel.click({ position: { x: 200, y: 400 } });
    await page.waitForTimeout(150);
    // Card 4 anchor — "Focus, reflect, repeat"
    await expect(page.getByText("Focus, reflect, repeat")).toBeVisible();

    await carousel.click({ position: { x: 200, y: 400 } });
    await page.waitForTimeout(150);
    // Card 5 anchor — "Watch your progress"
    await expect(page.getByText("Watch your progress")).toBeVisible();

    // Use regex to match the button regardless of the → Unicode arrow suffix
    const cta = page.getByRole("button", { name: /Get started/ });
    await expect(cta).toBeVisible();

    // Click CTA — should navigate to /library (the Library home)
    await cta.click();
    await expect(carousel).not.toBeVisible();
    await expect(page).toHaveURL(/\/library$/);
  });

  test("skip dismisses carousel and lands on library", async ({ page }) => {
    // Clear the fixture's primed flag so we see the carousel.
    await page.addInitScript(() =>
      localStorage.removeItem("intrada:welcome-seen")
    );
    await page.goto("/");

    const carousel = page.getByRole("region", { name: "Welcome" });
    await expect(carousel).toBeVisible();

    // Click Skip
    await page.getByRole("button", { name: "Skip" }).click();

    await expect(carousel).not.toBeVisible();
    // Should be on the library page
    await expect(page).toHaveURL(/\/library$/);

    // Note: reload-persistence is covered separately by the next test.
    // We can't easily test it here because the addInitScript above runs
    // on every page load and would re-clear the flag after Skip wrote it.
  });

  test("does not show welcome when localStorage flag is set", async ({
    page,
  }) => {
    // No clearing needed — the shared fixture primes the flag by default,
    // simulating a returning user / second-visit.
    // `/` is the public marketing page; authed users redirect to /library
    // immediately, where the carousel would mount if welcome-seen=false.
    await page.goto("/");
    await expect(page).toHaveURL(/\/library$/);

    // Carousel should not appear
    await expect(
      page.getByRole("region", { name: "Welcome" })
    ).not.toBeVisible();

    // Library content should be visible
    await expect(page.getByRole("main")).toBeVisible();
  });
});
