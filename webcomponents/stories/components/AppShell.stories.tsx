import type { Meta, StoryObj } from "@storybook/react-vite";
import {
  AppShell,
  BrandName,
  NavGroup,
  NavRow,
  SignOutButton,
  ThemeToggle,
} from "../../src/components/AppShell";
import { PageHeader } from "../../src/components/PageHeader";

const NOOP = () => undefined;

const STATUS_ROWS = [
  { label: "Model Providers", icon: "server", health: "green" },
  { label: "Projects", icon: "repo", health: "red" },
  { label: "Agents", icon: "person", health: "yellow" },
  { label: "Kanban", icon: "project", health: "gray" },
];

const PAGE_ROWS = [
  { label: "Dashboard", icon: "dashboard" },
  { label: "Queue", icon: "list-tree" },
];

/** Stories render plain anchors; the SPA supplies a react-router NavLink here. */
const anchor =
  (isActive: boolean) =>
  (content: React.ReactNode, className: (state: { isActive: boolean }) => string) => (
    <a href="#nav" className={className({ isActive })}>
      {content}
    </a>
  );

function Groups({ lockedReason }: { lockedReason?: string }) {
  return (
    <>
      <NavGroup label="Status">
        {STATUS_ROWS.map((row, index) => (
          <li key={row.label}>
            <NavRow
              label={row.label}
              icon={row.icon}
              health={row.health}
              disabledReason={row.label === "Kanban" ? lockedReason : undefined}
              renderLink={row.label === "Kanban" && lockedReason ? undefined : anchor(index === 0)}
            />
          </li>
        ))}
      </NavGroup>
      <NavGroup label="Pages">
        {PAGE_ROWS.map((row) => (
          <li key={row.label}>
            <NavRow label={row.label} icon={row.icon} renderLink={anchor(false)} />
          </li>
        ))}
      </NavGroup>
    </>
  );
}

const meta = {
  title: "Components/AppShell",
  component: AppShell,
  args: {
    brand: (
      <>
        <BrandName />
        <ThemeToggle theme="light" onToggle={NOOP} />
      </>
    ),
    groups: <Groups />,
    footer: <SignOutButton onClick={NOOP} />,
    children: (
      <PageHeader
        title="Dashboard"
        summary="Workspace health, ticket throughput, and active work."
        docsUrl="https://example.test/docs/dashboard"
        icon="dashboard"
      />
    ),
  },
} satisfies Meta<typeof AppShell>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** A section whose prerequisites are unmet: a disabled span naming what it needs. */
export const PrerequisiteLocked: Story = {
  args: { groups: <Groups lockedReason="Requires: Model Providers, Projects" /> },
};

export const SigningOut: Story = { args: { footer: <SignOutButton busy onClick={NOOP} /> } };

export const SignOutFailed: Story = { args: { footer: <SignOutButton failed onClick={NOOP} /> } };

export const DarkTheme: Story = {
  args: {
    brand: (
      <>
        <BrandName />
        <ThemeToggle theme="dark" onToggle={NOOP} />
      </>
    ),
  },
  globals: { theme: "dark" },
};

/** The nav rail is a fixed 260px; check the main column still works beside it. */
export const Narrow: Story = {
  globals: { viewport: { value: "narrow" } },
};
