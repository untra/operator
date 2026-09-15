import type { Meta, StoryObj } from "@storybook/react-vite";
import { LaunchForm, type LaunchFormValue } from "../../src/components/LaunchForm";
import { delegators, unnamedDelegator } from "../fixtures/operator";

const DEFAULT_VALUE: LaunchFormValue = { delegator: "", wrapper: "", target: "", yolo: false };

const meta = {
  title: "Components/LaunchForm",
  component: LaunchForm,
  args: {
    value: DEFAULT_VALUE,
    delegators,
    targets: ["local", "remote-linux"],
    defaultWrapperLabel: "tmux",
    onChange: () => undefined,
    onSubmit: () => undefined,
  },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-form">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof LaunchForm>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Launching: Story = { args: { busy: true } };

export const Disabled: Story = { args: { disabled: true } };

export const ServerValidationError: Story = {
  args: {
    value: { ...DEFAULT_VALUE, delegator: "claude-opus", wrapper: "tmux" },
    error: "The selected delegator is not available for this workspace.",
  },
};

/** An authenticated request the server refused; the controls stay visible. */
export const Forbidden: Story = {
  args: {
    value: { ...DEFAULT_VALUE, delegator: "claude-opus", wrapper: "tmux" },
    error: "Forbidden: this access key cannot launch agents.",
  },
};

/** With no execution targets configured the Target field is not rendered at all. */
export const NoTargets: Story = { args: { targets: [] } };

/** No delegators configured yet. */
export const NoDelegators: Story = { args: { delegators: [] } };

/** `display_name` is null on this delegator, so its `name` is shown. */
export const UnnamedDelegator: Story = { args: { delegators: [unnamedDelegator] } };

export const YoloEnabled: Story = {
  args: { value: { ...DEFAULT_VALUE, delegator: "claude-opus", wrapper: "tmux", yolo: true } },
};
