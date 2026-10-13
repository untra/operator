import { describe, expect, test } from "bun:test";
import type { ConnectOption } from "@operator/bindings/ConnectOption";
import { connectAction } from "./session-links";

const base: ConnectOption = {
  interface: "vscode",
  kind: "ide",
  label: "Open in VS Code",
  available: true,
};

describe("connectAction", () => {
  test("ide option opens its uri", () => {
    expect(connectAction({ ...base, uri: "vscode://x/connect" })).toEqual({
      kind: "open-url",
      url: "vscode://x/connect",
    });
  });

  test("command option copies its command", () => {
    expect(
      connectAction({ ...base, interface: "tmux", kind: "command", command: "tmux attach -t s" }),
    ).toEqual({ kind: "copy", text: "tmux attach -t s" });
  });

  test("focus option calls the api", () => {
    expect(connectAction({ ...base, interface: "cmux", kind: "focus_api" })).toEqual({
      kind: "focus-api",
    });
  });

  test("unavailable option does nothing", () => {
    expect(connectAction({ ...base, uri: "vscode://x", available: false })).toEqual({
      kind: "none",
    });
  });
});
