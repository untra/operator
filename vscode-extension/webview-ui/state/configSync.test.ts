import { describe, expect, test } from "bun:test";
import { configErrorFrom, errorAfterSnapshot, isStaleSnapshot, toWireValue } from "./configSync";

describe("isStaleSnapshot", () => {
  test("a snapshot older than the last sent rev is stale", () => {
    expect(isStaleSnapshot(2, 3)).toBe(true);
  });

  test("a snapshot at or after the last sent rev is current", () => {
    expect(isStaleSnapshot(3, 3)).toBe(false);
    expect(isStaleSnapshot(4, 3)).toBe(false);
  });

  test("a snapshot without a rev is never stale", () => {
    expect(isStaleSnapshot(undefined, 3)).toBe(false);
  });
});

describe("toWireValue", () => {
  test("bigint becomes a JSON-safe number", () => {
    const wire = toWireValue(BigInt(300));
    expect(wire).toBe(300);
    expect(() => JSON.stringify({ wire })).not.toThrow();
  });

  test("other values pass through unchanged", () => {
    const obj = { a: 1 };
    expect(toWireValue(obj)).toBe(obj);
    expect(toWireValue("x")).toBe("x");
  });
});

describe("config error lifecycle", () => {
  const writeError = configErrorFrom("disk full", 4);
  const loadError = configErrorFrom("parse failed", undefined);

  test("an error with a rev is a write error, without one a load error", () => {
    expect(writeError).toEqual({ message: "disk full", source: "write" });
    expect(loadError).toEqual({ message: "parse failed", source: "load" });
  });

  test("the rollback configLoaded keeps the write error visible", () => {
    expect(errorAfterSnapshot(writeError, "configLoaded")).toBe(writeError);
  });

  test("a successful configLoaded clears a load error", () => {
    expect(errorAfterSnapshot(loadError, "configLoaded")).toBeNull();
  });

  test("a successful write clears any error", () => {
    expect(errorAfterSnapshot(writeError, "configUpdated")).toBeNull();
    expect(errorAfterSnapshot(loadError, "configUpdated")).toBeNull();
  });

  test("no error stays no error", () => {
    expect(errorAfterSnapshot(null, "configLoaded")).toBeNull();
  });
});
