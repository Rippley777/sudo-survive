import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests",
  workers: 1,
  timeout: 90_000,
  use: {
    baseURL: "http://127.0.0.1:1420",
    viewport: { width: 1100, height: 730 },
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npm run dev:browser",
    url: "http://127.0.0.1:1420/api/state",
    reuseExistingServer: false,
    timeout: 180_000,
    env: { SUDO_SURVIVE_DB: ":memory:", SUDO_SURVIVE_TICK_MS: "250" },
  },
});
