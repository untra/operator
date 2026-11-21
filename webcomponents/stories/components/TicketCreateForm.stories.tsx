import { useCallback, useState } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";
import {
  TicketCreateForm,
  type TicketCreateFormValue,
} from "../../src/components/TicketCreateForm";
import { issueTypeSummaries, projectSummary } from "../fixtures/operator";

const meta = {
  title: "Components/TicketCreateForm",
  component: TicketCreateForm,
  args: {
    value: { issueType: "", project: "", summary: "" },
    issueTypes: issueTypeSummaries,
    projects: [projectSummary],
    onChange: fn(),
    onSubmit: fn(),
  },
  render: function Stateful(args) {
    const [value, setValue] = useState(args.value);
    const onChange = useCallback(
      (next: TicketCreateFormValue) => {
        setValue(next);
        args.onChange(next);
      },
      [args],
    );
    return (
      <div className="story-bounded story-bounded-form">
        <TicketCreateForm {...args} value={value} onChange={onChange} />
      </div>
    );
  },
} satisfies Meta<typeof TicketCreateForm>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Empty: Story = {};
export const Populated: Story = {
  args: { value: { issueType: "FEAT", project: "operator", summary: "Make status accessible" } },
};
export const Busy: Story = { ...Populated, args: { ...Populated.args, busy: true } };
export const Error: Story = {
  ...Populated,
  args: { ...Populated.args, error: "Could not create ticket." },
};
export const MissingOptions: Story = { args: { issueTypes: [], projects: [] } };
