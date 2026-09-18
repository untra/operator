import { useCallback, useEffect, useRef, useState } from "react";
import { TicketCreateForm } from "@operator/webcomponents";
import type { TicketCreateFormValue } from "@operator/webcomponents";
import type { IssueTypeSummary } from "@operator/bindings/IssueTypeSummary";
import type { ProjectSummary } from "@operator/bindings/ProjectSummary";
import { OperatorApi } from "../api-client";
import { useHost } from "../host";
import { useRightPanel } from "../right-panel";
import styles from "./TicketCreatePanel.module.css";

const EMPTY: TicketCreateFormValue = { issueType: "", project: "", summary: "" };

/**
 * Right-panel contents for adding a card to the board. Writes through the same
 * `POST /api/v1/tickets` the CLI and MCP use, so a ticket created here is
 * identical to one created anywhere else.
 */
export function TicketCreatePanel({ onCreated }: { onCreated: () => void }) {
  const host = useHost();
  const { close } = useRightPanel();
  const [api] = useState(() => new OperatorApi(host));

  const [value, setValue] = useState<TicketCreateFormValue>(EMPTY);
  const [issueTypes, setIssueTypes] = useState<IssueTypeSummary[]>([]);
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const createRequest = useRef(0);

  useEffect(
    () => () => {
      createRequest.current += 1;
    },
    [],
  );

  useEffect(() => {
    let cancelled = false;
    Promise.all([api.listIssueTypes(), api.listProjects()])
      .then(([types, projectList]) => {
        if (!cancelled) {
          setIssueTypes(types);
          setProjects(projectList.filter((project) => project.exists));
        }
        return undefined;
      })
      .catch((e: Error) => !cancelled && setError(e.message));
    return () => {
      cancelled = true;
    };
  }, [api]);

  const onSubmit = useCallback(() => {
    const request = ++createRequest.current;
    setBusy(true);
    setError(null);
    api
      .createTicket({
        template: value.issueType,
        project: value.project,
        summary: value.summary,
        values: {},
      })
      .then(() => {
        if (request === createRequest.current) {
          onCreated();
          close();
        }
        return undefined;
      })
      .catch((e: Error) => {
        if (request === createRequest.current) {
          setError(e.message);
        }
      })
      .finally(() => {
        if (request === createRequest.current) {
          setBusy(false);
        }
      });
  }, [api, close, onCreated, value]);

  return (
    <div className={styles.panel}>
      <h2 className={styles.title}>New ticket</h2>
      <p className={styles.hint}>Lands in the TODO queue, ready to launch.</p>
      <TicketCreateForm
        value={value}
        issueTypes={issueTypes}
        projects={projects}
        busy={busy}
        error={error}
        onChange={setValue}
        onSubmit={onSubmit}
      />
    </div>
  );
}
