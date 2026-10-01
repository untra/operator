// Shared HTTP helper for the operator-plugin tools.
//
// Zero-dependency: uses the global `fetch` (Node >= 18). Every tool returns the
// AGNT contract `{ success, result, error }`.

const DEFAULT_BASE_URL = "http://localhost:7008";
const REFRESH_SKEW_MS = 30_000;
const tokenCache = new Map();

export function resolveBaseUrl(params, env = process.env) {
  const fromParam = params?.operatorBaseUrl;
  const fromEnv = env?.OPERATOR_BASE_URL;
  return String(fromParam || fromEnv || DEFAULT_BASE_URL).replace(/\/+$/, "");
}

function nonempty(value) {
  if (typeof value !== "string") {
    return undefined;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

// First hit wins: an explicit bearer, an env bearer, a param access key, an env access key.
export function resolveCredential(params, env = process.env) {
  const bearer = nonempty(params?.operatorApiToken) || nonempty(env?.OPERATOR_API_TOKEN);
  if (bearer) {
    return { kind: "bearer", token: bearer };
  }
  const accessKey = nonempty(params?.operatorAccessKey) || nonempty(env?.OPERATOR_ACCESS_KEY);
  if (accessKey) {
    return { kind: "access_key", token: accessKey };
  }
  return null;
}

export function tokenStillValid(entry, nowMs) {
  return Boolean(entry) && entry.expiresAtMs - nowMs > REFRESH_SKEW_MS;
}

export function clearTokenCache() {
  tokenCache.clear();
}

function cacheKey(baseUrl, accessKey) {
  return `${baseUrl}\n${accessKey}`;
}

async function readJson(res) {
  const text = await res.text();
  try {
    return text ? JSON.parse(text) : null;
  } catch {
    return text;
  }
}

async function exchangeAccessKey(baseUrl, accessKey) {
  const key = cacheKey(baseUrl, accessKey);
  const cached = tokenCache.get(key);
  if (tokenStillValid(cached, Date.now())) {
    return cached.accessToken;
  }

  const res = await fetch(`${baseUrl}/api/v1/auth/token`, {
    method: "POST",
    headers: { Accept: "application/json", "Content-Type": "application/json" },
    body: JSON.stringify({ grant_type: "operator:access-key", access_key: accessKey }),
  });
  const parsed = await readJson(res);
  if (!res.ok || !parsed?.access_token) {
    tokenCache.delete(key);
    const detail =
      parsed && typeof parsed === "object" && parsed.error ? parsed.error : `HTTP ${res.status}`;
    throw new Error(`access key exchange failed: ${detail}`);
  }
  const expiresInMs = Number(parsed.expires_in) * 1000;
  tokenCache.set(key, {
    accessToken: parsed.access_token,
    expiresAtMs: Date.now() + (Number.isFinite(expiresInMs) ? expiresInMs : 0),
  });
  return parsed.access_token;
}

async function send(url, method, body, bearer) {
  const init = {
    method,
    headers: { Accept: "application/json", Authorization: `Bearer ${bearer}` },
  };
  if (body !== undefined) {
    init.headers["Content-Type"] = "application/json";
    init.body = JSON.stringify(body);
  }
  const res = await fetch(url, init);
  const parsed = await readJson(res);
  return { ok: res.ok, status: res.status, parsed };
}

function failure(method, url, parsed, status) {
  const detail =
    parsed && typeof parsed === "object" && parsed.error ? parsed.error : `HTTP ${status}`;
  return { success: false, result: parsed, error: `${method} ${url} failed: ${detail}` };
}

/**
 * Call the Operator REST API and normalize the response into the AGNT
 * `{ success, result, error }` contract.
 *
 * @param {object} opts
 * @param {object} opts.params  tool params (base URL and optional credential overrides)
 * @param {string} opts.path    request path, e.g. "/api/v1/queue/status"
 * @param {string} [opts.method=GET]
 * @param {object} [opts.body]  JSON body for POST/PUT
 * @param {object} [opts.env]   environment used to resolve the credential
 */
export async function callOperator({ params, path, method = "GET", body, env = process.env }) {
  const baseUrl = resolveBaseUrl(params, env);
  const credential = resolveCredential(params, env);
  if (!credential) {
    return {
      success: false,
      result: null,
      error:
        "Operator requires a credential. Set OPERATOR_ACCESS_KEY (service access key) or OPERATOR_API_TOKEN (bearer, including the loopback local token).",
    };
  }

  const url = `${baseUrl}${path}`;
  try {
    let bearer =
      credential.kind === "bearer"
        ? credential.token
        : await exchangeAccessKey(baseUrl, credential.token);
    let result = await send(url, method, body, bearer);
    if (!result.ok && result.status === 401 && credential.kind === "access_key") {
      tokenCache.delete(cacheKey(baseUrl, credential.token));
      bearer = await exchangeAccessKey(baseUrl, credential.token);
      result = await send(url, method, body, bearer);
    }
    if (!result.ok) {
      return failure(method, url, result.parsed, result.status);
    }
    return { success: true, result: result.parsed, error: null };
  } catch (e) {
    const hint = /CERT|UNABLE_TO_VERIFY|self.signed/i.test(e.message)
      ? " The certificate was not trusted. Point NODE_EXTRA_CA_CERTS at the CA file. TLS verification stays on."
      : "";
    return { success: false, result: null, error: `${method} ${url} failed: ${e.message}${hint}` };
  }
}
