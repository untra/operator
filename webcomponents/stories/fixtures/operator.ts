import type { DelegatorResponse } from "../../src/generated/DelegatorResponse";
import type { HealthResponse } from "../../src/generated/HealthResponse";
import type { IssueType } from "../../src/generated/IssueType";
import type { IssueTypeResponse } from "../../src/generated/IssueTypeResponse";
import type { IssueTypeSummary } from "../../src/generated/IssueTypeSummary";
import type { KanbanBoardResponse } from "../../src/generated/KanbanBoardResponse";
import type { KanbanTicketCard } from "../../src/generated/KanbanTicketCard";
import type { QueueStatusResponse } from "../../src/generated/QueueStatusResponse";
import type { SectionDto } from "../../src/generated/SectionDto";
import type { StepSchema } from "../../src/generated/StepSchema";

export const FIXED_UPDATED_LABEL = "10:30:00 AM";
export const BRAND_ICON_SRC = (name: string) => `/icons/${name}.svg`;

export const queuedTicket: KanbanTicketCard = {
  id: "FEAT-1042",
  summary: "Add audit history to the workspace settings page",
  ticket_type: "FEAT",
  project: "operator",
  status: "queued",
  step: "plan",
  step_display_name: "Plan",
  priority: "P1-high",
  timestamp: "20260914-0930",
  filename: "FEAT-1042.md",
};

export const runningTicket: KanbanTicketCard = {
  id: "FIX-318",
  summary: "Prevent stale queue results after switching projects",
  ticket_type: "FIX",
  project: "operator",
  status: "running",
  step: "implement",
  step_display_name: "Implementation",
  priority: "P0-critical",
  timestamp: "20260914-0945",
  filename: "FIX-318.md",
};

export const awaitingTicket: KanbanTicketCard = {
  id: "INV-77",
  summary: "Compare provider retry behavior for long-running sessions",
  ticket_type: "INV",
  project: "platform",
  status: "awaiting",
  step: "review",
  step_display_name: "Human Review",
  priority: "P2-medium",
  timestamp: "20260914-1000",
  filename: "INV-77.md",
};

export const completedTicket: KanbanTicketCard = {
  id: "SPIKE-51",
  summary: "Prototype responsive workflow graph layout",
  ticket_type: "SPIKE",
  project: "operator",
  status: "completed",
  step: "done",
  step_display_name: "Complete",
  priority: "P3-low",
  timestamp: "20260914-1015",
  filename: "SPIKE-51.md",
};

/** `step_display_name` is nullable; the board falls back to the raw `step`. */
export const unnamedStepTicket: KanbanTicketCard = {
  id: "TASK-204",
  summary: "Rotate the signing key used by the release pipeline",
  ticket_type: "TASK",
  project: "operator",
  status: "queued",
  step: "implement",
  step_display_name: null,
  priority: "P2-medium",
  timestamp: "20260914-1020",
  filename: "TASK-204.md",
};

export const board: KanbanBoardResponse = {
  queue: [queuedTicket],
  running: [runningTicket],
  awaiting: [awaitingTicket],
  done: [completedTicket],
  total_count: 4,
  last_updated: "2026-09-14T16:30:00Z",
};

/** Enough cards across projects, types and priorities to exercise filtering. */
export const busyBoard: KanbanBoardResponse = {
  queue: [
    queuedTicket,
    unnamedStepTicket,
    {
      ...queuedTicket,
      id: "FEAT-1043",
      summary: "Paginate the leaderboard endpoint",
      project: "gamesvc",
      priority: "P0-critical",
      timestamp: "20260914-0935",
      filename: "FEAT-1043.md",
    },
  ],
  running: [
    runningTicket,
    {
      ...runningTicket,
      id: "FIX-319",
      summary: "Retry webhook delivery on 5xx",
      project: "platform",
      priority: "P3-low",
      timestamp: "20260914-0950",
      filename: "FIX-319.md",
    },
  ],
  awaiting: [awaitingTicket],
  done: [completedTicket],
  total_count: 7,
  last_updated: "2026-09-14T16:30:00Z",
};

export const emptyBoard: KanbanBoardResponse = {
  queue: [],
  running: [],
  awaiting: [],
  done: [],
  total_count: 0,
  last_updated: "2026-09-14T16:30:00Z",
};

export const health: HealthResponse = {
  status: "ok",
  version: "0.18.0",
  directory_name: "operator",
  directory_id: "workspace-fixture",
};

export const queueStatus: QueueStatusResponse = {
  queued: 1,
  in_progress: 1,
  awaiting: 1,
  completed: 1,
  by_type: { INV: 1, FIX: 1, TASK: 0, FEAT: 1, SPIKE: 1 },
};

const createStep = (
  name: string,
  displayName: string,
  nextStep: string | null,
  reviewType: StepSchema["review_type"] = "none",
): StepSchema => ({
  name,
  display_name: displayName,
  type: "task",
  outputs: [name === "implement" ? "code" : "report"],
  prompt: `Complete the ${displayName.toLowerCase()} step.`,
  review_type: reviewType,
  next_step: nextStep,
  allowed_tools: ["Read", "Write"],
  permission_mode: "default",
  artifact_patterns: [],
});

export const featureWorkflow: IssueType = {
  key: "FEAT",
  name: "Feature",
  description: "Plan, implement, and review a user-facing product capability.",
  mode: "autonomous",
  glyph: "◆",
  color: "#4f8cff",
  project_required: true,
  fields: [],
  steps: [
    createStep("plan", "Plan", "implement", "plan"),
    createStep("implement", "Implementation", "review"),
    createStep("review", "Pull Request Review", null, "pr"),
  ],
  source: "builtin",
};

export const emptyWorkflow: IssueType = {
  ...featureWorkflow,
  key: "NOTE",
  name: "Note",
  description: "Capture information without an execution workflow.",
  glyph: "●",
  steps: [],
};

export const longWorkflow: IssueType = {
  ...featureWorkflow,
  key: "RELEASE",
  name: "Release",
  description: "Coordinate a complete release across validation and delivery stages.",
  glyph: "⬢",
  steps: [
    createStep("scope", "Scope", "plan", "plan"),
    createStep("plan", "Plan", "implement", "plan"),
    createStep("implement", "Implementation", "verify"),
    createStep("verify", "Verification", "document", "proof"),
    createStep("document", "Documentation", "release", "visual"),
    createStep("release", "Release", null, "pr"),
  ],
};

export const issueTypeSummaries: IssueTypeSummary[] = [
  {
    key: "FEAT",
    name: "Feature",
    description: featureWorkflow.description,
    mode: "autonomous",
    glyph: "◆",
    source: "builtin",
    stepCount: 3,
  },
  {
    key: "FIX",
    name: "Bug Fix",
    description: "Diagnose and correct a reproducible defect.",
    mode: "paired",
    glyph: "▲",
    source: "user",
    stepCount: 3,
  },
  {
    key: "INV",
    name: "Investigation with a deliberately long display name",
    description: "Gather evidence before choosing an implementation path.",
    mode: "autonomous",
    glyph: "◉",
    source: "builtin",
    stepCount: 2,
  },
];

export const selectedIssueType: IssueTypeResponse = {
  key: "FEAT",
  name: "Feature",
  description: featureWorkflow.description,
  mode: "autonomous",
  glyph: "◆",
  color: "#4f8cff",
  project_required: true,
  source: "builtin",
  fields: [],
  steps: featureWorkflow.steps.map((step) => ({
    name: step.name,
    display_name: step.display_name ?? null,
    prompt: step.prompt,
    outputs: step.outputs,
    allowed_tools: step.allowed_tools,
    review_type: step.review_type,
    next_step: step.next_step ?? null,
    permission_mode: step.permission_mode,
  })),
};

export const delegators: DelegatorResponse[] = [
  {
    name: "claude-opus",
    llm_tool: "claude",
    model: "opus",
    display_name: "Claude Opus",
    model_properties: {},
    model_server: null,
    launch_config: null,
    remote_agent: null,
  },
  {
    name: "local-coder",
    llm_tool: "ollama",
    model: "qwen3-coder",
    display_name: "Local Coder",
    model_properties: { context_window: "32768" },
    model_server: "workstation",
    launch_config: null,
    remote_agent: null,
  },
];

/** `display_name` is nullable; the launch form falls back to `name`. */
export const unnamedDelegator: DelegatorResponse = {
  name: "codex-local",
  llm_tool: "codex",
  model: "gpt-5-codex",
  display_name: null,
  model_properties: {},
  model_server: null,
  launch_config: null,
  remote_agent: null,
};

export const healthySection: SectionDto = {
  id: "models",
  label: "Model Providers",
  health: "green",
  description: "Providers and models available to delegators.",
  prerequisites: [],
  met: true,
  children: [
    {
      id: "ollama",
      depth: 1,
      label: "Ollama",
      description: "2 local models available",
      icon: "server",
      brand_icon: "ollama",
      health: "green",
      actions: [{ label: "Open Web UI", url: "https://example.test/ollama" }],
    },
    {
      id: "qwen3-coder",
      depth: 2,
      label: "qwen3-coder",
      description: "32k context window",
      icon: "symbol-method",
      brand_icon: null,
      health: "green",
      actions: [],
    },
  ],
};

export const lockedSection: SectionDto = {
  id: "kanban",
  label: "Kanban",
  health: "gray",
  description: "Ticket lifecycle and work queues.",
  prerequisites: ["models", "projects"],
  met: false,
  children: [],
};

/** Unhealthy rows with multiple actions and long text, for wrapping and health colors. */
export const degradedSection: SectionDto = {
  id: "projects",
  label: "Projects",
  health: "red",
  description: "Repositories operator can launch agents against.",
  prerequisites: [],
  met: true,
  children: [
    {
      id: "operator",
      depth: 1,
      label: "operator",
      description:
        "Working tree has uncommitted changes on a branch that is behind its upstream, so worktree creation will fail until it is reconciled.",
      icon: "repo",
      brand_icon: "github",
      health: "red",
      actions: [
        { label: "Open repository", url: "https://example.test/operator" },
        { label: "View branches", url: "https://example.test/operator/branches" },
        { label: "Reconcile", url: "https://example.test/operator/reconcile" },
      ],
    },
    {
      id: "platform",
      depth: 1,
      label: "platform",
      description: "No agent marker file (CLAUDE.md, GEMINI.md, CODEX.md) was found at the root.",
      icon: "repo",
      brand_icon: "gitlab",
      health: "yellow",
      actions: [{ label: "Open repository", url: "https://example.test/platform" }],
    },
    {
      id: "platform-docs",
      depth: 3,
      label: "platform/docs",
      description: "Nested deeply enough to exercise the indent ramp.",
      icon: "book",
      brand_icon: null,
      health: "gray",
      actions: [],
    },
  ],
};

/** Prerequisites unmet *and* children present - the card renders both. */
export const lockedSectionWithChildren: SectionDto = {
  ...lockedSection,
  children: [
    {
      id: "jira",
      depth: 1,
      label: "Jira",
      description: "Configured, but unreachable until model providers are connected.",
      icon: "link",
      brand_icon: "atlassian",
      health: "gray",
      actions: [],
    },
  ],
};

/** Collection manifest entries, as the hosted bundle publishes them. */
export const manifestEntries = [
  { key: "FEAT", schema_path: "issuetypes/FEAT.json" },
  { key: "NOTE", schema_path: "issuetypes/NOTE.json" },
  { key: "RELEASE", schema_path: "issuetypes/RELEASE.json" },
];
