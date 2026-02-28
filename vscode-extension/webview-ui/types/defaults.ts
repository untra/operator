import type { WebviewConfig } from './messages';
import type { Config } from '../../src/generated/Config';

/** Sensible defaults matching Rust Config::default() */
const DEFAULT_CONFIG: Config = {
  projects: [],
  agents: {
    max_parallel: 2,
    cores_reserved: 1,
    health_check_interval: BigInt(30),
    generation_timeout_secs: BigInt(300),
    sync_interval: BigInt(60),
    step_timeout: BigInt(1800),
    silence_threshold: BigInt(30),
  },
  notifications: {
    enabled: true,
    os: { enabled: true, sound: false, events: [] },
    webhook: null,
    webhooks: [],
  },
  queue: {
    auto_assign: true,
    priority_order: ['INV', 'FIX', 'FEAT', 'SPIKE'],
    poll_interval_ms: BigInt(2000),
  },
  paths: {
    tickets: '.tickets',
    projects: '.',
    state: '.tickets/operator',
    worktrees: '.worktrees',
  },
  ui: {
    refresh_rate_ms: BigInt(1000),
    completed_history_hours: BigInt(24),
    summary_max_length: 80,
    panel_names: {
      queue: 'Queue',
      agents: 'Agents',
      awaiting: 'Awaiting',
      completed: 'Completed',
    },
  },
  launch: {
    confirm_autonomous: false,
    confirm_paired: true,
    launch_delay_ms: BigInt(500),
    docker: {
      enabled: false,
      image: '',
      extra_args: [],
      mount_path: '/workspace',
      env_vars: [],
    },
    yolo: { enabled: false },
  },
  templates: {
    preset: 'dev_kanban',
    collection: [],
    active_collection: null,
  },
  api: {
    pr_check_interval_secs: BigInt(300),
    rate_limit_check_interval_secs: BigInt(60),
    rate_limit_warning_threshold: 80,
  },
  logging: {
    level: 'info',
    to_file: false,
  },
  tmux: {
    config_generated: false,
  },
  sessions: {
    wrapper: 'vscode',
    tmux: {
      config_generated: false,
      socket_name: 'operator',
    },
    vscode: {
      webhook_port: 7007,
      connect_timeout_ms: BigInt(5000),
    },
  },
  llm_tools: {
    detected: [],
    providers: [],
    detection_complete: false,
    skill_directory_overrides: {},
  },
  backstage: {
    enabled: false,
    port: 7009,
    auto_start: false,
    subpath: '/backstage',
    branding_subpath: '/branding',
    release_url: '',
    local_binary_path: null,
    branding: {
      app_title: 'Operator',
      org_name: '',
      logo_path: null,
      colors: {
        primary: '#4f46e5',
        secondary: '#7c3aed',
        accent: '#06b6d4',
        warning: '#f59e0b',
        muted: '#6b7280',
      },
    },
  },
  rest_api: {
    enabled: false,
    port: 7008,
    cors_origins: [],
  },
  git: {
    provider: null,
    github: { enabled: true, token_env: 'GITHUB_TOKEN' },
    gitlab: { enabled: false, token_env: 'GITLAB_TOKEN', host: null },
    branch_format: '{type}/{ticket_id}-{slug}',
    use_worktrees: false,
  },
  kanban: {
    jira: {},
    linear: {},
  },
  version_check: {
    enabled: true,
    url: null,
    timeout_secs: BigInt(10),
  },
  delegators: [],
};

export const DEFAULT_WEBVIEW_CONFIG: WebviewConfig = {
  config_path: '',
  working_directory: '',
  config: DEFAULT_CONFIG,
};
