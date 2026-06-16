// The SPA's reference to the canonical concept→icon table documented in
// docs/design-system/ ("/design-system/"). Keyed by the SectionId serde rename
// from src/ui/status_panel.rs for status concepts, plus the web-only Dashboard
// and Queue pages. Each concept carries its codicon, sidebar/header label, the
// HashRouter route it owns, an absolute docs URL, and a one-line summary (the
// only prose rendered at the top of each page).
//
// Icons come from @vscode/codicons (MIT-licensed code, CC-BY-4.0 icons). This is
// the single place the SPA maps a concept to an icon — keep it in sync with the
// canonical table in the design-system docs.

const DOCS_BASE = 'https://operator.untra.io';

export interface Concept {
  /** SectionId serde rename for status concepts; route id for web-only pages. */
  key: string;
  /** codicon name (without the `codicon-` prefix). */
  icon: string;
  /** Sidebar and page-header label. */
  label: string;
  /** HashRouter route this concept owns (e.g. "/git", "/", "/queue"). */
  route: string;
  /** Absolute docs URL (operator.untra.io). Must resolve to a real docs page. */
  docsUrl: string;
  /** One-line page summary — the only prose at the top of the page. */
  summary: string;
}

export const CONCEPTS: Record<string, Concept> = {
  // --- Web-only pages (no status section) ---
  dashboard: {
    key: 'dashboard',
    icon: 'dashboard',
    label: 'Dashboard',
    route: '/',
    docsUrl: `${DOCS_BASE}/`,
    summary: 'At-a-glance queue counts, the kanban board, and active agents.',
  },
  queue: {
    key: 'queue',
    icon: 'list-ordered',
    label: 'Queue',
    route: '/queue',
    docsUrl: `${DOCS_BASE}/tickets/`,
    summary: 'The full kanban board of tickets across todo, in-progress, and done.',
  },

  // --- Status sections (mirror SectionId in src/ui/status_panel.rs) ---
  config: {
    key: 'config',
    icon: 'settings-gear',
    label: 'Configuration',
    route: '/config',
    docsUrl: `${DOCS_BASE}/configuration/`,
    summary: 'Operator configuration, collections, and managed projects.',
  },
  connections: {
    key: 'connections',
    icon: 'plug',
    label: 'Connections',
    route: '/connections',
    docsUrl: `${DOCS_BASE}/configuration/`,
    summary: 'Connectivity to the operator API and webhook endpoints.',
  },
  kanban: {
    key: 'kanban',
    icon: 'layout',
    label: 'Kanban',
    route: '/kanban',
    docsUrl: `${DOCS_BASE}/kanban/`,
    summary: 'Kanban provider wiring that backs the ticket board.',
  },
  llm: {
    key: 'llm',
    icon: 'sparkle',
    label: 'LLM Tools',
    route: '/llm',
    docsUrl: `${DOCS_BASE}/llm-tools/`,
    summary: 'Detected LLM CLIs and tools available to launch agents.',
  },
  'model-servers': {
    key: 'model-servers',
    icon: 'server',
    label: 'Model Providers',
    route: '/model-providers',
    docsUrl: `${DOCS_BASE}/getting-started/model-servers/`,
    summary: 'Connect model providers and list their models for delegators.',
  },
  git: {
    key: 'git',
    icon: 'git-branch',
    label: 'Git',
    route: '/git',
    docsUrl: `${DOCS_BASE}/getting-started/git/`,
    summary: 'Git provider and token wiring for branch and PR operations.',
  },
  issuetypes: {
    key: 'issuetypes',
    icon: 'issues',
    label: 'Issue Types',
    route: '/issuetypes',
    docsUrl: `${DOCS_BASE}/issue-types/`,
    summary: 'The catalog of issue types, their modes, and workflow steps.',
  },
  delegators: {
    key: 'delegators',
    icon: 'rocket',
    label: 'Delegators',
    route: '/delegators',
    docsUrl: `${DOCS_BASE}/delegators/`,
    summary: 'Delegators that launch and supervise agents on your behalf.',
  },
  projects: {
    key: 'projects',
    icon: 'project',
    label: 'Managed Projects',
    route: '/projects',
    docsUrl: `${DOCS_BASE}/configuration/`,
    summary: 'Projects operator manages and routes tickets into.',
  },
  workflows: {
    key: 'workflows',
    icon: 'type-hierarchy',
    label: 'Workflows',
    route: '/workflows',
    docsUrl: `${DOCS_BASE}/getting-started/workflows/`,
    summary: 'Export formats a ticket + issue type can be rendered into for other tools.',
  },
};

/** Sidebar order for the status sections (matches the TUI / VS Code ordering). */
export const STATUS_KEYS = [
  'config',
  'connections',
  'kanban',
  'llm',
  'model-servers',
  'git',
  'issuetypes',
  'delegators',
  'projects',
  'workflows',
] as const;

/** Sidebar order for the web-only pages. */
export const PAGE_KEYS = ['dashboard', 'queue'] as const;
