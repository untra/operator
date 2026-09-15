import type { Meta, StoryObj } from "@storybook/react-vite";
import { Choice, ChoiceGroup } from "../../src/components/Choice";

const NOOP = () => undefined;

const WRAPPERS = [
  { value: "tmux", label: "tmux", blurb: "Terminal multiplexer. The default." },
  { value: "vscode", label: "VS Code", blurb: "Launch agents in an editor terminal." },
  { value: "cmux", label: "cmux", blurb: "Container-backed sessions." },
  { value: "zellij", label: "Zellij", blurb: "Rust terminal workspace." },
] as const;

const meta = {
  title: "Components/Choice",
  component: Choice,
  args: { value: "tmux", selected: false, onSelect: NOOP, children: "tmux" },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-wide">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Choice>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The accent marker on the selected card is what `--accent` drives. */
export const Group: Story = {
  render: () => (
    <ChoiceGroup>
      {WRAPPERS.map((wrapper) => (
        <Choice
          key={wrapper.value}
          value={wrapper.value}
          selected={wrapper.value === "vscode"}
          onSelect={NOOP}
        >
          <strong>{wrapper.label}</strong>
          <small>{wrapper.blurb}</small>
        </Choice>
      ))}
    </ChoiceGroup>
  ),
};

export const Unselected: Story = {};

export const Selected: Story = { args: { selected: true } };

export const DarkTheme: Story = { args: { selected: true }, globals: { theme: "dark" } };

export const Narrow: Story = {
  ...Group,
  globals: { viewport: { value: "narrow" } },
};
