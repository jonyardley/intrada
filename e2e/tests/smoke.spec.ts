import { test, expect } from "../fixtures/api-mock";

test("app renders with library list", async ({ page }) => {
  // Visit /library directly. `/` is now the public marketing page; an
  // authed user redirects from there to /library, but the brief overlap
  // window includes a faux LibraryMock h3 ("Library") inside the marketing
  // hero that races the real page-title h1 and trips strict-mode
  // duplicate-locator detection.
  await page.goto("/library");

  // Verify page heading is visible
  await expect(
    page.getByRole("heading", { name: "Library" })
  ).toBeVisible();

  // Verify library list renders with stub data items (All tab is the
  // default — both stub items visible without switching tabs).
  await expect(
    page.getByRole("list", { name: "Library items" })
  ).toBeVisible();
  const items = page
    .getByRole("list", { name: "Library items" })
    .locator("li");
  await expect(items).toHaveCount(2); // stub: Clair de Lune + Hanon No. 1
});
