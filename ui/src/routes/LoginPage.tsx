// Password login. Rendered outside the app Layout: an unauthenticated visitor
// has no sections to show in the sidebar, and every API call the shell makes
// would 401.

import { useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useHost } from '../host';
import { OperatorApi, ApiError } from '../api-client';
import { MAX_PASSWORD_LENGTH, MAX_USERNAME_LENGTH } from '../auth-constraints';
import styles from './AuthPage.module.css';

export function LoginPage() {
  const host = useHost();
  const navigate = useNavigate();
  const [username, setUsername] = useState('');
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
      const api = new OperatorApi(host);
      await api.login(username, password);
      const setup = await api.setupStatus();
      void navigate(setup.initialized ? '/' : '/onboarding', { replace: true });
    } catch (e) {
      // 429 carries a wait, not a wrong password; saying "incorrect" would
      // send the operator hunting for a password problem they do not have.
      const status = e instanceof ApiError ? e.status : 0;
      setError(
        status === 429
          ? 'Too many attempts. Wait a moment and try again.'
          : 'Incorrect username or password.',
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className={styles.screen}>
      <form className={styles.card} onSubmit={submit}>
        <h1 className={styles.title}>Sign in to Operator</h1>
        <p className={styles.subtitle}>Enter your account credentials.</p>

        {error && <p className={styles.error}>{error}</p>}

        <label className={styles.field}>
          <span className={styles.label}>Username</span>
          <input
            className={styles.input}
            type="text"
            autoComplete="username"
            maxLength={MAX_USERNAME_LENGTH}
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
          />
        </label>

        <label className={styles.field}>
          <span className={styles.label}>Password</span>
          <input
            className={styles.input}
            type="password"
            autoComplete="current-password"
            maxLength={MAX_PASSWORD_LENGTH}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
          />
        </label>

        <button className={styles.button} type="submit" disabled={busy || !username || !password}>
          {busy ? 'Signing in…' : 'Sign in'}
        </button>
        <div className={styles.links}>
          <Link to="/forgot-password">Forgot password?</Link>
          <Link to="/reset-password">Change password</Link>
        </div>
      </form>
    </div>
  );
}
