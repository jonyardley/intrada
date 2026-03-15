/**
 * Playwright fixture that intercepts all HTTP API requests at the browser
 * level using page.route(). Each test gets an isolated in-memory store
 * pre-seeded with stub data (Clair de Lune + Hanon No. 1).
 *
 * No real API server is needed — requests never leave the browser.
 */

import { test as base, Page } from "@playwright/test";
import {
  Item,
  PracticeSession,
  Routine,
  createSeedItems,
  createSeedRoutines,
} from "./seed-data";

const API_BASE = "https://intrada-api.fly.dev";

let idCounter = 1000;
function generateId(): string {
  idCounter++;
  return `01JTEST${String(idCounter).padStart(19, "0")}`;
}

export interface MockStore {
  items: Item[];
  sessions: PracticeSession[];
  routines: Routine[];
}

/**
 * Inject a Clerk auth mock before the page loads.
 *
 * Stubs `window.__intrada_auth` so the WASM auth gate sees a signed-in user
 * without needing the real Clerk SDK. Also blocks the Clerk CDN script and
 * stubs `window.Clerk` to prevent the real SDK from interfering.
 */
async function setupClerkMock(page: Page) {
  // Block the Clerk CDN script — prevents the real SDK from loading
  await page.route("**/cdn.jsdelivr.net/npm/@clerk/**", (route) =>
    route.fulfill({ status: 200, contentType: "application/javascript", body: "// blocked" })
  );

  // Stub window.__intrada_auth and window.Clerk before the page loads
  await page.addInitScript(() => {
    const listeners: (() => void)[] = [];
    (window as any).__intrada_auth = {
      _clerk: null,
      _ready: true,
      init(_key: string) {
        // no-op in tests
      },
      isSignedIn() {
        return true;
      },
      async getToken() {
        return "fake-test-token";
      },
      getUserId() {
        return "test-user-001";
      },
      async signOut() {
        // no-op in tests
      },
      async signInWithGoogle() {
        // no-op in tests
      },
      addListener(callback: () => void) {
        listeners.push(callback);
      },
    };
    // Stub window.Clerk constructor so the onload handler doesn't fail
    (window as any).Clerk = class {
      async load() {}
    };
  });
}

async function setupApiMock(page: Page, store: MockStore) {
  await page.route(`${API_BASE}/api/**`, async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const method = request.method();
    const path = url.pathname;

    // ---- Items ----
    if (path === "/api/items" && method === "GET") {
      return route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(store.items),
      });
    }

    if (path === "/api/items" && method === "POST") {
      const body = request.postDataJSON();
      const now = new Date().toISOString();
      const item: Item = {
        id: generateId(),
        kind: body.kind,
        title: body.title,
        composer: body.composer ?? null,
        category: body.category ?? null,
        key: body.key ?? null,
        tempo: body.tempo ?? null,
        notes: body.notes ?? null,
        tags: body.tags ?? [],
        created_at: now,
        updated_at: now,
      };
      store.items.push(item);
      return route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify(item),
      });
    }

    const itemMatch = path.match(/^\/api\/items\/(.+)$/);
    if (itemMatch) {
      const id = itemMatch[1];
      if (method === "PUT") {
        const idx = store.items.findIndex((i) => i.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        const body = request.postDataJSON();
        const item = store.items[idx];
        if (body.title !== undefined) item.title = body.title;
        if (body.composer !== undefined) item.composer = body.composer;
        if (body.category !== undefined) item.category = body.category;
        if (body.key !== undefined) item.key = body.key;
        if (body.tempo !== undefined) item.tempo = body.tempo;
        if (body.notes !== undefined) item.notes = body.notes;
        if (body.tags !== undefined) item.tags = body.tags;
        item.updated_at = new Date().toISOString();
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(item),
        });
      }
      if (method === "DELETE") {
        const idx = store.items.findIndex((i) => i.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        store.items.splice(idx, 1);
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ message: "Item deleted" }),
        });
      }
    }

    // ---- Sessions ----
    if (path === "/api/sessions" && method === "GET") {
      return route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(store.sessions),
      });
    }

    if (path === "/api/sessions" && method === "POST") {
      const body = request.postDataJSON();
      store.sessions.push(body);
      return route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify(body),
      });
    }

    const sessionMatch = path.match(/^\/api\/sessions\/(.+)$/);
    if (sessionMatch) {
      const id = sessionMatch[1];
      if (method === "DELETE") {
        const idx = store.sessions.findIndex((s) => s.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        store.sessions.splice(idx, 1);
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ message: "Session deleted" }),
        });
      }
    }

    // ---- Routines ----
    if (path === "/api/routines" && method === "GET") {
      return route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(store.routines),
      });
    }

    if (path === "/api/routines" && method === "POST") {
      const body = request.postDataJSON();
      const now = new Date().toISOString();
      const routine: Routine = {
        id: generateId(),
        name: body.name,
        entries: (body.entries ?? []).map(
          (e: { item_id: string; item_title: string; item_type: string }, i: number) => ({
            id: generateId(),
            item_id: e.item_id,
            item_title: e.item_title,
            item_type: e.item_type,
            position: i,
          })
        ),
        created_at: now,
        updated_at: now,
      };
      store.routines.push(routine);
      return route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify(routine),
      });
    }

    const routineMatch = path.match(/^\/api\/routines\/(.+)$/);
    if (routineMatch) {
      const id = routineMatch[1];
      if (method === "GET") {
        const routine = store.routines.find((r) => r.id === id);
        if (!routine) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(routine),
        });
      }
      if (method === "PUT") {
        const idx = store.routines.findIndex((r) => r.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        const body = request.postDataJSON();
        const routine = store.routines[idx];
        routine.name = body.name;
        routine.entries = (body.entries ?? []).map(
          (e: { item_id: string; item_title: string; item_type: string }, i: number) => ({
            id: generateId(),
            item_id: e.item_id,
            item_title: e.item_title,
            item_type: e.item_type,
            position: i,
          })
        );
        routine.updated_at = new Date().toISOString();
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(routine),
        });
      }
      if (method === "DELETE") {
        const idx = store.routines.findIndex((r) => r.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        store.routines.splice(idx, 1);
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ message: "Routine deleted" }),
        });
      }
    }

    // Unmatched routes
    return route.fulfill({
      status: 404,
      contentType: "application/json",
      body: JSON.stringify({ error: `Not found: ${method} ${path}` }),
    });
  });
}

/**
 * Extended Playwright test with automatic API mocking.
 *
 * Import `test` and `expect` from this module instead of `@playwright/test`.
 * The `mockApi` fixture is set up automatically before each test.
 */
export const test = base.extend<{ mockApi: MockStore }>({
  mockApi: [
    async ({ page }, use) => {
      const store: MockStore = {
        items: createSeedItems(),
        sessions: [],
        routines: createSeedRoutines(),
      };
      await setupClerkMock(page);
      await setupApiMock(page, store);
      await use(store);
    },
    { auto: true },
  ],
});

export { expect } from "@playwright/test";
