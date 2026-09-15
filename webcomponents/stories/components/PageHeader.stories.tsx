import type { Meta, StoryObj } from "@storybook/react-vite";
import { PageHeader } from "../../src/components/PageHeader";

const meta = {
  title: "Components/PageHeader",
  component: PageHeader,
  args: {
    title: "Dashboard",
    summary: "Workspace health, ticket throughput, and active work.",
    docsUrl: "https://example.test/docs/dashboard",
    icon: "dashboard",
  },
} satisfies Meta<typeof PageHeader>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const LongContent: Story = {
  args: {
    title: "Model provider and delegation configuration",
    summary:
      "Review a deliberately long description that verifies wrapping without displacing the documentation link.",
  },
  globals: { viewport: { value: "narrow" } },
};

/** `icon` is optional; Status and Security ship without one. */
export const WithoutIcon: Story = {
  args: { title: "Status", summary: "Every section operator knows about.", icon: undefined },
};
