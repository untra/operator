import type { Meta, StoryObj } from "@storybook/react-vite";
import { KanbanFilterBar } from "../../src/components/KanbanFilterBar";
import { DEFAULT_FILTER_STATE, facetsFromBoard } from "../../src/shared/kanban-filters";
import { busyBoard } from "../fixtures/operator";

const facets = facetsFromBoard(busyBoard);

const meta = {
  title: "Components/KanbanFilterBar",
  component: KanbanFilterBar,
  args: {
    facets,
    state: DEFAULT_FILTER_STATE,
    onChange: () => undefined,
    onClear: () => undefined,
  },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-page">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof KanbanFilterBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Inert: Story = {};

/** The clear control only appears once the state differs from the default. */
export const Active: Story = {
  args: {
    state: { ...DEFAULT_FILTER_STATE, projects: ["gamesvc"], priorities: ["P0-critical"] },
  },
};

/** A facet with fewer than two options is not worth offering. */
export const SingleProject: Story = {
  args: { facets: { ...facets, projects: ["operator"] } },
};
