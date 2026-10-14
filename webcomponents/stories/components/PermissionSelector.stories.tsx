import type { Meta, StoryObj } from "@storybook/react-vite";
import { PermissionSelector } from "../../src/components/PermissionSelector";
import type { PermissionGrant } from "../../src/generated/PermissionGrant";

const available: PermissionGrant = {
  profile_id: "00000000-0000-0000-0000-000000000001",
  instance_wide: true,
  capabilities: ["tickets:read", "tickets:create", "agents:execute", "mcp:connect"],
  projects: ["operator", "website"],
};
const empty: PermissionGrant = {
  ...available,
  capabilities: [],
  projects: [],
  instance_wide: false,
};
function onChange() {}

const meta = {
  title: "Components/PermissionSelector",
  component: PermissionSelector,
  args: { available, value: empty, onChange },
} satisfies Meta<typeof PermissionSelector>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Empty: Story = {};
export const Selected: Story = { args: { value: available } };
export const Disabled: Story = { args: { disabled: true } };
export const DarkTheme: Story = { globals: { theme: "dark" } };
export const Narrow: Story = { globals: { viewport: { value: "narrow" } } };
