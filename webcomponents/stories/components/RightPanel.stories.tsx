import type { Meta, StoryObj } from "@storybook/react-vite";
import { RightPanel } from "../../src/components/RightPanel";

const meta = {
  title: "Components/RightPanel",
  component: RightPanel,
  args: {
    title: "FEAT-1042",
    children: <p>Ticket details and actions appear in this bounded panel.</p>,
    onClose: () => undefined,
  },
  decorators: [
    (Story) => (
      <div className="story-panel-frame">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof RightPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Open: Story = {};

/** No `title`, so the aside falls back to its generic accessible name. */
export const Untitled: Story = { args: { title: undefined } };

export const ScrollingBody: Story = {
  args: {
    children: (
      <>
        {Array.from({ length: 40 }, (_, index) => (
          <p key={index}>
            Row {index + 1} of a body that overflows the panel and must scroll rather than clip.
          </p>
        ))}
      </>
    ),
  },
};
