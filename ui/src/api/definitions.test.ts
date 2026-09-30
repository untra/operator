import { describe, expect, test } from "bun:test";
import { initializeSetupMutation, revokeSessionMutation } from "./definitions";
import { QUERY_KEYS } from "./queries";

describe("mutation invalidation definitions", () => {
  test("revoking a session refreshes the list and current principal", () => {
    expect(revokeSessionMutation.affected).toEqual([
      QUERY_KEYS.sessions,
      QUERY_KEYS.currentSession,
    ]);
  });

  test("setup initialization names its affected query families", () => {
    expect(initializeSetupMutation.affected).not.toBe("*");
    expect(initializeSetupMutation.affected).toContain(QUERY_KEYS.setupStatus);
    expect(initializeSetupMutation.affected).toContain(QUERY_KEYS.configuration);
    expect(initializeSetupMutation.affected).toContain(QUERY_KEYS.issueTypes);
    expect(initializeSetupMutation.affected).not.toContain(QUERY_KEYS.currentSession);
  });
});
