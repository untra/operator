import { useState, useCallback } from "react";
import { Link } from "react-router-dom";
import { useHost } from "../host";
import { OperatorApi } from "../api-client";
import { MAX_USERNAME_LENGTH } from "../auth-constraints";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function ForgotPasswordPage() {
  const host = useHost();
  const [username, setUsername] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const submit = useCallback(
    async (event: React.SubmitEvent<HTMLFormElement>) => {
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
    },
    [host, username],
  );

  return (
    <AuthCard
      title="Forgot password"
      subtitle="Operator recovery is performed by the server administrator."
      notice={message}
      onSubmit={submit}
      actions={
        <AuthSubmit busy={busy} busyLabel="Checking…" disabled={!username}>
          Get recovery instructions
        </AuthSubmit>
      }
      links={<Link to="/login">Back to sign in</Link>}
    >
      <AuthField
        label="Username"
        autoComplete="username"
        maxLength={MAX_USERNAME_LENGTH}
        value={username}
        onChange={setUsername}
        required
      />
    </AuthCard>
  );
}
