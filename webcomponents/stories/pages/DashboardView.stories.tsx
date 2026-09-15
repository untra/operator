import type { Meta, StoryObj } from "@storybook/react-vite";
import { DashboardView } from "../../src/views/DashboardView";
import { FIXED_UPDATED_LABEL, board, health, queueStatus } from "../fixtures/operator";

const meta = {
  title: "Pages/Dashboard",
  component: DashboardView,
  args: {
    header: {
      title: "Dashboard",
      summary: "Workspace health, ticket throughput, and active work.",
      docsUrl: "https://example.test/docs/dashboard",
      icon: "dashboard",
    },
    health,
    queue: queueStatus,
    board,
    updatedLabel: FIXED_UPDATED_LABEL,
    statusLink: <a href="#status">View system status</a>,
    onOpenTicket: () => undefined,
  },
  decorators: [
    (Story) => (
      <div className="story-page-frame">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof DashboardView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Ready: Story = {};

export const ApiError: Story = {
  args: { health: null, queue: null, board: null, error: "Unable to connect to Operator." },
  globals: { theme: "dark" },
};

/** A refresh failed but the previous data is still on screen - the real polling failure. */
export const StaleWithError: Story = {
  args: { error: "Refresh failed: Operator API is unavailable." },
};

/** Health resolved but the queue counts did not, exercising the "-" metric fallback. */
export const PartialData: Story = { args: { queue: null, board: null } };
