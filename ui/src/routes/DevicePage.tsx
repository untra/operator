// Device-approval screen for the OAuth device flow.
//
// The human lands here from the URL an IDE client printed, sees which client is
// asking, and approves. It renders inside the authenticated Layout on purpose:
// approving a device grants a credential, so it requires an admin session.

import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useHost } from "../host";
import { OperatorApi, ApiError } from "../api-client";
import styles from "./AuthPage.module.css";

export function DevicePage() {
  const host = useHost();
  const [params] = useSearchParams();
  const [userCode, setUserCode] = useState(params.get("user_code") ?? "");
  const [approved, setApproved] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // A CSRF token is required to approve, and a page load (or a fresh tab
  // opened by the IDE) has none in memory yet.
  useEffect(() => {
    new OperatorApi(host).refreshCsrf().catch(() => {
      /* An unauthenticated visitor is redirected to login by the request layer. */
    });
  }, [host]);

  async function submit(event: React.SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const res = await new OperatorApi(host).approveDevice(userCode.trim());
      setApproved(res.client_id);
    } catch (e) {
      setError(
        e instanceof ApiError && e.status === 404
          ? "That code is unknown or has expired. Start the connection again from your editor."
          : "Approval failed.",
      );
    } finally {
      setBusy(false);
    }
  }

  if (approved) {
    return (
      <div className={styles.screen}>
        <div className={styles.card}>
          <h1 className={styles.title}>Device approved</h1>
          <p className={styles.subtitle}>
            <strong>{approved}</strong> now has access. You can close this page and return to your
            editor.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.screen}>
      <form className={styles.card} onSubmit={submit}>
        <h1 className={styles.title}>Approve a device</h1>
        <p className={styles.subtitle}>
          Enter the code shown in the application requesting access.
        </p>

        {error && <p className={styles.error}>{error}</p>}

        <label className={styles.field}>
          <span className={styles.label}>Code</span>
          <input
            className={styles.input}
            value={userCode}
            onChange={(e) => setUserCode(e.target.value.toUpperCase())}
            placeholder="XXXX-XXXX"
            required
          />
        </label>

        <button className={styles.button} type="submit" disabled={busy || !userCode}>
          {busy ? "Approving…" : "Approve"}
        </button>
      </form>
    </div>
  );
}
