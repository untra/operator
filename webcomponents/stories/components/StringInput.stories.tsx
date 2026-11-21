import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";
import { StringInput } from "../../src/components/fields/StringInput";
import { SelectInput } from "../../src/components/fields/SelectInput";
import {
  isValid,
  maxLength,
  pattern,
  required,
  url,
  type Validator,
} from "../../src/components/fields/validators";

const NAME_RULES: readonly Validator<string>[] = [
  required("Name the configuration"),
  maxLength(64),
  pattern(/^[a-z0-9_-]+$/, "Use lowercase letters, numbers, hyphens, or underscores"),
];

const meta = {
  title: "Components/StringInput",
  component: StringInput,
  args: {
    label: "Configuration name",
    value: "",
    onChange: fn(),
    hint: "Lowercase letters, numbers, hyphens, and underscores.",
    rules: NAME_RULES,
  },
  decorators: [
    (Story) => (
      <div className="story-bounded story-bounded-form">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof StringInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithValue: Story = { args: { value: "untra-casa" } };

/** Rule errors forced visible, as after a submit attempt. */
export const Invalid: Story = { args: { value: "untra.casa", showErrors: true } };

export const ServerError: Story = {
  args: { value: "untra", error: "A configuration with this name already exists" },
};

export const Password: Story = {
  args: {
    label: "API token",
    type: "password",
    value: "secret-token",
    hint: "Stored in the session environment only.",
    rules: [required()],
  },
};

export const Disabled: Story = { args: { value: "untra", disabled: true } };

function Controlled(props: { initial: string }) {
  const [value, setValue] = useState(props.initial);
  return (
    <StringInput
      label="Configuration name"
      hint="Lowercase letters, numbers, hyphens, and underscores."
      rules={NAME_RULES}
      value={value}
      onChange={setValue}
    />
  );
}

/** No error while first typing; it appears on blur and clears once fixed. */
export const ValidatesOnBlur: Story = { render: () => <Controlled initial="" /> };

const KINDS = [
  { value: "jira", label: "Jira" },
  { value: "linear", label: "Linear" },
  { value: "github", label: "GitHub" },
] as const;
type Kind = (typeof KINDS)[number]["value"];

function Gated() {
  const [instance, setInstance] = useState("");
  const [kind, setKind] = useState<Kind | "">("");
  const instanceRules = [required(), url()];
  const kindRules = [required("Choose a provider")];
  const ready = isValid(instance, instanceRules) && isValid<string>(kind, kindRules);
  return (
    <form style={{ display: "grid", gap: "1rem" }} onSubmit={(event) => event.preventDefault()}>
      <SelectInput
        label="Provider"
        placeholder="Choose…"
        options={KINDS}
        rules={kindRules}
        value={kind}
        onChange={setKind}
      />
      <StringInput
        label="Instance URL"
        type="url"
        placeholder="https://org.atlassian.net"
        rules={instanceRules}
        value={instance}
        onChange={setInstance}
      />
      <button type="submit" disabled={!ready}>
        Continue
      </button>
    </form>
  );
}

/** Continue stays disabled until every field passes its rules. */
export const GatedForm: Story = { render: () => <Gated /> };
