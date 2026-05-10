// Deploy smoke test — runs against the post-build-modification dist
// (the artifact wrangler would actually publish), not the pristine
// trunk-build artifact.
//
// Catches the class of regression that broke production on 2026-05-09:
// `sentry-cli sourcemaps inject` modified JS files in place; Trunk's
// pre-computed SRI integrity hashes in index.html went stale; browser
// blocked every modified file; page rendered blank.
//
// The existing E2E suite tests app behaviour against the pristine
// dist. This spec specifically guards the deploy pipeline — it only
// fires alarms when SOMETHING breaks runtime loading. Test name is
// `smoke:` prefixed so the dedicated CI job can target it via
// `--grep "smoke:"`.

import { test, expect } from "../fixtures/api-mock";

test("smoke: post-modification dist renders without runtime-blocking console errors", async ({
  page,
}) => {
  // Capture console errors + uncaught page errors. We're looking for
  // the specific signatures that mean "the runtime is broken at the
  // load layer" — not arbitrary app-level errors (those are the job
  // of the rest of the E2E suite).
  const errors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(msg.text());
  });
  page.on("pageerror", (err) => errors.push(`pageerror: ${err.message}`));

  await page.goto("/");

  // Wait for the WASM to mount and put SOMETHING substantial in the
  // body. We deliberately don't pin to a specific element — the
  // marketing page, login screen, library, etc. all evolve, and
  // pinning a selector here would fail for unrelated UI changes.
  // The smoke test cares about ONE thing: did the runtime load at
  // all? If WASM was blocked (SRI mismatch, ERR_BLOCKED_BY_CLIENT,
  // missing import), `document.body.innerText` stays near-empty.
  // Empirical floor: pristine builds put >100 chars in the body
  // within a few seconds; a blank-page failure mode leaves under 30.
  await expect
    .poll(
      async () =>
        await page.evaluate(() =>
          (document.body.innerText || "").trim().length,
        ),
      {
        timeout: 20_000,
        message:
          "body.innerText stayed near-empty — WASM likely failed to load (see console errors below)",
      },
    )
    .toBeGreaterThan(100);

  // Filter to the error classes that indicate broken-runtime, not
  // app-level bugs:
  // - "Failed to find a valid digest"  →  SRI mismatch (PR #594 case)
  // - "ERR_BLOCKED_BY_CLIENT"          →  ad-blocker, content-blocker
  // - "Importing binding name"         →  WASM ↔ JS shim drift (#440)
  // - "module is not an object"        →  WASM imports point at wrong path (#488)
  // - "WebAssembly.instantiate"        →  WASM init failure
  // - "Failed to fetch dynamically imported module" → broken module path
  const blockingPatterns = [
    /Failed to find a valid digest/,
    /ERR_BLOCKED_BY_CLIENT/,
    /Importing binding name/,
    /module is not an object/,
    /WebAssembly\.instantiate/,
    /Failed to fetch dynamically imported module/,
  ];
  const blocking = errors.filter((e) =>
    blockingPatterns.some((re) => re.test(e)),
  );

  expect(
    blocking,
    "Console errors that would block runtime — see deploy modification chain in ci.yml:\n" +
      blocking.map((e) => `  ${e}`).join("\n"),
  ).toEqual([]);
});

// Prerendered HTML check — verifies the build-time prerender step
// produced files that contain real marketing content. In production
// worker.js serves these for marketing routes; here we load the
// prerendered file directly to confirm it has the expected content
// before WASM even loads.
test("smoke: prerendered homepage contains marketing content", async ({
  page,
}) => {
  const response = await page.goto("/prerendered/index.html");
  expect(response).not.toBeNull();
  expect(response!.status()).toBe(200);

  const html = await response!.text();
  expect(html).toContain("Practice with intent");
  expect(html).toContain("<h1");
});

test("smoke: prerendered login page contains sign-in content", async ({
  page,
}) => {
  const response = await page.goto("/prerendered/login.html");
  expect(response).not.toBeNull();
  expect(response!.status()).toBe(200);

  const html = await response!.text();
  expect(html).toContain("Sign in to continue");
});
