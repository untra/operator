import { describe, expect, test } from "bun:test";
import { applyUpdate } from "./applyUpdate";
import { DEFAULT_WEBVIEW_CONFIG } from "../types/defaults";
import type { WebviewConfig } from "../types/messages";
import type { JiraConfig } from "../../src/generated/JiraConfig";
import type { LinearConfig } from "../../src/generated/LinearConfig";

const JIRA_SECTION = "kanban.jira";
const LINEAR_SECTION = "kanban.linear";
const OLD_DOMAIN = "old.atlassian.net";
const NEW_DOMAIN = "new.atlassian.net";
const SECOND_DOMAIN = "second.atlassian.net";
const OLD_TEAM = "team-old";
const NEW_TEAM = "team-new";
const SECOND_TEAM = "team-second";
const PROJECT = "PROJ";

const JIRA_WORKSPACE: JiraConfig = {
  enabled: true,
  api_key_env: "JIRA_KEY",
  email: "me@example.com",
  projects: {
    [PROJECT]: {
      sync_user_id: "u1",
      status_mapping: { todo: "To Do" },
      collection_name: "dev",
      type_mappings: {},
      bidirectional: false,
    },
  },
};

const LINEAR_WORKSPACE: LinearConfig = {
  enabled: true,
  api_key_env: "LINEAR_KEY",
  projects: {},
};

const SECOND_JIRA_WORKSPACE: JiraConfig = {
  enabled: true,
  api_key_env: "SECOND_JIRA_KEY",
  email: "second@example.com",
  projects: {},
};

const SECOND_LINEAR_WORKSPACE: LinearConfig = {
  enabled: true,
  api_key_env: "SECOND_LINEAR_KEY",
  projects: {},
};

function withKanban(): WebviewConfig {
  return {
    ...DEFAULT_WEBVIEW_CONFIG,
    config: {
      ...DEFAULT_WEBVIEW_CONFIG.config,
      kanban: {
        ...DEFAULT_WEBVIEW_CONFIG.config.kanban,
        jira: { [OLD_DOMAIN]: JIRA_WORKSPACE },
        linear: { [OLD_TEAM]: LINEAR_WORKSPACE },
      },
    },
  };
}

function withMultipleKanban(): WebviewConfig {
  const config = withKanban();
  return {
    ...config,
    config: {
      ...config.config,
      kanban: {
        ...config.config.kanban,
        jira: {
          ...config.config.kanban.jira,
          [SECOND_DOMAIN]: SECOND_JIRA_WORKSPACE,
        },
        linear: {
          ...config.config.kanban.linear,
          [SECOND_TEAM]: SECOND_LINEAR_WORKSPACE,
        },
      },
    },
  };
}

describe("applyUpdate", () => {
  test("projects.<key>.<field> updates only that project field", () => {
    const next = applyUpdate(
      withKanban(),
      JIRA_SECTION,
      `projects.${PROJECT}.collection_name`,
      "ops",
      OLD_DOMAIN,
    );
    const project = next.config.kanban.jira[OLD_DOMAIN]?.projects[PROJECT];
    expect(project?.collection_name).toBe("ops");
    expect(project?.sync_user_id).toBe("u1");
    expect(project?.status_mapping).toEqual({ todo: "To Do" });
  });

  test("projects.<key>.<field> creates a missing project from defaults", () => {
    const next = applyUpdate(
      withKanban(),
      LINEAR_SECTION,
      "projects.ENG.collection_name",
      "",
      OLD_TEAM,
    );
    expect(next.config.kanban.linear[OLD_TEAM]?.projects.ENG?.collection_name).toBe("");
    expect(next.config.kanban.linear[OLD_TEAM]?.api_key_env).toBe("LINEAR_KEY");
  });

  test("jira domain rename preserves the workspace entry", () => {
    const next = applyUpdate(withKanban(), JIRA_SECTION, "domain", NEW_DOMAIN, OLD_DOMAIN);
    expect(Object.keys(next.config.kanban.jira)).toEqual([NEW_DOMAIN]);
    expect(next.config.kanban.jira[NEW_DOMAIN]).toEqual(JIRA_WORKSPACE);
  });

  test("linear team_id rename preserves the workspace entry", () => {
    const next = applyUpdate(withKanban(), LINEAR_SECTION, "team_id", NEW_TEAM, OLD_TEAM);
    expect(Object.keys(next.config.kanban.linear)).toEqual([NEW_TEAM]);
    expect(next.config.kanban.linear[NEW_TEAM]).toEqual(LINEAR_WORKSPACE);
  });

  test("instance scalar edits only the selected workspace without mutating the input", () => {
    const input = withMultipleKanban();
    const next = applyUpdate(input, JIRA_SECTION, "email", "updated@example.com", SECOND_DOMAIN);
    expect(next.config.kanban.jira[OLD_DOMAIN]?.email).toBe("me@example.com");
    expect(next.config.kanban.jira[SECOND_DOMAIN]?.email).toBe("updated@example.com");
    expect(input.config.kanban.jira[OLD_DOMAIN]?.email).toBe("me@example.com");
    expect(input.config.kanban.jira[SECOND_DOMAIN]?.email).toBe("second@example.com");
  });

  test("project field edits only the selected workspace", () => {
    const next = applyUpdate(
      withMultipleKanban(),
      LINEAR_SECTION,
      "projects.ENG.collection_name",
      "engineering",
      SECOND_TEAM,
    );
    expect(next.config.kanban.linear[OLD_TEAM]?.projects.ENG).toBeUndefined();
    expect(next.config.kanban.linear[SECOND_TEAM]?.projects.ENG?.collection_name).toBe(
      "engineering",
    );
  });

  test("jira domain rename moves only the selected workspace", () => {
    const next = applyUpdate(
      withMultipleKanban(),
      JIRA_SECTION,
      "domain",
      NEW_DOMAIN,
      SECOND_DOMAIN,
    );
    expect(next.config.kanban.jira[OLD_DOMAIN]).toEqual(JIRA_WORKSPACE);
    expect(next.config.kanban.jira[SECOND_DOMAIN]).toBeUndefined();
    expect(next.config.kanban.jira[NEW_DOMAIN]).toEqual(SECOND_JIRA_WORKSPACE);
  });

  test("linear team rename moves only the selected workspace", () => {
    const next = applyUpdate(
      withMultipleKanban(),
      LINEAR_SECTION,
      "team_id",
      NEW_TEAM,
      SECOND_TEAM,
    );
    expect(next.config.kanban.linear[OLD_TEAM]).toEqual(LINEAR_WORKSPACE);
    expect(next.config.kanban.linear[SECOND_TEAM]).toBeUndefined();
    expect(next.config.kanban.linear[NEW_TEAM]).toEqual(SECOND_LINEAR_WORKSPACE);
  });

  test("missing and colliding instance targets do not overwrite existing workspaces", () => {
    const input = withMultipleKanban();
    const missingTarget = applyUpdate(
      input,
      JIRA_SECTION,
      "email",
      "wrong@example.com",
      "missing.atlassian.net",
    );
    const collision = applyUpdate(input, JIRA_SECTION, "domain", OLD_DOMAIN, SECOND_DOMAIN);
    expect(missingTarget.config.kanban.jira).toEqual(input.config.kanban.jira);
    expect(collision.config.kanban.jira).toEqual(input.config.kanban.jira);
  });

  test("bigint fields stay bigint", () => {
    const next = applyUpdate(withKanban(), "agents", "step_timeout", BigInt(900));
    expect(typeof next.config.agents.step_timeout).toBe("bigint");
    expect(next.config.agents.step_timeout).toBe(BigInt(900));
    expect(typeof next.config.agents.generation_timeout_secs).toBe("bigint");
  });

  test("primary working_directory updates the wrapper field", () => {
    const next = applyUpdate(withKanban(), "primary", "working_directory", "/repos");
    expect(next.working_directory).toBe("/repos");
  });

  test("git.github merges into the nested github table", () => {
    const next = applyUpdate(withKanban(), "git.github", "token_env", "GH");
    expect(next.config.git.github.token_env).toBe("GH");
    expect(next.config.git.github.enabled).toBe(DEFAULT_WEBVIEW_CONFIG.config.git.github.enabled);
  });
});
