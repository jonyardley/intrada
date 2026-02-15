import { defineConfig } from "@playwright/test";
import path from "path";

const distDir = process.env.DIST_DIR || path.resolve(__dirname, "../crates/intrada-web/dist");

export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  retries: 1,
  use: {
    baseURL: "http://localhost:8080",
    headless: true,
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
  webServer: {
    command: `npx serve ${distDir} -l 8080 --single`,
    port: 8080,
    reuseExistingServer: !process.env.CI,
    timeout: 10_000,
  },
});
