import path from "node:path";
import { fileURLToPath } from "node:url";
import { storybookTest } from "@storybook/addon-vitest/vitest-plugin";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

const directory = path.dirname(fileURLToPath(import.meta.url));
const optimizeDeps = {
  include: ["react", "react-dom/client", "react/jsx-runtime", "react/jsx-dev-runtime"],
};

const browser = () => ({
  enabled: true,
  provider: playwright({}),
  headless: true,
  instances: [{ browser: "chromium" as const }],
});

const setupFiles = [path.join(directory, ".storybook/vitest.setup.ts")];

export default defineConfig({
  test: {
    projects: [
      ...["light", "dark"].flatMap((theme) =>
        [1440, 320].map((width) => ({
          plugins: [
            storybookTest({
              configDir: path.join(directory, ".storybook"),
              initialGlobals: { auditTheme: theme, auditWidth: width },
            }),
          ],
          optimizeDeps,
          test: { name: `storybook-${theme}-${width}`, setupFiles, browser: browser() },
        })),
      ),
      {
        optimizeDeps,
        test: {
          name: "stories-behaviour",
          include: ["stories/**/*.browser.tsx"],
          setupFiles,
          browser: browser(),
        },
      },
    ],
  },
});
