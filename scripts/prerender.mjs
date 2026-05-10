#!/usr/bin/env node
// Build-time prerender for marketing routes.
//
// Launches headless Chromium against the Trunk-built dist/, waits for
// WASM to render each marketing route, and writes the captured HTML to
// dist/prerendered/. The Cloudflare Worker serves these files for the
// matching paths so crawlers (and all users) see real content on first
// fetch. WASM loads on top and takes over interactively.
//
// Usage:
//   node scripts/prerender.mjs [DIST_DIR]
//
// Defaults to crates/intrada-web/dist. Requires Playwright browsers
// installed (`npx playwright install chromium`).

import { chromium } from "playwright";
import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { createConnection } from "node:net";

const DEFAULT_DIST = "crates/intrada-web/dist";
const PORT = 9222;
const TIMEOUT_MS = 30_000;

const ROUTES = [
  { path: "/", output: "index.html", waitFor: "Music practice with intent" },
  { path: "/login", output: "login.html", waitFor: "Sign in" },
];

const distDir = resolve(process.argv[2] || DEFAULT_DIST);

async function waitForPort(port, timeoutMs = 10_000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const open = await new Promise((res) => {
      const sock = createConnection({ port, host: "127.0.0.1" }, () => {
        sock.destroy();
        res(true);
      });
      sock.on("error", () => res(false));
    });
    if (open) return;
    await new Promise((r) => setTimeout(r, 200));
  }
  throw new Error(`Port ${port} not ready within ${timeoutMs}ms`);
}

async function main() {
  console.log(`prerender: dist=${distDir}, routes=${ROUTES.length}`);

  // Start a local static server with SPA fallback (--single).
  const server = spawn("npx", ["serve", distDir, "-l", String(PORT), "--single"], {
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, FORCE_COLOR: "0" },
  });
  server.stderr.on("data", (d) => {
    const msg = d.toString().trim();
    if (msg) console.error(`  serve: ${msg}`);
  });

  try {
    await waitForPort(PORT);
    console.log(`prerender: server ready on port ${PORT}`);

    const outDir = join(distDir, "prerendered");
    await mkdir(outDir, { recursive: true });

    const browser = await chromium.launch();
    try {
      for (const route of ROUTES) {
        const page = await browser.newPage();
        const url = `http://127.0.0.1:${PORT}${route.path}`;
        console.log(`prerender: ${route.path} → ${route.output}`);

        await page.goto(url, { waitUntil: "networkidle" });

        // Wait for the WASM app to render the expected content.
        await page.waitForFunction(
          (text) => document.body?.innerText?.includes(text),
          route.waitFor,
          { timeout: TIMEOUT_MS },
        );

        const html = await page.content();
        const outPath = join(outDir, route.output);
        await writeFile(outPath, html, "utf-8");
        console.log(`prerender: wrote ${outPath} (${html.length} bytes)`);
        await page.close();
      }
    } finally {
      await browser.close();
    }
  } finally {
    server.kill("SIGTERM");
  }

  console.log("prerender: done");
}

main().catch((err) => {
  console.error("prerender: FAILED", err);
  process.exit(1);
});
