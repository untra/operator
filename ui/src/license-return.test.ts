import { describe, expect, test } from "bun:test";
import { licenseReturnHash, withReturnTo } from "./license-return";

const HERE = { origin: "http://localhost:7008", pathname: "/" };

describe("withReturnTo", () => {
  test("appends this UI's URL and keeps the existing query", () => {
    const url = new URL(
      withReturnTo("https://app.operator.untra.casa/checkout?sku=premium&profile_id=abc", HERE)!,
    );
    expect(url.searchParams.get("sku")).toBe("premium");
    expect(url.searchParams.get("profile_id")).toBe("abc");
    expect(url.searchParams.get("return_to")).toBe("http://localhost:7008/");
  });

  test("keeps a path prefix the UI is served under", () => {
    const url = new URL(
      withReturnTo("https://app.example/checkout", {
        origin: "https://ops.example",
        pathname: "/operator/",
      })!,
    );
    expect(url.searchParams.get("return_to")).toBe("https://ops.example/operator/");
  });

  test("never carries the router fragment", () => {
    const url = new URL(withReturnTo("https://app.example/checkout", HERE)!);
    expect(url.searchParams.get("return_to")).not.toContain("#");
  });

  test("passes over a missing or malformed purchase URL", () => {
    expect(withReturnTo(null, HERE)).toBeNull();
    expect(withReturnTo(undefined, HERE)).toBeNull();
    expect(withReturnTo("not a url", HERE)).toBeNull();
  });
});

describe("licenseReturnHash", () => {
  test("routes a checkout return to the license return page", () => {
    expect(licenseReturnHash("#license_key=abc.def-ghi")).toBe(
      "#/license/return?license_key=abc.def-ghi",
    );
  });

  test("decodes and re-encodes the key", () => {
    const hash = licenseReturnHash("#license_key=KEY%2B%2F%3D.x")!;
    const query = new URLSearchParams(hash.split("?")[1]);
    expect(query.get("license_key")).toBe("KEY+/=.x");
  });

  test.each([[""], ["#/"], ["#/settings/license"], ["#license_key="], ["#other=1"]])(
    "leaves %p alone",
    (hash) => {
      expect(licenseReturnHash(hash)).toBeNull();
    },
  );
});
