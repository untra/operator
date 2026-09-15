import type { Meta, StoryObj } from "@storybook/react-vite";
import { BrandIcon } from "../../src/components/BrandIcon";

const BRANDS = ["ollama", "openrouter", "anthropic", "google"];

const meta = {
  title: "Components/BrandIcon",
  component: BrandIcon,
  args: { src: "/icons/ollama.svg", label: "Ollama" },
} satisfies Meta<typeof BrandIcon>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Providers: Story = {
  render: () => (
    <div className="story-icon-grid">
      {BRANDS.map((name) => (
        <div key={name} className="story-icon-cell">
          <BrandIcon src={`/icons/${name}.svg`} label={name} />
          <span>{name}</span>
        </div>
      ))}
    </div>
  ),
};

/** Without a `label` the icon is decorative: empty alt text and aria-hidden. */
export const Decorative: Story = {
  args: { label: undefined },
  decorators: [
    (Story) => (
      <div className="story-icon-grid">
        <div className="story-icon-cell">
          <Story />
          <span>Decorative, labelled by its neighbouring text</span>
        </div>
      </div>
    ),
  ],
};
