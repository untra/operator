import type { Preview } from "@storybook/react-vite";
import "@xyflow/react/dist/style.css";
import "@untra/naiveworkflow-react/styles.css";
import "@vscode/codicons/dist/codicon.css";
import "../../docs/assets/css/tokens.css";
import "../src/elements.css";
import "../src/styles/semantic.css";
import "./preview.css";

/** Under the browser test runner, render without motion so captures are stable. */
if (import.meta.env.MODE === "test" || navigator.webdriver) {
  document.documentElement.dataset.storybookTest = "true";
}

const preview: Preview = {
  globalTypes: {
    theme: {
      description: "Operator color theme",
      toolbar: {
        icon: "paintbrush",
        items: ["light", "dark"],
      },
    },
  },
  initialGlobals: { theme: "light" },
  decorators: [
    (Story, context) => {
      document.documentElement.dataset.theme =
        (context.globals.auditTheme ?? context.globals.theme) === "dark" ? "dark" : "light";
      return <Story />;
    },
  ],
  beforeEach: async (context) => {
    if (import.meta.env.MODE === "test" && context.globals.auditWidth) {
      const { page } = await import("vitest/browser");
      await page.viewport(context.globals.auditWidth, 1000);
    }
  },
  parameters: {
    a11y: {
      test: "error",
      options: {
        runOnly: {
          type: "tag",
          values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"],
        },
      },
    },
    controls: { expanded: true },
    viewport: {
      options: {
        mobile: { name: "Mobile 320px", styles: { width: "320px", height: "1000px" } },
        desktop: { name: "Desktop", styles: { width: "1440px", height: "1000px" } },
        narrow: { name: "Narrow", styles: { width: "768px", height: "1024px" } },
      },
    },
  },
};

export default preview;
