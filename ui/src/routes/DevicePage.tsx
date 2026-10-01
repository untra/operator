// Device-approval screen for the OAuth device flow.
//
// The human lands here from the URL an IDE client printed, sees which client is
// asking, and approves. It renders inside the authenticated Layout on purpose:
// approving a device grants a credential, so it requires an admin session.

import { useState, useCallback } from "react";
import { useSearchParams } from "react-router-dom";
import { ApiError } from "../api-client";
import { useApiMutation } from "../api";
import { approveDeviceMutation } from "../api/definitions";
import { AuthCard, AuthField, AuthSubmit } from "@operator/webcomponents";

export function DevicePage() {
  const [params] = useSearchParams();
  const [userCode, setUserCode] = useState(params.get("user_code") ?? "");
  const [approved, setApproved] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const approve = useApiMutation(approveDeviceMutation);

  const submit = useCallback(
    async (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      setError(null);
      try {
        const res = await approve.mutateAsync({ userCode: userCode.trim() });
        setApproved(res.client_id);
      } catch (e) {
        setError(
          e instanceof ApiError && e.status === 404
            ? "That code is unknown or has expired. Start the connection again from your editor."
            : "Approval failed.",
        );
      }
    },
    [approve, userCode],
  );

  const onCodeChange = useCallback((value: string) => setUserCode(value.toUpperCase()), []);

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
      subtitle="Enter the code shown in the application requesting access."
      error={error}
      onSubmit={submit}
      actions={
        <AuthSubmit busy={approve.isPending} busyLabel="Approving…" disabled={!userCode}>
          Approve
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
