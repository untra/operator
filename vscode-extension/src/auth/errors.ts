/**
 * Errors raised by the authenticated request path.
 */

export const SIGN_IN_COMMAND_TITLE = 'Operator: Sign In';

/** A non-2xx response from the Operator daemon, carrying the HTTP status. */
export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
  }
}

/**
 * The daemon rejected every credential the extension could present.
 * Sign-in is deliberately not started automatically; the message names the
 * command so the user chooses when to go through the browser.
 */
export class AuthRequiredError extends ApiError {
  constructor(apiUrl: string) {
    super(401, `Not signed in to Operator at ${apiUrl}. Run "${SIGN_IN_COMMAND_TITLE}".`);
    this.name = 'AuthRequiredError';
  }
}
