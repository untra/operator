// First-run admin setup. Reachable only while the server has no usable admin
// account; once bootstrap completes this redirects to login, so it cannot be
// used to re-claim the account.

import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useHost } from '../host';
import { OperatorApi, ApiError } from '../api-client';
import { MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH } from '../auth-constraints';
import styles from './AuthPage.module.css';

export function SetupPage() {
  const host = useHost();
  const navigate = useNavigate();
  const [needsTemporary, setNeedsTemporary] = useState(false);
  const [temporary, setTemporary] = useState('');
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const api = new OperatorApi(host);
    api
      .bootstrapStatus()
      .then((status) => {
        if (status.state === 'complete') {
          void navigate('/login', { replace: true });
        } else {
          setNeedsTemporary(status.requires_temporary_password);
        }
        return undefined;
      })
      .catch(() => setError('Cannot reach the Operator server.'));
  }, [host, navigate]);

  const tooShort = password.length > 0 && password.length < MIN_PASSWORD_LENGTH;
  const mismatch = confirm.length > 0 && password !== confirm;
  const ready =
    password.length >= MIN_PASSWORD_LENGTH &&
    password === confirm &&
    (!needsTemporary || temporary.length > 0);

  async function submit(event: React.SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const api = new OperatorApi(host);
      const result = await api.bootstrap({
        temporary_password: needsTemporary ? temporary : null,
        new_password: password,
      });
      // Bootstrap creates the account but does not sign you in.
      await api.login(result.username, password);
      void navigate('/onboarding', { replace: true });
    } catch (e) {
      if (e instanceof ApiError && e.status === 409) {
        setError('This server already has an admin account. Sign in instead.');
      } else if (e instanceof ApiError && e.status === 401) {
        setError('The temporary password is incorrect.');
      } else if (e instanceof ApiError && e.status === 429) {
        setError('Too many attempts. Wait a moment and try again.');
      } else {
        setError(e instanceof ApiError ? e.message : 'Setup failed.');
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className={styles.screen}>
      <form className={styles.card} onSubmit={submit}>
        <h1 className={styles.title}>Set up Operator</h1>
        <p className={styles.subtitle}>
          Choose the admin password for this workspace. Operator has a single
          human account.
        </p>

        {error && <p className={styles.error}>{error}</p>}

        {needsTemporary && (
          <>
            <p className={styles.notice}>
              This server was started with a bootstrap secret. Enter it to claim
              the admin account.
            </p>
            <label className={styles.field}>
              <span className={styles.label}>Temporary password</span>
              <input
                className={styles.input}
                type="password"
                autoComplete="one-time-code"
                maxLength={MAX_PASSWORD_LENGTH}
                value={temporary}
                onChange={(e) => setTemporary(e.target.value)}
                required
              />
            </label>
          </>
        )}

        <label className={styles.field}>
          <span className={styles.label}>New admin password</span>
          <input
            className={styles.input}
            type="password"
            autoComplete="new-password"
            maxLength={MAX_PASSWORD_LENGTH}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
          />
          <span className={styles.hint}>
            {tooShort
              ? `At least ${MIN_PASSWORD_LENGTH} characters.`
              : `A passphrase of ${MIN_PASSWORD_LENGTH} characters or more.`}
          </span>
        </label>

        <label className={styles.field}>
          <span className={styles.label}>Confirm password</span>
          <input
            className={styles.input}
            type="password"
            autoComplete="new-password"
            maxLength={MAX_PASSWORD_LENGTH}
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            required
          />
          {mismatch && <span className={styles.hint}>Passwords do not match.</span>}
        </label>

        <button className={styles.button} type="submit" disabled={busy || !ready}>
          {busy ? 'Creating…' : 'Create admin account'}
        </button>
      </form>
    </div>
  );
}
