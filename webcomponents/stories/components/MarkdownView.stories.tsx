import type { Meta, StoryObj } from "@storybook/react-vite";
import { MarkdownView } from "../../src/components/MarkdownView";

const TICKET_BODY = `# Feature: Serve price tiers

Expose the new \`price_tiers\` table over the API.

## Acceptance

- [x] Schema migrated
- [ ] \`GET /tiers\` returns every tier
- [ ] Docs updated

| Tier | Monthly |
|------|---------|
| Free | $0 |
| Team | $20 |

\`\`\`rust
fn tiers() -> Vec<Tier> { load() }
\`\`\`

## History

- 2026-10-05 12:00 plan completed
`;

const meta = {
  title: "Components/MarkdownView",
  component: MarkdownView,
  args: { source: TICKET_BODY },
} satisfies Meta<typeof MarkdownView>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A ticket body: headings, a task list, a table, code and history. */
export const TicketBody: Story = {};

/** Raw HTML in a ticket is shown as nothing, never executed. */
export const IgnoresRawHtml: Story = {
  args: { source: "Before\n\n<script>alert(1)</script>\n\nAfter" },
};

export const DarkTheme: Story = { globals: { theme: "dark" } };
