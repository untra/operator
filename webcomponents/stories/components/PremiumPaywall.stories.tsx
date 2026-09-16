import type { Meta, StoryObj } from "@storybook/react-vite";
import { PremiumPaywall } from "../../src/components/PremiumPaywall";

const NOOP = () => undefined;

const meta = {
  title: "Components/PremiumPaywall",
  component: PremiumPaywall,
  args: {
    feature: "Remote targets",
    purchaseUrl: "https://operator.untra.io/premium",
    onAddLicense: NOOP,
  },
} satisfies Meta<typeof PremiumPaywall>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The full-section form, as the Remote Targets page renders it. */
export const Section: Story = {};

/** Inline, as the onboarding execution-mode step renders it. */
export const Inline: Story = {
  args: { inline: true },
};

/** A build with no purchase destination configured offers only "Add license". */
export const WithoutPurchaseUrl: Story = {
  args: { purchaseUrl: null },
};

/**
 * A non-https destination is not linked. The URL is a build-time input, so this
 * is the state that must not render an anchor.
 */
export const RefusesInsecurePurchaseUrl: Story = {
  args: { purchaseUrl: "http://operator.untra.io/premium" },
};

/** Callers can replace the default explanation. */
export const CustomDescription: Story = {
  args: {
    feature: "SSH hosts",
    description:
      "Running agents on another machine requires a Premium license for this configuration.",
  },
};

export const DarkTheme: Story = { globals: { theme: "dark" } };

export const Narrow: Story = {
  globals: { viewport: { value: "narrow" } },
};
