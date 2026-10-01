// Password login. Rendered outside the app Layout: an unauthenticated visitor
// has no sections to show in the sidebar, and every API call the shell makes
// would 401.

import { useState, useCallback } from "react";
import { Link, Navigate, useNavigate } from "react-router-dom";
import { ApiError } from "../api-client";
import { useApiMutation, useApiQuery, useResetSession } from "../api";
import { bootstrapStatusQuery, loginMutation, setupStatusQuery } from "../api/definitions";
import { MAX_PASSWORD_LENGTH, MAX_USERNAME_LENGTH } from "../auth-constraints";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function LoginPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const bootstrap = useApiQuery(bootstrapStatusQuery());
  const login = useApiMutation(loginMutation);
  const setup = useApiQuery(setupStatusQuery(), { enabled: false });
  const resetSession = useResetSession();

  const submit = useCallback(
    async (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      setError(null);
      try {
        await login.mutateAsync({ username, password });
        const status = await setup.refetch();
        resetSession();
        void navigate(status.initialized ? "/" : "/onboarding", { replace: true });
      } catch (e) {
        const status = e instanceof ApiError ? e.status : 0;
        setError(
          status === 429
            ? "Too many attempts. Wait a moment and try again."
            : "Incorrect username or password.",
        );
      }
    },
    [login, username, password, navigate, resetSession, setup],
  );

  if (bootstrap.data && bootstrap.data.state !== "complete") {
    return <Navigate to="/setup" replace />;
  }

  if (bootstrap.isLoading) {
    return <AuthCard title="Sign in to Operator">Checking server status…</AuthCard>;
  }

  if (bootstrap.error) {
    return (
      <AuthCard title="Sign in to Operator" error={bootstrap.error.message}>
        The Operator server status could not be loaded. Check the server and reload this page.
      </AuthCard>
    );
  }

  return (
    <AuthCard
      title="Sign in to Operator"
      subtitle="Enter your account credentials."
      error={error}
      onSubmit={submit}
      actions={
        <AuthSubmit
          busy={login.isPending}
          busyLabel="Signing in…"
          disabled={!username || !password}
        >
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
