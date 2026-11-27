import { useState, useCallback } from "react";
import { useSearchParams } from "react-router-dom";
import { ApiError } from "../api-client";
import { useApiMutation, useApiQuery } from "../api";
import { approveDeviceMutation, currentSessionQuery } from "../api/definitions";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function DevicePage() {
  const [params, setParams] = useSearchParams();
  const suppliedCode = params.get("user_code");
  const [userCode, setUserCode] = useState(suppliedCode ?? "");
  const [approved, setApproved] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const session = useApiQuery(currentSessionQuery(), { enabled: !!suppliedCode });
  const approve = useApiMutation(approveDeviceMutation);

  const submit = useCallback(
    async (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      if (!suppliedCode) {
        setParams({ user_code: userCode.trim() }, { replace: true });
        return;
      }
      setError(null);
      try {
        const res = await approve.mutateAsync({ userCode: userCode.trim() });
        setApproved(res.client_id);
      } catch (e) {
        setError(
          e instanceof ApiError && e.status === 404
            ? "That code is unknown or has expired. Start the connection again from your editor."
            : e instanceof ApiError && e.status === 403
              ? "Your account lacks device approval access. Ask your Operator administrator to approve your account and grant access to this profile."
              : "Approval failed. Try again.",
        );
      }
    },
    [approve, userCode, suppliedCode, setParams],
  );

  const onCodeChange = useCallback((value: string) => setUserCode(value.toUpperCase()), []);

  if (suppliedCode && !session.data) {
    return (
      <AuthCard title="Approve a device" error={session.error?.message}>
        Checking sign-in...
      </AuthCard>
    );
  }

  if (approved) {
    return (
      <AuthCard title="Device approved">
        <p>
          <strong>{approved}</strong> now has access. You can close this page and return to your
          editor.
        </p>
      </AuthCard>
    );
  }

  return (
    <AuthCard
      title="Approve a device"
      subtitle={
        suppliedCode && session.data
          ? `Signed in as ${session.data.subject}. Compare the code with the one in your editor before approving access.`
          : "Enter the code shown in your editor to continue."
      }
      error={error}
      onSubmit={submit}
      actions={
        <AuthSubmit busy={approve.isPending} busyLabel="Approving..." disabled={!userCode.trim()}>
          {suppliedCode ? "Approve" : "Continue"}
        </AuthSubmit>
      }
    >
      <AuthField
        label="Code"
        value={userCode}
        placeholder="XXXX-XXXX"
        onChange={onCodeChange}
        required
      />
    </AuthCard>
  );
}
