import type { IssueTypeSummary } from "../generated/IssueTypeSummary";
import type { ProjectSummary } from "../generated/ProjectSummary";
import styles from "./TicketCreateForm.module.css";

export interface TicketCreateFormValue {
  issueType: string;
  project: string;
  summary: string;
}

export interface TicketCreateFormProps {
  value: TicketCreateFormValue;
  issueTypes: IssueTypeSummary[];
  projects: ProjectSummary[];
  busy?: boolean;
  error?: string | null;
  onChange: (value: TicketCreateFormValue) => void;
  onSubmit: () => void;
}

export function TicketCreateForm({
  value,
  issueTypes,
  projects,
  busy = false,
  error,
  onChange,
  onSubmit,
}: TicketCreateFormProps) {
  const update = <K extends keyof TicketCreateFormValue>(key: K, next: TicketCreateFormValue[K]) =>
    onChange({ ...value, [key]: next });

  const ready = value.issueType !== "" && value.project !== "" && value.summary.trim() !== "";

  return (
    <form
      className={styles.form}
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <label className={styles.field}>
        <span className={styles.fieldLabel}>Issue type</span>
        <select
          className={styles.select}
          value={value.issueType}
          onChange={(event) => update("issueType", event.target.value)}
          disabled={busy}
          required
        >
          <option value="">Choose a type…</option>
          {issueTypes.map((issueType) => (
            <option key={issueType.key} value={issueType.key}>
              {issueType.glyph} {issueType.key} — {issueType.name}
            </option>
          ))}
        </select>
      </label>
      <label className={styles.field}>
        <span className={styles.fieldLabel}>Project</span>
        <select
          className={styles.select}
          value={value.project}
          onChange={(event) => update("project", event.target.value)}
          disabled={busy}
          required
        >
          <option value="">Choose a project…</option>
          {projects.map((project) => (
            <option key={project.project_name} value={project.project_name}>
              {project.project_name}
            </option>
          ))}
        </select>
      </label>
      <label className={styles.field}>
        <span className={styles.fieldLabel}>Summary</span>
        <input
          className={styles.input}
          type="text"
          value={value.summary}
          placeholder="What needs doing?"
          onChange={(event) => update("summary", event.target.value)}
          disabled={busy}
          required
        />
      </label>
      {error && <div className={styles.error}>{error}</div>}
      <button type="submit" className={styles.createBtn} disabled={busy || !ready}>
        {busy ? "Creating…" : "Add to queue"}
      </button>
    </form>
  );
}
