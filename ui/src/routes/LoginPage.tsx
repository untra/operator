import { useState, useCallback } from "react";
import { Link, Navigate, useNavigate, useSearchParams } from "react-router-dom";
import { useHost } from "../host";
import { ApiError, deviceApprovalQuery } from "../api-client";
import { useApiMutation, useApiQuery, useResetSession } from "../api";
import {
  bootstrapStatusQuery,
  loginMutation,
  setupStatusQuery,
  oidcProvidersQuery,
} from "../api/definitions";
import { MAX_PASSWORD_LENGTH, MAX_USERNAME_LENGTH } from "../auth-constraints";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function LoginPage() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const deviceQuery = deviceApprovalQuery(params);
  const host = useHost();
  const providers = useApiQuery(oidcProvidersQuery());
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
        if (deviceQuery) {
          resetSession();
          void navigate(`/device${deviceQuery}`, { replace: true });
          return;
        }
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
    [login, username, password, navigate, resetSession, setup, deviceQuery],
  );

  if (bootstrap.data && bootstrap.data.state !== "complete") {
    return <Navigate to={`/setup${deviceQuery}`} replace />;
  }

  if (bootstrap.isLoading) {
    return <AuthCard title="Sign in to Operator">Checking server status...</AuthCard>;
  }

  if (bootstrap.error) {
    return (
      <AuthCard title="Sign in to Operator" error={bootstrap.error.message}>
        The Operator server status could not be loaded. Check the server and reload this page.
      </AuthCard>
    );
  }

  if (bootstrap.data?.oidc_only) {
    return (
      <AuthCard title="Sign in to Operator" error={providers.error?.message}>
        {providers.isLoading
          ? "Loading sign-in providers..."
          : providers.data?.length
            ? providers.data.map((provider) => (
                <a
                  key={provider.id}
                  href={`${host.baseUrl()}/api/v1/auth/oidc/login/${encodeURIComponent(
                    provider.id,
                  )}${deviceQuery}`}
                >
                  Sign in with {provider.provider}
                </a>
              ))
            : "Contact your administrator to configure a sign-in provider."}
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
