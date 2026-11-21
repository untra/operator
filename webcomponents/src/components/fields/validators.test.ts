import { describe, expect, test } from "bun:test";

import {
  isValid,
  maxLength,
  minLength,
  oneOf,
  pattern,
  required,
  url,
  validate,
} from "./validators";

describe("required", () => {
  test("blank string is an error", () => {
    expect(required()("   ")).toBe("Required");
  });
  test("custom message is used", () => {
    expect(required("Name it")("")).toBe("Name it");
  });
  test("non-blank string is valid", () => {
    expect(required()("x")).toBeNull();
  });
});

describe("pattern", () => {
  const lower = pattern(/^[a-z]+$/, "lowercase only");
  test("mismatch returns message", () => {
    expect(lower("Abc")).toBe("lowercase only");
  });
  test("match is valid", () => {
    expect(lower("abc")).toBeNull();
  });
  test("empty value defers to required", () => {
    expect(lower("")).toBeNull();
  });
});

describe("length rules", () => {
  test("maxLength rejects longer values", () => {
    expect(maxLength(3)("abcd")).toBe("At most 3 characters");
  });
  test("maxLength accepts equal length", () => {
    expect(maxLength(3)("abc")).toBeNull();
  });
  test("minLength rejects shorter non-empty values", () => {
    expect(minLength(3)("ab")).toBe("At least 3 characters");
  });
  test("minLength defers empty to required", () => {
    expect(minLength(3)("")).toBeNull();
  });
});

describe("url", () => {
  test("http url is valid", () => {
    expect(url()("https://example.com/x")).toBeNull();
  });
  test("non-http scheme is an error", () => {
    expect(url()("ftp://example.com")).toBe("Enter an http(s) URL");
  });
  test("garbage is an error", () => {
    expect(url()("not a url")).toBe("Enter an http(s) URL");
  });
  test("empty defers to required", () => {
    expect(url()("")).toBeNull();
  });
});

describe("oneOf", () => {
  const rule = oneOf(["a", "b"] as const);
  test("listed value is valid", () => {
    expect(rule("a")).toBeNull();
  });
  test("unlisted value is an error", () => {
    expect(rule("c")).toBe("Choose an option");
  });
});

describe("validate / isValid", () => {
  const rules = [required(), maxLength(3), pattern(/^[a-z]+$/, "lowercase only")];
  test("returns first failing message", () => {
    expect(validate("", rules)).toBe("Required");
    expect(validate("ABCD", rules)).toBe("At most 3 characters");
    expect(validate("AB", rules)).toBe("lowercase only");
  });
  test("valid value returns null", () => {
    expect(validate("ab", rules)).toBeNull();
    expect(isValid("ab", rules)).toBe(true);
  });
  test("no rules is valid", () => {
    expect(isValid("", [])).toBe(true);
  });
});
