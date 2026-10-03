import { defineConfig, devices } from "@playwright/test";
import { ADMIN_STATE, BASE_URL } from "./fixtures/env";

// One server, one admin session: suites run serially against shared state.
export default defineConfig({
  globalSetup: "./fixtures/global-setup.ts",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  forbidOnly: Boolean(process.env.CI),
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: BASE_URL,
    trace: "retain-on-failure",
    ...devices["Desktop Chrome"],
  },
  projects: [
    { name: "workspace", testDir: "./workspace" },
    {
      name: "license",
      testDir: "./license",
      dependencies: ["workspace"],
      use: { storageState: ADMIN_STATE },
    },
  ],
});
