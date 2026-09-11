// Password login. Rendered outside the app Layout: an unauthenticated visitor
// has no sections to show in the sidebar, and every API call the shell makes
// would 401.

import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useHost } from '../host';
import { OperatorApi, ApiError } from '../api-client';
import styles from './AuthPage.module.css';

export function LoginPage() {
  const host = useHost();
  const navigate = useNavigate();
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // A server with no admin account yet needs setup, not login.
  useEffect(() => {
    const api = new OperatorApi(host);
    api
      .bootstrapStatus()
      .then((status) => {
        if (status.state !== 'complete') {void navigate('/setup', { replace: true });}
        return undefined;
      })
      .catch(() => {
        /* Unreachable server: let the login attempt report it. */
      });
  }, [host, navigate]);

  async function submit(event: React.SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await new OperatorApi(host).login(password);
      void navigate('/', { replace: true });
    } catch (e) {
      // 429 carries a wait, not a wrong password; saying "incorrect" would
      // send the operator hunting for a password problem they do not have.
      const status = e instanceof ApiError ? e.status : 0;
      setError(
        status === 429
          ? 'Too many attempts. Wait a moment and try again.'
          : 'Incorrect password.',
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className={styles.screen}>
      <form className={styles.card} onSubmit={submit}>
        <h1 className={styles.title}>Sign in to Operator</h1>
        <p className={styles.subtitle}>This workspace requires the admin password.</p>

        {error && <p className={styles.error}>{error}</p>}

        <label className={styles.field}>
          <span className={styles.label}>Password</span>
          <input
            className={styles.input}
            type="password"
            autoComplete="current-password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
          />
        </label>

        <button className={styles.button} type="submit" disabled={busy || !password}>
          {busy ? 'Signing in…' : 'Sign in'}
        </button>
      </form>
    </div>
  );
}
