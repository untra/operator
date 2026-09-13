import { useState } from "react";
import { Link } from "react-router-dom";
import { useHost } from "../host";
import { OperatorApi } from "../api-client";
import { MAX_USERNAME_LENGTH } from "../auth-constraints";
import styles from "./AuthPage.module.css";

export function ForgotPasswordPage() {
  const host = useHost();
  const [username, setUsername] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function submit(event: React.SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    try {
      const response = await new OperatorApi(host).forgotPassword(username);
      setMessage(response.message);
    } catch {
      setMessage(
        "Recovery instructions are unavailable. Ask the server administrator to run `operator auth reset-admin-password` locally.",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className={styles.screen}>
      <form className={styles.card} onSubmit={submit}>
        <h1 className={styles.title}>Forgot password</h1>
        <p className={styles.subtitle}>
          Operator recovery is performed by the server administrator.
        </p>
        {message && <p className={styles.notice}>{message}</p>}
        <label className={styles.field}>
          <span className={styles.label}>Username</span>
          <input
            className={styles.input}
            autoComplete="username"
            maxLength={MAX_USERNAME_LENGTH}
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            required
          />
        </label>
        <button className={styles.button} type="submit" disabled={busy || !username}>
          {busy ? "Checking…" : "Get recovery instructions"}
        </button>
        <div className={styles.links}>
          <Link to="/login">Back to sign in</Link>
        </div>
      </form>
    </div>
  );
}
