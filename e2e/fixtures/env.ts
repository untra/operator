// localhost: the session cookie is `__Host-` + Secure, which Chromium only accepts over plain HTTP on localhost.
export const BASE_URL = process.env.OPERATOR_E2E_BASE_URL || "http://localhost:7008";

/** Out-of-band password the server was started with; empty when none was mounted. */
export const BOOTSTRAP_PASSWORD = process.env.OPERATOR_E2E_BOOTSTRAP_PASSWORD ?? "";

export const ISSUER_URL = process.env.OPERATOR_E2E_ISSUER_URL ?? "";
export const LICENSE_TOKEN = process.env.OPERATOR_E2E_LICENSE_TOKEN ?? "";

export const ADMIN_STATE = ".auth/admin.json";
