import type { Meta, StoryObj } from "@storybook/react-vite";
import { KanbanBoard } from "../../src/components/KanbanBoard";
import { board, emptyBoard, unnamedStepTicket } from "../fixtures/operator";

const meta = {
  title: "Components/KanbanBoard",
  component: KanbanBoard,
  args: { board, onOpenTicket: () => undefined },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-page">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof KanbanBoard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Populated: Story = {};

export const Empty: Story = { args: { board: emptyBoard } };

/** Without `onOpenTicket` the cards render as static divs rather than buttons. */
export const ReadOnly: Story = { args: { onOpenTicket: undefined } };

/** `step_display_name` is null on TASK-204, so the raw `step` shows instead. */
export const UnnamedStep: Story = {
  args: { board: { ...board, queue: [...board.queue, unnamedStepTicket], total_count: 5 } },
};

/** Columns collapse to one at the 900px breakpoint. */
export const Narrow: Story = {
  globals: { viewport: { value: "narrow" } },
};
