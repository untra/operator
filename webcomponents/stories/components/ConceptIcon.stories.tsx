import type { Meta, StoryObj } from "@storybook/react-vite";
import { ConceptIcon } from "../../src/components/ConceptIcon";

const ICONS = ["dashboard", "list-tree", "git-pull-request", "warning", "check", "close"];

const meta = {
  title: "Components/ConceptIcon",
  component: ConceptIcon,
  args: { name: "dashboard" },
} satisfies Meta<typeof ConceptIcon>;

export default meta;
type Story = StoryObj<typeof meta>;

export const OperatorIcons: Story = {
  render: () => (
    <div className="story-icon-grid">
      {ICONS.map((name) => (
        <div key={name} className="story-icon-cell">
          <ConceptIcon name={name} />
          <span>{name}</span>
        </div>
      ))}
    </div>
  ),
};
