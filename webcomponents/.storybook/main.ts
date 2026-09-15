import type { StorybookConfig } from "@storybook/react-vite";
import { iconAssets } from "./icon-assets.ts";

const config: StorybookConfig = {
  stories: ["../stories/**/*.stories.tsx"],
  addons: ["@storybook/addon-a11y", "@storybook/addon-vitest"],
  framework: "@storybook/react-vite",
  viteFinal(viteConfig) {
    viteConfig.plugins = [...(viteConfig.plugins ?? []), iconAssets()];
    return viteConfig;
  },
};

export default config;
