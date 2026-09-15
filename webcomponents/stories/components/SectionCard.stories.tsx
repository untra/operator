import type { Meta, StoryObj } from "@storybook/react-vite";
import { SectionCard } from "../../src/components/SectionCard";
import {
  BRAND_ICON_SRC,
  degradedSection,
  healthySection,
  lockedSection,
  lockedSectionWithChildren,
} from "../fixtures/operator";

const meta = {
  title: "Components/SectionCard",
  component: SectionCard,
  args: { section: healthySection, brandIconSrc: BRAND_ICON_SRC },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-wide">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof SectionCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Healthy: Story = {};

export const Locked: Story = {
  args: {
    section: lockedSection,
    renderPrerequisite: (id) => <a href={`#${id}`}>{id}</a>,
  },
};

export const Empty: Story = {
  args: {
    section: {
      ...healthySection,
      id: "projects",
      label: "Projects",
      description: "No projects have been configured.",
      health: "yellow",
      children: [],
    },
  },
};

/** Red and yellow rows, three actions on one row, and long wrapping descriptions. */
export const DegradedLongRows: Story = { args: { section: degradedSection } };

/** Unmet prerequisites *and* children: the card renders the lock, the prereqs and the rows. */
export const LockedWithChildren: Story = {
  args: {
    section: lockedSectionWithChildren,
    renderPrerequisite: (id) => <a href={`#${id}`}>{id}</a>,
  },
};

/** No `renderPrerequisite`, so prerequisites fall back to their raw ids. */
export const PrerequisitesUnlinked: Story = {
  args: { section: lockedSection, renderPrerequisite: undefined },
};

/** No `brandIconSrc`, so rows carrying a brand_icon render without one. */
export const WithoutBrandIcons: Story = { args: { brandIconSrc: undefined } };

export const DarkTheme: Story = { globals: { theme: "dark" } };
