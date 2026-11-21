import { describe, expect, mock, test } from "bun:test";
import * as webcomponentMocks from "../../test-webcomponents";
import type { WizardDraft } from "./types";

await mock.module("@operator/webcomponents", () => webcomponentMocks);

const { coderParameterNameRules, kanbanCredentialsValid, stepValid } = await import("./validation");

const DRAFT: WizardDraft = {
  configurationName: "default",
  executionMode: "local",
  premium: false,
  preset: "simple",
  taskFields: [],
  wrapper: "tmux",
  executionTarget: { kind: "local" },
  coderParameters: [],
  useWorktrees: false,
  acceptanceCriteria: "",
  modelServers: [],
  hostedCollectionIds: [],
};

const coder = (name: string, template: string): WizardDraft["executionTarget"] => ({
  kind: "coder",
  name,
  template,
  parameters: {},
});

const params = (...names: string[]) => names.map((name, id) => ({ id, name, value: "v" }));

describe("stepValid welcome", () => {
  test("accepts a slug name", () => {
    expect(stepValid("welcome", { ...DRAFT, configurationName: "untra-casa_1" })).toBe(true);
  });
  test("rejects dots, uppercase, empty and overlong names", () => {
    for (const configurationName of ["untra.casa", "Untra", "", "a".repeat(65)]) {
      expect(stepValid("welcome", { ...DRAFT, configurationName })).toBe(false);
    }
  });
});

describe("stepValid hosted-collections", () => {
  test("requires at least one collection", () => {
    expect(stepValid("hosted-collections", DRAFT)).toBe(false);
    expect(stepValid("hosted-collections", { ...DRAFT, hostedCollectionIds: ["a"] })).toBe(true);
  });
});

describe("stepValid execution-target", () => {
  test("local target is valid", () => {
    expect(stepValid("execution-target", DRAFT)).toBe(true);
  });
  test("coder target needs name and template", () => {
    expect(stepValid("execution-target", { ...DRAFT, executionTarget: coder("", "t") })).toBe(
      false,
    );
    expect(stepValid("execution-target", { ...DRAFT, executionTarget: coder("n", " ") })).toBe(
      false,
    );
    expect(stepValid("execution-target", { ...DRAFT, executionTarget: coder("n", "t") })).toBe(
      true,
    );
  });
  test("coder parameter names must be present and unique", () => {
    const base = { ...DRAFT, executionTarget: coder("n", "t") };
    expect(stepValid("execution-target", { ...base, coderParameters: params("a", "") })).toBe(
      false,
    );
    expect(stepValid("execution-target", { ...base, coderParameters: params("a", " a") })).toBe(
      false,
    );
    expect(stepValid("execution-target", { ...base, coderParameters: params("a", "b") })).toBe(
      true,
    );
  });
});

describe("coderParameterNameRules", () => {
  test("flags a duplicate of another row only", () => {
    const rules = coderParameterNameRules(["a", "b", "a"], 2);
    expect(webcomponentMocks.validate("a", rules)).toBe("Parameter names must be unique");
    expect(webcomponentMocks.validate("c", rules)).toBeNull();
  });
});

describe("stepValid other steps", () => {
  test("steps without field rules are valid", () => {
    expect(stepValid("session-wrapper-choice", DRAFT)).toBe(true);
  });
});

const CREDENTIALS = {
  domain: "",
  email: "",
  instance: "",
  repository: "",
  rootPath: "",
  token: "",
};

describe("kanbanCredentialsValid", () => {
  test("token providers need only a token", () => {
    expect(kanbanCredentialsValid("github", CREDENTIALS)).toBe(false);
    expect(kanbanCredentialsValid("linear", { ...CREDENTIALS, token: "t" })).toBe(true);
  });
  test("jira needs domain, email and token", () => {
    const jira = { ...CREDENTIALS, domain: "org.atlassian.net", token: "t" };
    expect(kanbanCredentialsValid("jira", jira)).toBe(false);
    expect(kanbanCredentialsValid("jira", { ...jira, email: "nope" })).toBe(false);
    expect(kanbanCredentialsValid("jira", { ...jira, email: "a@b.co" })).toBe(true);
  });
  test("gitea needs an http url and owner/repository", () => {
    const gitea = {
      ...CREDENTIALS,
      instance: "home",
      domain: "https://git.example.com",
      repository: "owner/repo",
      token: "t",
    };
    expect(kanbanCredentialsValid("gitea", gitea)).toBe(true);
    expect(kanbanCredentialsValid("gitea", { ...gitea, domain: "git.example.com" })).toBe(false);
    expect(kanbanCredentialsValid("gitea", { ...gitea, repository: "repo" })).toBe(false);
  });
  test("openspec needs instance and root, not a token", () => {
    expect(kanbanCredentialsValid("openspec", { ...CREDENTIALS, instance: "i" })).toBe(false);
    expect(
      kanbanCredentialsValid("openspec", { ...CREDENTIALS, instance: "i", rootPath: "/specs" }),
    ).toBe(true);
  });
});
