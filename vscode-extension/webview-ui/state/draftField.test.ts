import { describe, expect, test } from "bun:test";
import { initialDraft, syncDraft, withDraft } from "./draftField";

describe("syncDraft", () => {
  test("follows an external value change while the draft is clean", () => {
    const state = syncDraft(initialDraft("a"), "b");
    expect(state).toEqual({ draft: "b", synced: "b" });
  });

  test("keeps in-progress edits when the external value changes", () => {
    const editing = withDraft(initialDraft("a"), "typed");
    expect(syncDraft(editing, "b")).toEqual({ draft: "typed", synced: "b" });
  });

  test("returns the same state when the value is unchanged", () => {
    const state = withDraft(initialDraft("a"), "typed");
    expect(syncDraft(state, "a")).toBe(state);
  });

  test("rolls back to the external value after a committed edit is reverted", () => {
    const committed = syncDraft(withDraft(initialDraft("old"), "new"), "new");
    expect(committed).toEqual({ draft: "new", synced: "new" });
    expect(syncDraft(committed, "old")).toEqual({ draft: "old", synced: "old" });
  });

  test("a committed draft resyncs to a normalized external value", () => {
    const committed = initialDraft("0300");
    expect(syncDraft(committed, "300")).toEqual({ draft: "300", synced: "300" });
  });
});
