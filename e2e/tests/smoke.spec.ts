import { test, expect } from "../fixtures/api-mock";

test("app renders with library list", async ({ page }) => {
  await page.goto("/");

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
