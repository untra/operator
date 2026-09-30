import { useCallback, useState } from "react";
import { TicketCreateForm } from "@operator/webcomponents";
import type { TicketCreateFormValue } from "@operator/webcomponents";
import { useApiMutation, useApiQuery } from "../api";
import { createTicketMutation, issueTypesQuery, projectsQuery } from "../api/definitions";
import { useRightPanel } from "../right-panel";
import styles from "./TicketCreatePanel.module.css";

const EMPTY: TicketCreateFormValue = { issueType: "", project: "", summary: "" };

/**
 * Right-panel contents for adding a card to the board. Writes through the same
 * `POST /api/v1/tickets` the CLI and MCP use, so a ticket created here is
 * identical to one created anywhere else.
 */
export function TicketCreatePanel({ onCreated }: { onCreated: () => void }) {
  const { close } = useRightPanel();
  const [value, setValue] = useState<TicketCreateFormValue>(EMPTY);
  const types = useApiQuery(issueTypesQuery());
  const projects = useApiQuery(projectsQuery());
  const create = useApiMutation(createTicketMutation);
  const issueTypes = types.data ?? [];
  const projectList = (projects.data ?? []).filter((project) => project.exists);
  const error = create.error?.message ?? types.error?.message ?? projects.error?.message ?? null;
  const loading = types.isLoading || projects.isLoading;
  const empty = !loading && !error && (issueTypes.length === 0 || projectList.length === 0);

  const onSubmit = useCallback(() => {
    create.mutate(
      {
        template: value.issueType,
        project: value.project,
        summary: value.summary,
        values: {},
      },
      {
        onSuccess: () => {
          onCreated();
          close();
        },
      },
    );
  }, [close, create, onCreated, value]);

  return (
    <div className={styles.panel}>
      <h2 className={styles.title}>New ticket</h2>
      <p className={styles.hint}>Lands in the TODO queue, ready to launch.</p>
      {loading ? (
        <p className={styles.hint}>Loading issue types and projects…</p>
      ) : empty ? (
        <p className={styles.hint}>
          {issueTypes.length === 0
            ? "No issue types are configured. Add one before creating a ticket."
            : "No available projects were found."}
        </p>
      ) : (
        <TicketCreateForm
          value={value}
          issueTypes={issueTypes}
          projects={projectList}
          busy={create.isPending}
          error={error}
          onChange={setValue}
          onSubmit={onSubmit}
        />
      )}
    </div>
  );
}
