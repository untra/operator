import { useState, useCallback } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ApiError } from "../api-client";
import { useApiMutation, useResetSession } from "../api";
import { loginMutation, resetPasswordMutation } from "../api/definitions";
import { MAX_PASSWORD_LENGTH, MAX_USERNAME_LENGTH, MIN_PASSWORD_LENGTH } from "../auth-constraints";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function ResetPasswordPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const reset = useApiMutation(resetPasswordMutation);
  const login = useApiMutation(loginMutation);
  const resetSession = useResetSession();
  const ready =
    username.length > 0 &&
    currentPassword.length > 0 &&
    newPassword.length >= MIN_PASSWORD_LENGTH &&
    newPassword === confirmPassword;

  const submit = useCallback(
    async (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      setError(null);
      try {
        await reset.mutateAsync({
          username,
          current_password: currentPassword,
          new_password: newPassword,
        });
        await login.mutateAsync({ username, password: newPassword });
        resetSession();
        void navigate("/", { replace: true });
      } catch (caught) {
        const status = caught instanceof ApiError ? caught.status : 0;
        setError(
          status === 429
            ? "Too many attempts. Wait a moment and try again."
            : status === 400
              ? caught instanceof ApiError
                ? caught.message
                : "The new password is invalid."
              : "Incorrect username or current password.",
        );
      }
    },
    [reset, login, username, currentPassword, newPassword, navigate, resetSession],
  );

  return (
    <AuthCard
      title="Change password"
      subtitle="Changing the password revokes all sessions and access keys."
      error={error}
      onSubmit={submit}
      actions={
        <AuthSubmit
          busy={reset.isPending || login.isPending}
          busyLabel="Changing…"
          disabled={!ready}
        >
          Change password
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
      <AuthField
        label="Current password"
        type="password"
        maxLength={MAX_PASSWORD_LENGTH}
        value={currentPassword}
        onChange={setCurrentPassword}
        required
      />
      <AuthField
        label="New password"
        type="password"
        maxLength={MAX_PASSWORD_LENGTH}
        value={newPassword}
        onChange={setNewPassword}
        hint={`At least ${MIN_PASSWORD_LENGTH} characters.`}
        required
      />
      <AuthField
        label="Confirm new password"
        type="password"
        maxLength={MAX_PASSWORD_LENGTH}
        value={confirmPassword}
        onChange={setConfirmPassword}
        required
      />
    </AuthCard>
  );
}
