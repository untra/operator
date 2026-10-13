import type { Meta, StoryObj } from "@storybook/react-vite";
import { StepProgress } from "../../src/components/StepProgress";

const STEPS = [
  { name: "plan", display_name: "Plan" },
  { name: "build", display_name: "Build" },
  { name: "review", display_name: "Review" },
];

const meta = {
  title: "Components/StepProgress",
  component: StepProgress,
  args: { steps: STEPS, current: "build", completed: ["plan"] },
} satisfies Meta<typeof StepProgress>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Mid-way: one step done, one running, one to come. */
export const InProgress: Story = {};

/** Queued: nothing has started. */
export const NotStarted: Story = { args: { current: "plan", completed: [] } };

/** A completed ticket shows every step done. */
export const Finished: Story = { args: { current: null, completed: [], finished: true } };

export const DarkTheme: Story = { globals: { theme: "dark" } };
