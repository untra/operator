import type { Meta, StoryObj } from "@storybook/react-vite";
import { IssueTypesView } from "../../src/views/IssueTypesView";
import { featureWorkflow, issueTypeSummaries, selectedIssueType } from "../fixtures/operator";

const meta = {
  title: "Pages/Issue Types",
  component: IssueTypesView,
  args: {
    header: {
      title: "Issue Types",
      summary: "Reusable workflows that define how Operator handles work.",
      docsUrl: "https://example.test/docs/issue-types",
      icon: "symbol-enum",
    },
    issueTypes: { status: "ready", data: issueTypeSummaries },
    selected: null,
    workflow: { status: "initial" },
    mode: "steps",
    onSelect: () => undefined,
    onModeChange: () => undefined,
  },
  decorators: [
    (Story) => (
      <div className="story-page-frame">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof IssueTypesView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const List: Story = {};

export const SelectedGraph: Story = {
  args: {
    selected: selectedIssueType,
    workflow: { status: "ready", data: featureWorkflow },
    mode: "graph",
  },
};

/** `mode: "steps"` with a selection renders the ordered step list, not the graph. */
export const StepsMode: Story = {
  args: {
    selected: selectedIssueType,
    workflow: { status: "ready", data: featureWorkflow },
    mode: "steps",
  },
};

export const LoadFailed: Story = {
  args: {
    issueTypes: { status: "error", message: "Could not list issue types." },
    error: "Operator API is unavailable.",
  },
};

/** The workflow document failed after its issue type was selected. */
export const WorkflowFailed: Story = {
  args: {
    selected: selectedIssueType,
    workflow: { status: "error", message: "Could not load the FEAT workflow document." },
    mode: "graph",
  },
};

/** The split view collapses to one column at the 900px breakpoint. */
export const NarrowGraph: Story = {
  args: {
    selected: selectedIssueType,
    workflow: { status: "ready", data: featureWorkflow },
    mode: "graph",
  },
  globals: { viewport: { value: "narrow" } },
};
