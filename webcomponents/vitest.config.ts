import path from "node:path";
import { fileURLToPath } from "node:url";
import { storybookTest } from "@storybook/addon-vitest/vitest-plugin";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

const directory = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  test: {
    projects: [
      {
        plugins: [storybookTest({ configDir: path.join(directory, ".storybook") })],
        // Pre-bundle these or Vite discovers them mid-run and reloads, which
        // destroys every in-flight suite on a cold cache (i.e. every CI run).
        optimizeDeps: {
          include: ["react", "react-dom/client", "react/jsx-runtime", "react/jsx-dev-runtime"],
        },
        test: {
          name: "storybook",
          setupFiles: [path.join(directory, ".storybook/vitest.setup.ts")],
          browser: {
            enabled: true,
            provider: playwright({}),
            headless: true,
            instances: [{ browser: "chromium" }],
          },
        },
      },
    ],
  },
});
