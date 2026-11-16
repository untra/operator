import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";
import { SelectInput } from "../../src/components/fields/SelectInput";
import { required } from "../../src/components/fields/validators";

const WRAPPERS = [
  { value: "tmux", label: "tmux" },
  { value: "vscode", label: "VS Code" },
  { value: "cmux", label: "cmux" },
  { value: "zellij", label: "Zellij", disabled: true },
];

const meta = {
  title: "Components/SelectInput",
  component: SelectInput,
  args: {
    label: "Session wrapper",
    options: WRAPPERS,
    value: "",
    placeholder: "Choose a wrapper…",
    onChange: fn(),
    rules: [required("Choose a wrapper")],
  },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-form">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof SelectInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Selected: Story = { args: { value: "vscode" } };

export const WithHint: Story = {
  args: { value: "tmux", hint: "Where launched agents run." },
};

export const Invalid: Story = { args: { showErrors: true } };

export const Disabled: Story = { args: { value: "tmux", disabled: true } };
