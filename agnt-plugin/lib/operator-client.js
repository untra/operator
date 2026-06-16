// Shared HTTP helper for the operator-plugin tools.
//
// Zero-dependency: uses the global `fetch` (Node >= 18). Every tool returns the
// AGNT contract `{ success, result, error }`.

const DEFAULT_BASE_URL = "http://localhost:7008";

/**
 * Resolve the Operator REST base URL from params, env, or the default.
 */
export function resolveBaseUrl(params) {
  const fromParam = params && params.operatorBaseUrl;
  const fromEnv =
    typeof process !== "undefined" && process.env
      ? process.env.OPERATOR_BASE_URL
      : undefined;
  return (fromParam || fromEnv || DEFAULT_BASE_URL).replace(/\/+$/, "");
}

/**
 * Call the Operator REST API and normalize the response into the AGNT
 * `{ success, result, error }` contract.
 *
 * @param {object} opts
 * @param {object} opts.params  tool params (used to resolve the base URL)
 * @param {string} opts.path    request path, e.g. "/api/v1/queue/status"
 * @param {string} [opts.method=GET]
 * @param {object} [opts.body]  JSON body for POST/PUT
 */
export async function callOperator({ params, path, method = "GET", body }) {
  const baseUrl = resolveBaseUrl(params);
  const url = `${baseUrl}${path}`;
  try {
    const init = { method, headers: { Accept: "application/json" } };
    if (body !== undefined) {
      init.headers["Content-Type"] = "application/json";
      init.body = JSON.stringify(body);
    }
    const res = await fetch(url, init);
    const text = await res.text();
    let parsed;
    try {
      parsed = text ? JSON.parse(text) : null;
    } catch {
      parsed = text;
    }
    if (!res.ok) {
      const detail =
        parsed && parsed.error ? parsed.error : `HTTP ${res.status}`;
      return { success: false, result: parsed, error: `${method} ${url} failed: ${detail}` };
    }
    return { success: true, result: parsed, error: null };
  } catch (e) {
    return { success: false, result: null, error: `${method} ${url} failed: ${e.message}` };
  }
}
