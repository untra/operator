// First-run admin setup. Reachable only while the server has no usable admin
// account; once bootstrap completes this redirects to login, so it cannot be
// used to re-claim the account.

import { useEffect, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useHost } from "../host";
import { OperatorApi, ApiError } from "../api-client";
import { MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH } from "../auth-constraints";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function SetupPage() {
  const host = useHost();
  const navigate = useNavigate();
  const [needsTemporary, setNeedsTemporary] = useState(false);
  const [temporary, setTemporary] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const api = new OperatorApi(host);
    api
      .bootstrapStatus()
      .then((status) => {
        if (status.state === "complete") {
          void navigate("/login", { replace: true });
        } else {
          setNeedsTemporary(status.requires_temporary_password);
        }
        return undefined;
      })
      .catch(() => setError("Cannot reach the Operator server."));
  }, [host, navigate]);

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
      setBusy(true);
      try {
        const api = new OperatorApi(host);
        const result = await api.bootstrap({
          temporary_password: needsTemporary ? temporary : null,
          new_password: password,
        });
        // Bootstrap creates the account but does not sign you in.
        await api.login(result.username, password);
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
      } finally {
        setBusy(false);
      }
    },
    [host, needsTemporary, temporary, password, navigate],
  );

  return (
    <AuthCard
      title="Set up Operator"
      subtitle="Choose the admin password for this workspace. Operator has a single human account."
      error={error}
      notice={
        needsTemporary
          ? "This server was started with a bootstrap secret. Enter it to claim the admin account."
          : null
      }
      onSubmit={submit}
      actions={
        <AuthSubmit busy={busy} busyLabel="Creating…" disabled={!ready}>
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
