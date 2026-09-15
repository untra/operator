import type { Meta, StoryObj } from "@storybook/react-vite";
import { WorkflowExplorerView } from "../../src/views/WorkflowExplorerView";
import { featureWorkflow, longWorkflow, manifestEntries } from "../fixtures/operator";

const meta = {
  title: "Docs/WorkflowExplorerView",
  component: WorkflowExplorerView,
  args: {
    entries: manifestEntries,
    selected: "FEAT",
    document: featureWorkflow,
    onSelect: () => undefined,
  },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-page">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof WorkflowExplorerView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Selected: Story = {};

/** Manifest resolved, document still in flight. */
export const LoadingWorkflow: Story = { args: { selected: "RELEASE", document: null } };

/** Manifest itself still loading. */
export const LoadingCollection: Story = { args: { entries: null, document: null } };

/**
 * A manifest with no issue types. Previously rendered an empty rail reading
 * "Loading workflow…" indefinitely.
 */
export const EmptyCollection: Story = { args: { entries: [], document: null } };

export const LoadFailed: Story = {
  args: { error: "404 Not Found for /collections/missing/collection.json" },
};

/** The layout the narrow docs prose column actually gets. */
export const Narrow: Story = {
  args: { selected: "RELEASE", document: longWorkflow },
  globals: { viewport: { value: "narrow" } },
};

export const DarkTheme: Story = { globals: { theme: "dark" } };
