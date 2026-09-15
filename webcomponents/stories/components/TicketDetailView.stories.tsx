import type { Meta, StoryObj } from "@storybook/react-vite";
import { LaunchForm } from "../../src/components/LaunchForm";
import { TicketDetailView } from "../../src/components/TicketDetailView";
import { delegators, featureWorkflow, queuedTicket, unnamedStepTicket } from "../fixtures/operator";

const launchControls = (
  <LaunchForm
    value={{ delegator: "", wrapper: "", target: "", yolo: false }}
    delegators={delegators}
    targets={["local"]}
    defaultWrapperLabel="tmux"
    onChange={() => undefined}
    onSubmit={() => undefined}
  />
);

const meta = {
  title: "Components/TicketDetailView",
  component: TicketDetailView,
  args: {
    ticket: queuedTicket,
    launchControls,
    workflow: { status: "ready", data: featureWorkflow },
  },
  decorators: [
    (Story) => (
      <div className="story-panel-frame">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof TicketDetailView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ReadyToLaunch: Story = {};

export const LaunchComplete: Story = {
  args: {
    launchControls: undefined,
    launchedTicketId: "FEAT-1042",
    launchActions: (
      <>
        <button type="button">Open session</button>
        <button type="button">View ticket</button>
      </>
    ),
  },
};

/** The workflow document failed to load - a live branch of TicketDetailPanel. */
export const WorkflowFailed: Story = {
  args: { workflow: { status: "error", message: "Could not load the FEAT workflow." } },
};

export const WorkflowLoading: Story = {
  args: { workflow: { status: "loading", message: "Loading workflow…" } },
};

/** Launched, but the controller supplied no follow-up actions. */
export const LaunchedWithoutActions: Story = {
  args: { launchControls: undefined, launchedTicketId: "FEAT-1042", launchActions: undefined },
};

/** `step_display_name` is null, so the raw step name is shown. */
export const UnnamedStep: Story = { args: { ticket: unnamedStepTicket } };
