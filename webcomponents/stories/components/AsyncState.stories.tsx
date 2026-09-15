import type { Meta, StoryObj } from "@storybook/react-vite";
import { AsyncState, type AsyncValue } from "../../src/components/AsyncState";

interface TextStateProps {
  value: AsyncValue<string>;
}

function TextState({ value }: TextStateProps) {
  return <AsyncState value={value}>{(text) => <div>{text}</div>}</AsyncState>;
}

const meta = {
  title: "Components/AsyncState",
  component: TextState,
  args: { value: { status: "loading", message: "Loading ticket data…" } },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-form">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof TextState>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Loading: Story = {};

export const Empty: Story = { args: { value: { status: "empty", message: "No tickets found." } } };

export const Error: Story = {
  args: { value: { status: "error", message: "Operator API is unavailable." } },
};

export const Ready: Story = { args: { value: { status: "ready", data: "Ticket data loaded." } } };

export const Initial: Story = { args: { value: { status: "initial" } } };

const UNMESSAGED: AsyncValue<string>[] = [
  { status: "initial" },
  { status: "loading" },
  { status: "empty" },
];

/** No `message` on any variant, so the component's own defaults render. */
export const DefaultMessages: Story = {
  render: () => (
    <div className="story-bounded story-bounded-form">
      {UNMESSAGED.map((value) => (
        <TextState key={value.status} value={value} />
      ))}
    </div>
  ),
};
