/**
 * Playwright fixture that intercepts all HTTP API requests at the browser
 * level using page.route(). Each test gets an isolated in-memory store
 * pre-seeded with stub data (Clair de Lune + Hanon No. 1).
 *
 * No real API server is needed — requests never leave the browser.
 */

import { test as base, Page } from "@playwright/test";
import {
  Piece,
  Exercise,
  PracticeSession,
  createSeedPieces,
  createSeedExercises,
} from "./seed-data";

const API_BASE = "https://intrada-api.fly.dev";

let idCounter = 1000;
function generateId(): string {
  idCounter++;
  return `01JTEST${String(idCounter).padStart(19, "0")}`;
}

export interface MockStore {
  pieces: Piece[];
  exercises: Exercise[];
  sessions: PracticeSession[];
}

async function setupApiMock(page: Page, store: MockStore) {
  await page.route(`${API_BASE}/api/**`, async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const method = request.method();
    const path = url.pathname;

    // ---- Pieces ----
    if (path === "/api/pieces" && method === "GET") {
      return route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(store.pieces),
      });
    }

    if (path === "/api/pieces" && method === "POST") {
      const body = request.postDataJSON();
      const now = new Date().toISOString();
      const piece: Piece = {
        id: generateId(),
        title: body.title,
        composer: body.composer,
        key: body.key ?? null,
        tempo: body.tempo ?? null,
        notes: body.notes ?? null,
        tags: body.tags ?? [],
        created_at: now,
        updated_at: now,
      };
      store.pieces.push(piece);
      return route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify(piece),
      });
    }

    const pieceMatch = path.match(/^\/api\/pieces\/(.+)$/);
    if (pieceMatch) {
      const id = pieceMatch[1];
      if (method === "PUT") {
        const idx = store.pieces.findIndex((p) => p.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        const body = request.postDataJSON();
        const piece = store.pieces[idx];
        if (body.title !== undefined) piece.title = body.title;
        if (body.composer !== undefined) piece.composer = body.composer;
        if (body.key !== undefined) piece.key = body.key;
        if (body.tempo !== undefined) piece.tempo = body.tempo;
        if (body.notes !== undefined) piece.notes = body.notes;
        if (body.tags !== undefined) piece.tags = body.tags;
        piece.updated_at = new Date().toISOString();
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(piece),
        });
      }
      if (method === "DELETE") {
        const idx = store.pieces.findIndex((p) => p.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        store.pieces.splice(idx, 1);
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ message: "Piece deleted" }),
        });
      }
    }

    // ---- Exercises ----
    if (path === "/api/exercises" && method === "GET") {
      return route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(store.exercises),
      });
    }

    if (path === "/api/exercises" && method === "POST") {
      const body = request.postDataJSON();
      const now = new Date().toISOString();
      const exercise: Exercise = {
        id: generateId(),
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
      store.exercises.push(exercise);
      return route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify(exercise),
      });
    }

    const exerciseMatch = path.match(/^\/api\/exercises\/(.+)$/);
    if (exerciseMatch) {
      const id = exerciseMatch[1];
      if (method === "PUT") {
        const idx = store.exercises.findIndex((e) => e.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        const body = request.postDataJSON();
        const exercise = store.exercises[idx];
        if (body.title !== undefined) exercise.title = body.title;
        if (body.composer !== undefined) exercise.composer = body.composer;
        if (body.category !== undefined) exercise.category = body.category;
        if (body.key !== undefined) exercise.key = body.key;
        if (body.tempo !== undefined) exercise.tempo = body.tempo;
        if (body.notes !== undefined) exercise.notes = body.notes;
        if (body.tags !== undefined) exercise.tags = body.tags;
        exercise.updated_at = new Date().toISOString();
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify(exercise),
        });
      }
      if (method === "DELETE") {
        const idx = store.exercises.findIndex((e) => e.id === id);
        if (idx === -1) {
          return route.fulfill({
            status: 404,
            contentType: "application/json",
            body: JSON.stringify({ error: "Not found" }),
          });
        }
        store.exercises.splice(idx, 1);
        return route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({ message: "Exercise deleted" }),
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
        pieces: createSeedPieces(),
        exercises: createSeedExercises(),
        sessions: [],
      };
      await setupApiMock(page, store);
      await use(store);
    },
    { auto: true },
  ],
});

export { expect } from "@playwright/test";
