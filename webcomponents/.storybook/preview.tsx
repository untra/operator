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
      document.documentElement.dataset.theme = context.globals.theme === "dark" ? "dark" : "light";
      return <Story />;
    },
  ],
  parameters: {
    a11y: { test: "error" },
    controls: { expanded: true },
    viewport: {
      options: {
        desktop: { name: "Desktop", styles: { width: "1440px", height: "1000px" } },
        narrow: { name: "Narrow", styles: { width: "768px", height: "1024px" } },
      },
    },
  },
};

export default preview;
