import { useState, useCallback } from "react";
import { Link } from "react-router-dom";
import { useApiMutation } from "../api";
import { forgotPasswordMutation } from "../api/definitions";
import { MAX_USERNAME_LENGTH } from "../auth-constraints";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function ForgotPasswordPage() {
  const [username, setUsername] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const forgot = useApiMutation(forgotPasswordMutation);
  const submit = useCallback(
    async (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      try {
        const response = await forgot.mutateAsync({ username });
        setMessage(response.message);
      } catch {
        setMessage(
          "Recovery instructions are unavailable. Ask the server administrator to run `operator auth reset-admin-password` locally.",
        );
      }
    },
    [forgot, username],
  );

  return (
    <AuthCard
      title="Forgot password"
      subtitle="Operator recovery is performed by the server administrator."
      notice={message}
      onSubmit={submit}
      actions={
        <AuthSubmit busy={forgot.isPending} busyLabel="Checking…" disabled={!username}>
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
