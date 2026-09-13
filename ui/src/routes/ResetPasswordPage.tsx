import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ApiError, OperatorApi } from "../api-client";
import { useHost } from "../host";
import { MAX_PASSWORD_LENGTH, MAX_USERNAME_LENGTH, MIN_PASSWORD_LENGTH } from "../auth-constraints";
import styles from "./AuthPage.module.css";

export function ResetPasswordPage() {
  const host = useHost();
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const ready =
    username.length > 0 &&
    currentPassword.length > 0 &&
    newPassword.length >= MIN_PASSWORD_LENGTH &&
    newPassword === confirmPassword;

  async function submit(event: React.SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const api = new OperatorApi(host);
      await api.resetPassword({
        username,
        current_password: currentPassword,
        new_password: newPassword,
      });
      await api.login(username, newPassword);
      void navigate("/", { replace: true });
    } catch (caught) {
      const status = caught instanceof ApiError ? caught.status : 0;
      setError(
        status === 429
          ? "Too many attempts. Wait a moment and try again."
          : status === 400
            ? caught instanceof ApiError
              ? caught.message
              : "The new password is invalid."
            : "Incorrect username or current password.",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className={styles.screen}>
      <form className={styles.card} onSubmit={submit}>
        <h1 className={styles.title}>Change password</h1>
        <p className={styles.subtitle}>
          Changing the password revokes all sessions and access keys.
        </p>
        {error && <p className={styles.error}>{error}</p>}
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
        <label className={styles.field}>
          <span className={styles.label}>Current password</span>
          <input
            className={styles.input}
            type="password"
            maxLength={MAX_PASSWORD_LENGTH}
            value={currentPassword}
            onChange={(event) => setCurrentPassword(event.target.value)}
            required
          />
        </label>
        <label className={styles.field}>
          <span className={styles.label}>New password</span>
          <input
            className={styles.input}
            type="password"
            maxLength={MAX_PASSWORD_LENGTH}
            value={newPassword}
            onChange={(event) => setNewPassword(event.target.value)}
            required
          />
          <span className={styles.hint}>At least {MIN_PASSWORD_LENGTH} characters.</span>
        </label>
        <label className={styles.field}>
          <span className={styles.label}>Confirm new password</span>
          <input
            className={styles.input}
            type="password"
            maxLength={MAX_PASSWORD_LENGTH}
            value={confirmPassword}
            onChange={(event) => setConfirmPassword(event.target.value)}
            required
          />
        </label>
        <button className={styles.button} type="submit" disabled={busy || !ready}>
          {busy ? "Changing…" : "Change password"}
        </button>
        <div className={styles.links}>
          <Link to="/login">Back to sign in</Link>
        </div>
      </form>
    </div>
  );
}
