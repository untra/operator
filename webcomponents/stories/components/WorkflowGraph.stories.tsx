import type { Meta, StoryObj } from "@storybook/react-vite";
import { WorkflowGraph } from "../../src/workflow/WorkflowGraph";
import { emptyWorkflow, featureWorkflow, longWorkflow } from "../fixtures/operator";

const meta = {
  title: "Components/WorkflowGraph",
  component: WorkflowGraph,
  args: { issueType: featureWorkflow, height: 520 },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-page">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof WorkflowGraph>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const LongWorkflow: Story = { args: { issueType: longWorkflow, height: 680 } };

export const Empty: Story = { args: { issueType: emptyWorkflow } };

/** The layout the docs site ships via <operator-workflow-explorer>. */
export const VerticalLayout: Story = {
  args: { issueType: longWorkflow, vertical: true, height: 760 },
};

/** Exercises useDocumentTheme/usePhaseColors, which only light mode covered. */
export const DarkTheme: Story = { globals: { theme: "dark" } };
