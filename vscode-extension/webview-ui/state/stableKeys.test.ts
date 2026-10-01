import { describe, expect, test } from "bun:test";
import { EMPTY_STABLE_KEYS, reconcileStableKeys } from "./stableKeys";

describe("reconcileStableKeys", () => {
  test("assigns a fresh id per key", () => {
    const state = reconcileStableKeys(EMPTY_STABLE_KEYS, ["a", "b"]);
    expect(state.ids.get("a")).not.toBe(state.ids.get("b"));
  });

  test("returns the same state when keys are unchanged", () => {
    const state = reconcileStableKeys(EMPTY_STABLE_KEYS, ["a"]);
    expect(reconcileStableKeys(state, ["a"])).toBe(state);
  });

  test("a single rename keeps the original id", () => {
    const state = reconcileStableKeys(EMPTY_STABLE_KEYS, ["old.atlassian.net"]);
    const renamed = reconcileStableKeys(state, ["new.atlassian.net"]);
    expect(renamed.ids.get("new.atlassian.net")).toBe(state.ids.get("old.atlassian.net"));
    expect(renamed.ids.has("old.atlassian.net")).toBe(false);
  });

  test("repeated renames keep the original id", () => {
    let state = reconcileStableKeys(EMPTY_STABLE_KEYS, ["a"]);
    const original = state.ids.get("a");
    for (const key of ["ab", "abc", "abcd"]) {
      state = reconcileStableKeys(state, [key]);
    }
    expect(state.ids.get("abcd")).toBe(original);
  });

  test("an added key alongside existing ones gets a new id", () => {
    const state = reconcileStableKeys(EMPTY_STABLE_KEYS, ["a"]);
    const next = reconcileStableKeys(state, ["a", "b"]);
    expect(next.ids.get("a")).toBe(state.ids.get("a"));
    expect(next.ids.get("b")).not.toBe(state.ids.get("a"));
  });

  test("ids are never reused after removal", () => {
    const state = reconcileStableKeys(EMPTY_STABLE_KEYS, ["a", "b"]);
    const removed = reconcileStableKeys(state, ["a"]);
    const added = reconcileStableKeys(removed, ["a", "c"]);
    expect(added.ids.get("c")).not.toBe(state.ids.get("b"));
  });
});
