// First-run admin setup. Reachable only while the server has no usable admin
// account; once bootstrap completes this redirects to login, so it cannot be
// used to re-claim the account.

import { useState, useCallback } from "react";
import { Navigate, useNavigate } from "react-router-dom";
import { ApiError } from "../api-client";
import { useApiMutation, useApiQuery, useResetSession } from "../api";
import { bootstrapMutation, bootstrapStatusQuery, loginMutation } from "../api/definitions";
import { MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH } from "../auth-constraints";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function SetupPage() {
  const navigate = useNavigate();
  const [temporary, setTemporary] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const bootstrap = useApiQuery(bootstrapStatusQuery());
  const submitBootstrap = useApiMutation(bootstrapMutation);
  const login = useApiMutation(loginMutation);
  const resetSession = useResetSession();
  const needsTemporary = bootstrap.data?.requires_temporary_password ?? false;

  const displayError = error ?? (bootstrap.error ? "Cannot reach the Operator server." : null);

  const tooShort = password.length > 0 && password.length < MIN_PASSWORD_LENGTH;
  const mismatch = confirm.length > 0 && password !== confirm;
  const ready =
    password.length >= MIN_PASSWORD_LENGTH &&
    password === confirm &&
    (!needsTemporary || temporary.length > 0);

  const submit = useCallback(
    async (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      setError(null);
      try {
        const result = await submitBootstrap.mutateAsync({
          temporary_password: needsTemporary ? temporary : null,
          new_password: password,
        });
        await login.mutateAsync({ username: result.username, password });
        resetSession();
        void navigate("/onboarding", { replace: true });
      } catch (e) {
        if (e instanceof ApiError && e.status === 409) {
          setError("This server already has an admin account. Sign in instead.");
        } else if (e instanceof ApiError && e.status === 401) {
          setError("The temporary password is incorrect.");
        } else if (e instanceof ApiError && e.status === 429) {
          setError("Too many attempts. Wait a moment and try again.");
        } else {
          setError(e instanceof ApiError ? e.message : "Setup failed.");
        }
      }
    },
    [needsTemporary, temporary, password, navigate, submitBootstrap, login, resetSession],
  );

  if (bootstrap.data?.state === "complete") {
    return <Navigate to="/login" replace />;
  }

  if (bootstrap.isLoading) {
    return <AuthCard title="Set up Operator">Checking server status…</AuthCard>;
  }

  if (bootstrap.error) {
    return (
      <AuthCard title="Set up Operator" error={bootstrap.error.message}>
        The Operator server status could not be loaded. Check the server and reload this page.
      </AuthCard>
    );
  }

  return (
    <AuthCard
      title="Set up Operator"
      subtitle="Choose the admin password for this workspace. Operator has a single human account."
      error={displayError}
      notice={
        needsTemporary
          ? "This server was started with a bootstrap secret. Enter it to claim the admin account."
          : null
      }
      onSubmit={submit}
      actions={
        <AuthSubmit
          busy={submitBootstrap.isPending || login.isPending}
          busyLabel="Creating…"
          disabled={!ready}
        >
          Create admin account
        </AuthSubmit>
      }
    >
      {needsTemporary && (
        <AuthField
          label="Temporary password"
          type="password"
          autoComplete="one-time-code"
          maxLength={MAX_PASSWORD_LENGTH}
          value={temporary}
          onChange={setTemporary}
          required
        />
      )}
      <AuthField
        label="New admin password"
        type="password"
        autoComplete="new-password"
        maxLength={MAX_PASSWORD_LENGTH}
        value={password}
        onChange={setPassword}
        hint={
          tooShort
            ? `At least ${MIN_PASSWORD_LENGTH} characters.`
            : `A passphrase of ${MIN_PASSWORD_LENGTH} characters or more.`
        }
        required
      />
      <AuthField
        label="Confirm password"
        type="password"
        autoComplete="new-password"
        maxLength={MAX_PASSWORD_LENGTH}
        value={confirm}
        onChange={setConfirm}
        hint={mismatch ? "Passwords do not match." : undefined}
        required
      />
    </AuthCard>
  );
}
