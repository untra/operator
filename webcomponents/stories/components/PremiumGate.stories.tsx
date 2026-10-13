import type { Meta, StoryObj } from "@storybook/react-vite";
import { PremiumGate } from "../../src/components/PremiumGate";

const NOOP = () => undefined;

const LICENSE = {
  status: "valid" as const,
  profile_id: "default",
  terms: null,
  purchase_url: "https://operator.untra.io/premium",
};

const meta = {
  title: "Components/PremiumGate",
  component: PremiumGate,
  args: {
    feature: "Campaigns",
    onAddLicense: NOOP,
    license: { ...LICENSE, premium: false },
    children: <p>Campaign content, shown only with Premium.</p>,
  },
} satisfies Meta<typeof PremiumGate>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Without Premium the gate renders the paywall in place of its content. */
export const Unlicensed: Story = {};

/** With Premium the gate renders its content untouched. */
export const Licensed: Story = {
  args: { license: { ...LICENSE, premium: true } },
};

export const DarkTheme: Story = { globals: { theme: "dark" } };
