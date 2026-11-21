import { describe, expect, mock, test } from "bun:test";
import * as webcomponentMocks from "./test-webcomponents";
import type { TargetDef } from "@operator/bindings/TargetDef";

await mock.module("@operator/webcomponents", () => webcomponentMocks);

const { envVarRules, gitDraftValid, targetDraftValid } = await import("./field-rules");
const { validate } = webcomponentMocks;

describe("envVarRules", () => {
  test("accepts upper snake case", () => {
    expect(validate("CODER_URL", envVarRules)).toBeNull();
  });
  test("rejects lowercase and leading digits", () => {
    expect(validate("coder_url", envVarRules)).toBe("Use an UPPER_SNAKE_CASE variable name");
    expect(validate("1URL", envVarRules)).toBe("Use an UPPER_SNAKE_CASE variable name");
  });
  test("requires a value", () => {
    expect(validate("", envVarRules)).toBe("Required");
  });
});

const SSH: TargetDef = { name: "box", kind: "ssh", ssh_alias: "box", workdir: "/src" };
const CODER: TargetDef = {
  name: "cdr",
  kind: "coder",
  template: "dev",
  url_env: "CODER_URL",
  token_env: "CODER_SESSION_TOKEN",
  name_prefix: "op",
  stop_on_complete: true,
  create_timeout_secs: 300n,
  delete_on_complete: false,
  delete_on_merge: false,
};

describe("targetDraftValid", () => {
  test("complete ssh and coder drafts are valid", () => {
    expect(targetDraftValid(SSH)).toBe(true);
    expect(targetDraftValid(CODER)).toBe(true);
  });
  test("ssh needs name, alias and workdir", () => {
    expect(targetDraftValid({ ...SSH, name: " " })).toBe(false);
    expect(targetDraftValid({ ...SSH, ssh_alias: "" })).toBe(false);
    expect(targetDraftValid({ ...SSH, workdir: "" })).toBe(false);
  });
  test("coder needs template and valid env names", () => {
    expect(targetDraftValid({ ...CODER, template: "" })).toBe(false);
    expect(targetDraftValid({ ...CODER, url_env: "coder url" })).toBe(false);
  });
});

const GIT = { identity: null, credentials: null, settings: [] };

describe("gitDraftValid", () => {
  test("no git customization is valid", () => {
    expect(gitDraftValid(null)).toBe(true);
    expect(gitDraftValid(GIT)).toBe(true);
  });
  test("identity needs name and email", () => {
    expect(gitDraftValid({ ...GIT, identity: { name: "bot", email: "" } })).toBe(false);
    expect(gitDraftValid({ ...GIT, identity: { name: "bot", email: "{ticket_id}@x" } })).toBe(true);
  });
  test("credentials need an http url, username and env var", () => {
    const credentials = {
      repository_url: "https://git.example.com/o/r.git",
      username: "bot",
      token_env: "AGENT_GIT_TOKEN",
    };
    expect(gitDraftValid({ ...GIT, credentials })).toBe(true);
    expect(gitDraftValid({ ...GIT, credentials: { ...credentials, token_env: "token" } })).toBe(
      false,
    );
    expect(
      gitDraftValid({ ...GIT, credentials: { ...credentials, repository_url: "git@x:o/r" } }),
    ).toBe(false);
  });
  test("settings need a key", () => {
    expect(gitDraftValid({ ...GIT, settings: [{ key: "", value: "v" }] })).toBe(false);
    expect(gitDraftValid({ ...GIT, settings: [{ key: "core.autocrlf", value: "" }] })).toBe(true);
  });
});
