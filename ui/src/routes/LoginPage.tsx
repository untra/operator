// Password login. Rendered outside the app Layout: an unauthenticated visitor
// has no sections to show in the sidebar, and every API call the shell makes
// would 401.

import { useEffect, useState, useCallback } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useHost } from "../host";
import { OperatorApi, ApiError } from "../api-client";
import { MAX_PASSWORD_LENGTH, MAX_USERNAME_LENGTH } from "../auth-constraints";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function LoginPage() {
  const host = useHost();
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // A server with no admin account yet needs setup, not login.
  useEffect(() => {
    const api = new OperatorApi(host);
    api
      .bootstrapStatus()
      .then((status) => {
        if (status.state !== "complete") {
          void navigate("/setup", { replace: true });
        }
        return undefined;
      })
      .catch(() => {
        /* Unreachable server: let the login attempt report it. */
      });
  }, [host, navigate]);

  const submit = useCallback(
    async (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      setError(null);
      setBusy(true);
      try {
        const api = new OperatorApi(host);
        await api.login(username, password);
        const setup = await api.setupStatus();
        void navigate(setup.initialized ? "/" : "/onboarding", { replace: true });
      } catch (e) {
        const status = e instanceof ApiError ? e.status : 0;
        setError(
          status === 429
            ? "Too many attempts. Wait a moment and try again."
            : "Incorrect username or password.",
        );
      } finally {
        setBusy(false);
      }
    },
    [host, username, password, navigate],
  );

  return (
    <AuthCard
      title="Sign in to Operator"
      subtitle="Enter your account credentials."
      error={error}
      onSubmit={submit}
      actions={
        <AuthSubmit busy={busy} busyLabel="Signing in…" disabled={!username || !password}>
          Sign in
        </AuthSubmit>
      }
      links={
        <>
          <Link to="/forgot-password">Forgot password?</Link>
          <Link to="/reset-password">Change password</Link>
        </>
      }
    >
      <AuthField
        label="Username"
        type="text"
        autoComplete="username"
        maxLength={MAX_USERNAME_LENGTH}
        value={username}
        onChange={setUsername}
        required
      />
      <AuthField
        label="Password"
        type="password"
        autoComplete="current-password"
        maxLength={MAX_PASSWORD_LENGTH}
        value={password}
        onChange={setPassword}
        required
      />
    </AuthCard>
  );
}
