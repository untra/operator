import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import {
  callOperator,
  clearTokenCache,
  resolveCredential,
  tokenStillValid,
} from "./operator-client.js";

const env = {};

function requestUrl(url) {
  if (typeof url === "string") {
    return url;
  }
  if (url instanceof URL) {
    return url.href;
  }
  return url.url;
}

describe("resolveCredential", () => {
  test("prefers an explicit bearer over an access key", () => {
    const cred = resolveCredential(
      { operatorApiToken: "local-token", operatorAccessKey: "opk_should_not_be_sent" },
      { OPERATOR_API_TOKEN: "env-bearer", OPERATOR_ACCESS_KEY: "env-key" },
    );
    expect(cred).toEqual({ kind: "bearer", token: "local-token" });
  });

  test("uses OPERATOR_API_TOKEN when no param bearer is set", () => {
    const cred = resolveCredential(
      {},
      { OPERATOR_API_TOKEN: "env-bearer", OPERATOR_ACCESS_KEY: "env-key" },
    );
    expect(cred).toEqual({ kind: "bearer", token: "env-bearer" });
  });

  test("uses an access key only when no bearer is set", () => {
    const cred = resolveCredential(
      { operatorAccessKey: "param-key" },
      { OPERATOR_ACCESS_KEY: "env-key" },
    );
    expect(cred).toEqual({ kind: "access_key", token: "param-key" });
  });

  test("falls back to OPERATOR_ACCESS_KEY", () => {
    expect(resolveCredential({}, { OPERATOR_ACCESS_KEY: "env-key" })).toEqual({
      kind: "access_key",
      token: "env-key",
    });
  });

  test("treats blank values as unset", () => {
    expect(resolveCredential({ operatorApiToken: "  " }, { OPERATOR_API_TOKEN: "" })).toBeNull();
  });
});

test("refreshes inside the 30s skew and keeps a token outside it", () => {
  const now = 1_000_000;
  expect(tokenStillValid({ expiresAtMs: now + 20_000 }, now)).toBe(false);
  expect(tokenStillValid({ expiresAtMs: now + 60_000 }, now)).toBe(true);
});

describe("callOperator", () => {
  let calls;

  beforeEach(() => {
    clearTokenCache();
    calls = [];
    globalThis.fetch = (url, init) => {
      const u = requestUrl(url);
      calls.push({ url: u, init });
      if (u.endsWith("/api/v1/auth/token")) {
        const body = JSON.parse(init.body);
        if (body.access_key !== "good-key") {
          return new Response(JSON.stringify({ error: "invalid_grant" }), { status: 400 });
        }
        return new Response(
          JSON.stringify({
            access_token: "minted-access",
            token_type: "Bearer",
            expires_in: 1,
            scopes: ["read", "write", "execute"],
          }),
          { status: 200 },
        );
      }
      const auth = init.headers.Authorization;
      if (auth !== "Bearer minted-access" && auth !== "Bearer local-token") {
        return new Response(JSON.stringify({ error: "no valid credential was presented" }), {
          status: 401,
        });
      }
      return new Response(JSON.stringify({ queued: 1 }), { status: 200 });
    };
  });

  afterEach(() => {
    delete globalThis.fetch;
    clearTokenCache();
  });

  test("sends a loopback bearer and does not call the token endpoint", async () => {
    const out = await callOperator({
      params: { operatorBaseUrl: "http://127.0.0.1:7008", operatorApiToken: "local-token" },
      path: "/api/v1/queue/status",
      env,
    });
    expect(out.success).toBe(true);
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe("http://127.0.0.1:7008/api/v1/queue/status");
    expect(calls[0].init.headers.Authorization).toBe("Bearer local-token");
  });

  test("exchanges an access key, then sends the access token", async () => {
    const out = await callOperator({
      params: { operatorBaseUrl: "https://operator.example", operatorAccessKey: "good-key" },
      path: "/api/v1/queue/status",
      env,
    });
    expect(out.success).toBe(true);
    expect(calls[0].url).toBe("https://operator.example/api/v1/auth/token");
    expect(JSON.parse(calls[0].init.body)).toEqual({
      grant_type: "operator:access-key",
      access_key: "good-key",
    });
    expect(calls[0].init.headers.Authorization).toBeUndefined();
    expect(calls[1].init.headers.Authorization).toBe("Bearer minted-access");
  });

  test("exchanges again once the cached access token is inside the skew window", async () => {
    await callOperator({
      params: { operatorBaseUrl: "https://operator.example", operatorAccessKey: "good-key" },
      path: "/api/v1/queue/status",
      env,
    });
    await new Promise((r) => setTimeout(r, 1100));
    await callOperator({
      params: { operatorBaseUrl: "https://operator.example", operatorAccessKey: "good-key" },
      path: "/api/v1/queue/status",
      env,
    });
    const tokenCalls = calls.filter((c) => c.url.endsWith("/api/v1/auth/token"));
    expect(tokenCalls).toHaveLength(2);
  });

  test("does not send the access key as a bearer when exchange fails", async () => {
    const out = await callOperator({
      params: { operatorBaseUrl: "https://operator.example", operatorAccessKey: "bad-key" },
      path: "/api/v1/queue/status",
      env,
    });
    expect(out.success).toBe(false);
    expect(calls.some((c) => c.init.headers.Authorization)).toBe(false);
    expect(out.error).toContain("access key");
  });

  test("a missing credential names the env vars", async () => {
    const out = await callOperator({
      params: { operatorBaseUrl: "https://operator.example" },
      path: "/api/v1/queue/status",
      env,
    });
    expect(out.success).toBe(false);
    expect(calls).toHaveLength(0);
    expect(out.error).toContain("OPERATOR_ACCESS_KEY");
    expect(out.error).toContain("OPERATOR_API_TOKEN");
  });

  test("re-exchanges once after a 401 and does not loop", async () => {
    let queueCalls = 0;
    globalThis.fetch = (url, init) => {
      const u = requestUrl(url);
      calls.push({ url: u, init });
      if (u.endsWith("/api/v1/auth/token")) {
        const n = calls.filter((c) => c.url.endsWith("/api/v1/auth/token")).length;
        return new Response(
          JSON.stringify({
            access_token: n === 1 ? "stale" : "fresh",
            token_type: "Bearer",
            expires_in: 900,
            scopes: ["read"],
          }),
          { status: 200 },
        );
      }
      queueCalls += 1;
      if (queueCalls === 1 || init.headers.Authorization !== "Bearer fresh") {
        return new Response(JSON.stringify({ error: "no valid credential was presented" }), {
          status: 401,
        });
      }
      return new Response(JSON.stringify({ queued: 1 }), { status: 200 });
    };

    const out = await callOperator({
      params: { operatorBaseUrl: "https://operator.example", operatorAccessKey: "good-key" },
      path: "/api/v1/queue/status",
      env,
    });
    expect(out.success).toBe(true);
    expect(calls.filter((c) => c.url.endsWith("/api/v1/auth/token"))).toHaveLength(2);
    expect(calls.filter((c) => c.url.endsWith("/queue/status"))).toHaveLength(2);
  });

  test("a second 401 is a failure", async () => {
    globalThis.fetch = (url, init) => {
      const u = requestUrl(url);
      calls.push({ url: u, init });
      if (u.endsWith("/api/v1/auth/token")) {
        return new Response(
          JSON.stringify({
            access_token: "still-bad",
            token_type: "Bearer",
            expires_in: 900,
            scopes: ["read"],
          }),
          { status: 200 },
        );
      }
      return new Response(JSON.stringify({ error: "no valid credential was presented" }), {
        status: 401,
      });
    };

    const out = await callOperator({
      params: { operatorBaseUrl: "https://operator.example", operatorAccessKey: "good-key" },
      path: "/api/v1/queue/status",
      env,
    });
    expect(out.success).toBe(false);
    expect(calls.filter((c) => c.url.endsWith("/api/v1/auth/token"))).toHaveLength(2);
    expect(calls.filter((c) => c.url.endsWith("/queue/status"))).toHaveLength(2);
  });

  test("a bearer 401 is not retried", async () => {
    globalThis.fetch = (url, init) => {
      calls.push({ url: requestUrl(url), init });
      return new Response(JSON.stringify({ error: "no valid credential was presented" }), {
        status: 401,
      });
    };
    const out = await callOperator({
      params: { operatorBaseUrl: "http://127.0.0.1:7008", operatorApiToken: "local-token" },
      path: "/api/v1/queue/status",
      env,
    });
    expect(out.success).toBe(false);
    expect(calls).toHaveLength(1);
  });

  test("a certificate failure stays a failure and names NODE_EXTRA_CA_CERTS", async () => {
    globalThis.fetch = () => {
      throw new Error("fetch failed: self-signed certificate");
    };
    const out = await callOperator({
      params: { operatorBaseUrl: "https://operator.example", operatorApiToken: "local-token" },
      path: "/api/v1/queue/status",
      env,
    });
    expect(out.success).toBe(false);
    expect(out.error).toContain("NODE_EXTRA_CA_CERTS");
    expect(out.error).toContain("self-signed certificate");
  });
});
