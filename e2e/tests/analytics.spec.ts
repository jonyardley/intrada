import { test, expect } from "../fixtures/api-mock";
import { PracticeSession, SetlistEntry } from "../fixtures/seed-data";

function createCompletedSession(overrides?: Partial<PracticeSession>): PracticeSession {
  const now = new Date();
  const startedAt = new Date(now.getTime() - 30 * 60 * 1000); // 30 min ago
  const entry: SetlistEntry = {
    id: "01JTEST0000000000ENTRY00001",
    item_id: "01JSTUB0000000000PIECE00001",
    item_title: "Clair de Lune",
    item_type: "piece",
    position: 0,
    duration_secs: 1800,
    status: "Completed",
    notes: null,
  };
  return {
    id: "01JTEST0000000000SESN000001",
    entries: [entry],
    session_notes: null,
    started_at: startedAt.toISOString(),
    completed_at: now.toISOString(),
    total_duration_secs: 1800,
    completion_status: "Completed",
    ...overrides,
  };
}

test.describe("analytics page", () => {
  test("shows empty state when no sessions exist", async ({ page }) => {
    await page.goto("/analytics");

    await expect(
      page.getByRole("heading", { name: "Analytics" })
    ).toBeVisible();

    // Empty state
    await expect(page.getByText("No practice data yet")).toBeVisible();
    await expect(
      page.getByRole("link", { name: "Start a Session" })
    ).toBeVisible();
  });

  test("shows stat cards when sessions exist", async ({ page, mockApi }) => {
    // Pre-seed a completed session from today
    mockApi.sessions.push(createCompletedSession());

    await page.goto("/analytics");

    await expect(
      page.getByRole("heading", { name: "Analytics" })
    ).toBeVisible();

    // Stat cards should be present (not the empty state)
    await expect(
      page.getByText("This Week", { exact: true })
    ).toBeVisible();
    await expect(page.getByText("Streak")).toBeVisible();

    // Most Practised section should show the item
    await expect(page.getByText("Most Practised")).toBeVisible();
  });

  test("shows most practised items with session data", async ({
    page,
    mockApi,
  }) => {
    mockApi.sessions.push(createCompletedSession());

    await page.goto("/analytics");

    // The most practised section should list "Clair de Lune"
    await expect(page.getByText("Most Practised")).toBeVisible();
    await expect(page.getByText("Clair de Lune")).toBeVisible();
  });
});
