import type { Meta, StoryObj } from "@storybook/react-vite";
import { AuthCard, AuthField, AuthSubmit } from "../../src/components/AuthCard";

const NOOP = () => undefined;

const credentials = (
  <>
    <AuthField label="Username" autoComplete="username" value="operator" onChange={NOOP} required />
    <AuthField
      label="Password"
      type="password"
      autoComplete="current-password"
      value="hunter2hunter2"
      onChange={NOOP}
      required
    />
  </>
);

const meta = {
  title: "Components/AuthCard",
  component: AuthCard,
  args: {
    title: "Sign in to Operator",
    subtitle: "Enter your account credentials.",
    children: credentials,
    actions: <AuthSubmit busyLabel="Signing in…">Sign in</AuthSubmit>,
    links: <a href="#forgot">Forgot password?</a>,
  },
} satisfies Meta<typeof AuthCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const SignIn: Story = {};

export const RejectedCredentials: Story = {
  args: { error: "Incorrect username or password." },
};

/** A non-failure status, as Forgot Password reports. */
export const WithNotice: Story = {
  args: {
    title: "Forgot password",
    subtitle: "Operator recovery is performed by the server administrator.",
    notice: "Ask the administrator to run `operator auth reset-admin-password` locally.",
  },
};

export const Submitting: Story = {
  args: {
    actions: (
      <AuthSubmit busy busyLabel="Signing in…">
        Sign in
      </AuthSubmit>
    ),
  },
};

/** Setup shows a password policy hint, and a mismatch hint on confirmation. */
export const FieldHints: Story = {
  args: {
    title: "Set up Operator",
    subtitle: "Choose the admin password for this workspace.",
    children: (
      <>
        <AuthField
          label="New admin password"
          type="password"
          value="short"
          onChange={NOOP}
          hint="At least 12 characters."
          required
        />
        <AuthField
          label="Confirm password"
          type="password"
          value="shorter"
          onChange={NOOP}
          hint="Passwords do not match."
          required
        />
      </>
    ),
    actions: (
      <AuthSubmit disabled busyLabel="Creating…">
        Create admin account
      </AuthSubmit>
    ),
    links: undefined,
  },
};

/** The device-approval outcome card: no fields and no actions. */
export const OutcomeOnly: Story = {
  args: {
    title: "Device approved",
    subtitle: undefined,
    children: (
      <p>
        <strong>operator-cli</strong> now has access. You can close this page.
      </p>
    ),
    actions: undefined,
    links: undefined,
  },
};

export const DarkTheme: Story = { globals: { theme: "dark" } };

export const Narrow: Story = {
  globals: { viewport: { value: "narrow" } },
};
