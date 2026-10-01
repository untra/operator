import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { ApiError, OperatorApi, getCsrfToken, setCsrfToken } from "./api-client";
import type { Host } from "./host";

const ORIGIN = "http://operator.test";
const CSRF_PATH = "/api/v1/auth/csrf";
const CSRF_HEADER = "x-operator-csrf";
const CSRF_FAILED = "csrf_failed";
const FORBIDDEN = 403;

type Call = { url: string; method: string; csrf: string | null };

const host: Host = {
  baseUrl: () => ORIGIN,
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};

const realFetch = globalThis.fetch;
let calls: Call[] = [];

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function stubFetch(respond: (call: Call) => Response | Promise<Response>): void {
  const stub = async (input: RequestInfo | URL, init?: RequestInit) => {
    const call: Call = {
      url: input instanceof Request ? input.url : input.toString(),
      method: (init?.method ?? "GET").toUpperCase(),
      csrf: new Headers(init?.headers).get(CSRF_HEADER),
    };
    calls.push(call);
    return respond(call);
  };
  globalThis.fetch = Object.assign(stub, { preconnect: realFetch.preconnect });
}

function csrfFetches(): Call[] {
  return calls.filter((c) => c.url === `${ORIGIN}${CSRF_PATH}`);
}

function mutations(): Call[] {
  return calls.filter((c) => c.method !== "GET");
}

beforeEach(() => {
  calls = [];
  setCsrfToken(null);
});

afterEach(() => {
  globalThis.fetch = realFetch;
  setCsrfToken(null);
});

describe("CSRF on mutations", () => {
  test("a mutation with no token fetches one and sends it", async () => {
    stubFetch((call) =>
      call.method === "GET" ? json({ csrf_token: "fresh" }) : json({ status: "removed" }),
    );

    await new OperatorApi(host).removeLicense();

    expect(csrfFetches()).toHaveLength(1);
    expect(mutations()).toHaveLength(1);
    expect(mutations()[0].csrf).toBe("fresh");
    expect(getCsrfToken()).toBe("fresh");
  });

  test("concurrent mutations share a single refresh", async () => {
    let release!: () => void;
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    stubFetch(async (call) => {
      if (call.method === "GET") {
        await gate;
        return json({ csrf_token: "shared" });
      }
      return json({ status: "ok" });
    });

    const api = new OperatorApi(host);
    const pending = Promise.all([api.removeLicense(), api.removeLicense()]);
    release();
    await pending;

    expect(csrfFetches()).toHaveLength(1);
    expect(mutations().map((c) => c.csrf)).toEqual(["shared", "shared"]);
  });

  test("a rejected token is refreshed and retried once, then the error surfaces", async () => {
    setCsrfToken("stale");
    let issued = 0;
    stubFetch((call) => {
      if (call.method === "GET") {
        issued += 1;
        return json({ csrf_token: `fresh-${issued}` });
      }
      return json({ error: CSRF_FAILED, message: "CSRF token is invalid" }, FORBIDDEN);
    });

    const error = await new OperatorApi(host).removeLicense().catch((e: unknown) => e);

    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(FORBIDDEN);
    expect((error as ApiError).code).toBe(CSRF_FAILED);
    expect(csrfFetches()).toHaveLength(1);
    expect(mutations().map((c) => c.csrf)).toEqual(["stale", "fresh-1"]);
  });

  test("a failed refresh is not cached, so the next mutation tries again", async () => {
    let attempt = 0;
    stubFetch((call) => {
      if (call.method === "GET") {
        attempt += 1;
        return attempt === 1
          ? json({ error: "internal_error", message: "boom" }, 500)
          : json({ csrf_token: "second" });
      }
      return json({ status: "ok" });
    });

    const api = new OperatorApi(host);
    await expect(api.removeLicense()).rejects.toBeInstanceOf(ApiError);
    await api.removeLicense();

    expect(csrfFetches()).toHaveLength(2);
    expect(mutations().map((c) => c.csrf)).toEqual(["second"]);
  });

  test("sessionless mutations such as login do not fetch a token", async () => {
    stubFetch(() => json({ csrf_token: "issued", scopes: [], expires_at: "" }));

    await new OperatorApi(host).login("admin", "password");

    expect(csrfFetches()).toHaveLength(0);
    expect(getCsrfToken()).toBe("issued");
  });
});

describe("ApiError", () => {
  test("carries the server's machine-readable error as its code", async () => {
    stubFetch(() => json({ error: "not_found", message: "no such target" }, 404));

    const error = await new OperatorApi(host).targets().catch((e: unknown) => e);

    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).code).toBe("not_found");
    expect((error as ApiError).message).toBe("no such target");
  });
});
