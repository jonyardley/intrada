import { test, expect } from "../fixtures/api-mock";

test("app renders with library list", async ({ page }) => {
  await page.goto("/");

  // Verify page heading is visible
  await expect(
    page.getByRole("heading", { name: "Library" })
  ).toBeVisible();

  // Verify library list renders. Library defaults to the Pieces tab; stub
  // data is 1 piece + 1 exercise, so 1 row visible until tabs swap.
  await expect(
    page.getByRole("list", { name: "Library items" })
  ).toBeVisible();
  const items = page
    .getByRole("list", { name: "Library items" })
    .locator("li");
  await expect(items).toHaveCount(1); // stub piece (Clair de Lune)

  // Swap to Exercises — should show the stub exercise (Hanon No. 1).
  await page.getByRole("tab", { name: "Exercises" }).click();
  await expect(items).toHaveCount(1);
  await expect(items.first()).toContainText("Hanon No. 1");
});
