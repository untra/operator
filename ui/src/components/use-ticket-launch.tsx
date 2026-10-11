import { useCallback, useMemo, useState, type ReactNode } from "react";
import { LaunchForm } from "@operator/webcomponents";
import type { LaunchFormValue } from "@operator/webcomponents";
import type { LaunchTicketResponse } from "../api-client";
import { useApiMutation, useApiQuery } from "../api";
import {
  configurationQuery,
  delegatorsQuery,
  executionTargetsQuery,
  launchTicketMutation,
} from "../api/definitions";
import styles from "./TicketDetailPanel.module.css";

const EMPTY_FORM: LaunchFormValue = { delegator: "", wrapper: "", target: "", yolo: false };

/** The launch form for one ticket, and the launch it produced. */
export function useTicketLaunch(ticketId: string): {
  controls: ReactNode;
  result: LaunchTicketResponse | undefined;
} {
  const [form, setForm] = useState<LaunchFormValue>(EMPTY_FORM);
  const config = useApiQuery(configurationQuery());
  const delegators = useApiQuery(delegatorsQuery());
  const targets = useApiQuery(executionTargetsQuery());
  const launch = useApiMutation(launchTicketMutation);

  const onLaunch = useCallback(() => {
    launch.mutate({
      ticketId,
      options: {
        delegator: form.delegator || null,
        provider: null,
        model: null,
        model_server: null,
        yolo_mode: form.yolo,
        wrapper: form.wrapper || null,
        retry_reason: null,
        resume_session_id: null,
        target: form.target || null,
      },
    });
  }, [form, launch, ticketId]);

  const delegatorList = useMemo(() => delegators.data?.delegators ?? [], [delegators.data]);
  const targetNames = useMemo(
    () => (targets.data?.targets ?? []).filter((item) => item.available).map((item) => item.name),
    [targets.data],
  );

  const loading = config.isLoading || delegators.isLoading || targets.isLoading;
  const error = config.error?.message ?? delegators.error?.message ?? targets.error?.message;

  const controls = loading ? (
    <div className={styles.loading}>Loading launch configuration…</div>
  ) : error ? (
    <div className={styles.error}>Launch options unavailable: {error}</div>
  ) : (
    <LaunchForm
      value={form}
      delegators={delegatorList}
      targets={targetNames}
      defaultWrapperLabel={config.data?.launch.session_wrapper ?? "configured"}
      busy={launch.isPending}
      error={launch.error?.message ?? null}
      onChange={setForm}
      onSubmit={onLaunch}
    />
  );

  return { controls, result: launch.data ?? undefined };
}
