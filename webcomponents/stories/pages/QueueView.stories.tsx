import type { Meta, StoryObj } from "@storybook/react-vite";
import { RightPanel } from "../../src/components/RightPanel";
import { TicketDetailView } from "../../src/components/TicketDetailView";
import { QueueView } from "../../src/views/QueueView";
import {
  FIXED_UPDATED_LABEL,
  board,
  emptyBoard,
  featureWorkflow,
  queuedTicket,
} from "../fixtures/operator";

const header = {
  title: "Queue",
  summary: "Work waiting, running, awaiting review, and complete.",
  docsUrl: "https://example.test/docs/queue",
  icon: "list-tree",
};
const NOOP = () => undefined;
const READY_BOARD = { status: "ready", data: board } as const;
const READY_WORKFLOW = { status: "ready", data: featureWorkflow } as const;

const meta = {
  title: "Pages/Queue",
  component: QueueView,
  args: {
    header,
    board: { status: "loading", message: "Loading queue…" },
    onOpenTicket: NOOP,
  },
  decorators: [
    (Story) => (
      <div className="story-page-frame">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof QueueView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Loading: Story = {};

export const TicketPanelOpen: Story = {
  render: () => (
    <div className="story-page-with-panel">
      <QueueView
        header={header}
        board={READY_BOARD}
        updatedLabel={FIXED_UPDATED_LABEL}
        onOpenTicket={NOOP}
      />
      <RightPanel title={queuedTicket.id} onClose={NOOP}>
        <TicketDetailView ticket={queuedTicket} workflow={READY_WORKFLOW} />
      </RightPanel>
    </div>
  ),
};

/** What QueuePage emits when the board comes back with no tickets. */
export const Empty: Story = {
  args: { board: { status: "empty", message: "The queue is empty." } },
};

export const LoadFailed: Story = {
  args: {
    board: { status: "error", message: "Could not load the queue." },
    error: "Operator API is unavailable.",
  },
};

/** A ready board with no `updatedLabel`, so the meta line omits the timestamp. */
export const WithoutUpdatedLabel: Story = {
  args: { board: { status: "ready", data: emptyBoard } },
};
