import { describe, expect, test } from "bun:test";
import { titleWithCount } from "./attention-title";

describe("titleWithCount", () => {
  test("prefixes a count", () => {
    expect(titleWithCount("Operator", 2)).toBe("(2) Operator");
  });

  test("replaces an existing count", () => {
    expect(titleWithCount("(2) Operator", 3)).toBe("(3) Operator");
  });

  test("clears the count at zero", () => {
    expect(titleWithCount("(2) Operator", 0)).toBe("Operator");
  });
});
