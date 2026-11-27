import { useState } from "react";
import { Link } from "react-router-dom";
import type { CreateIssueTypeRequest } from "@operator/bindings/CreateIssueTypeRequest";
import { useApiMutation, useApiQuery } from "../api";
import {
  collectionsQuery,
  createIssueTypeMutation,
  sampleWorkflowMutation,
} from "../api/definitions";
import styles from "./WorkflowForms.module.css";

const summaryField = {
  name: "summary",
  description: "Ticket summary",
  field_type: "string",
  required: true,
  default: null,
  options: [],
  placeholder: null,
  max_length: null,
  user_editable: true,
};
const firstStep = {
  name: "execute",
  display_name: null,
  prompt: "",
  outputs: [],
  allowed_tools: [],
  review_type: "none",
  next_step: null,
  permission_mode: "default",
};

type Draft = Omit<CreateIssueTypeRequest, "fields" | "steps"> & {
  fields: (CreateIssueTypeRequest["fields"][number] & { formId: string })[];
  steps: (CreateIssueTypeRequest["steps"][number] & { formId: string })[];
};

export function NewWorkflowPage() {
  const [draft, setDraft] = useState<Draft>({
    key: "",
    name: "",
    description: "",
    glyph: ">",
    color: null,
    mode: "autonomous",
    project_required: false,
    fields: [{ ...summaryField, formId: "summary" }],
    steps: [{ ...firstStep, formId: "first" }],
  });
  const [values, setValues] = useState<Record<string, string>>({});
  const [sample, setSample] = useState<{ input: string; markdown: string }>();
  const [saved, setSaved] = useState<string>();
  const collections = useApiQuery(collectionsQuery());
  const create = useApiMutation(createIssueTypeMutation);
  const preview = useApiMutation(sampleWorkflowMutation);
  const update = (patch: Partial<Draft>) => setDraft({ ...draft, ...patch });
  const definition = {
    ...draft,
    key:
      draft.key ||
      draft.name
        .toUpperCase()
        .replace(/[^A-Z0-9]+/g, "_")
        .slice(0, 16),
    fields: draft.fields.map(({ formId: _formId, ...field }) => field),
    steps: draft.steps.map(({ formId: _formId, ...step }, index) => ({
      ...step,
      next_step: draft.steps[index + 1]?.name ?? null,
    })),
  };
  const sampleRequest = {
    definition,
    values: Object.fromEntries(
      Object.entries(values).filter(
        ([key]) =>
          definition.fields.some((field) => field.name === key) ||
          (key === "project" && draft.project_required),
      ),
    ),
  };
  const input = JSON.stringify(sampleRequest);
  if (saved) {
    return (
      <section className={styles.page}>
        <h1>Workflow saved</h1>
        <Link to={`/tickets/create?workflow=${encodeURIComponent(saved)}`}>
          Create tickets with this workflow
        </Link>
      </section>
    );
  }
  return (
    <form
      className={styles.page}
      onSubmit={(event) => {
        event.preventDefault();
        create.mutate(definition, {
          onSuccess: (result) => setSaved(`${result.collection}/${result.key}`),
        });
      }}
    >
      <h1>New workflow</h1>
      <label>
        Name
        <input required value={draft.name} onChange={(e) => update({ name: e.target.value })} />
      </label>
      <label>
        Outcome
        <textarea
          required
          value={draft.description}
          onChange={(e) => update({ description: e.target.value })}
        />
      </label>
      <details>
        <summary>Workflow key and collection</summary>
        <label>
          Key
          <input
            pattern="[A-Z][A-Z0-9_]{1,15}"
            value={definition.key}
            onChange={(e) => update({ key: e.target.value })}
          />
        </label>
        <label>
          Collection
          <select
            value={draft.collection ?? ""}
            onChange={(e) => update({ collection: e.target.value || undefined })}
          >
            <option value="">Active collection</option>
            {collections.data?.map((c) => (
              <option key={c.name}>{c.name}</option>
            ))}
          </select>
        </label>
      </details>
      <label>
        Execution mode
        <select value={draft.mode} onChange={(e) => update({ mode: e.target.value })}>
          <option value="autonomous">Autonomous</option>
          <option value="paired">Paired</option>
        </select>
      </label>
      <label>
        <input
          type="checkbox"
          checked={draft.project_required}
          onChange={(e) => update({ project_required: e.target.checked })}
        />{" "}
        Requires a local project
      </label>
      <h2>Ticket inputs</h2>
      {draft.fields.map((field, index) => (
        <fieldset key={field.formId}>
          <legend>Input {index + 1}</legend>
          <label>
            Input name
            <input
              required
              pattern="[a-z][a-z0-9_]*"
              readOnly={index === 0}
              value={field.name}
              onChange={(e) =>
                update({
                  fields: draft.fields.map((f, i) =>
                    i === index ? { ...f, name: e.target.value, description: e.target.value } : f,
                  ),
                })
              }
            />
          </label>
          <label>
            <input
              type="checkbox"
              checked={field.required}
              onChange={(e) =>
                update({
                  fields: draft.fields.map((f, i) =>
                    i === index ? { ...f, required: e.target.checked } : f,
                  ),
                })
              }
            />{" "}
            Required
          </label>
          <label>
            Sample value
            <input
              value={values[field.name] ?? ""}
              onChange={(e) => setValues({ ...values, [field.name]: e.target.value })}
            />
          </label>
          {index > 0 && (
            <button
              type="button"
              onClick={() => update({ fields: draft.fields.filter((_, i) => i !== index) })}
            >
              Remove input
            </button>
          )}
        </fieldset>
      ))}
      <button
        type="button"
        onClick={() =>
          update({
            fields: [
              ...draft.fields,
              {
                ...summaryField,
                formId: crypto.randomUUID(),
                name: "",
                description: "",
                required: false,
              },
            ],
          })
        }
      >
        Add input
      </button>
      {draft.project_required && (
        <label>
          Sample project
          <input
            value={values.project ?? ""}
            onChange={(e) => setValues({ ...values, project: e.target.value })}
          />
        </label>
      )}
      <h2>Ordered steps</h2>
      {draft.steps.map((step, index) => (
        <fieldset key={step.formId}>
          <legend>Step {index + 1}</legend>
          <label>
            Step name
            <input
              required
              pattern="[a-z][a-z0-9_]*"
              value={step.name}
              onChange={(e) =>
                update({
                  steps: draft.steps.map((s, i) =>
                    i === index ? { ...s, name: e.target.value } : s,
                  ),
                })
              }
            />
          </label>
          <label>
            Instructions and expected output
            <textarea
              required
              placeholder="Refer to an input with {{input_name}}"
              value={step.prompt}
              onChange={(e) =>
                update({
                  steps: draft.steps.map((s, i) =>
                    i === index ? { ...s, prompt: e.target.value } : s,
                  ),
                })
              }
            />
          </label>
          {draft.steps.length > 1 && (
            <button
              type="button"
              onClick={() => update({ steps: draft.steps.filter((_, i) => i !== index) })}
            >
              Remove step
            </button>
          )}
        </fieldset>
      ))}
      <button
        type="button"
        onClick={() =>
          update({
            steps: [...draft.steps, { ...firstStep, formId: crypto.randomUUID(), name: "" }],
          })
        }
      >
        Add step
      </button>
      <button
        type="button"
        disabled={preview.isPending}
        onClick={() =>
          preview.mutate(sampleRequest, { onSuccess: (markdown) => setSample({ input, markdown }) })
        }
      >
        Preview sample
      </button>
      {sample?.input === input && <pre>{sample.markdown}</pre>}
      <button disabled={create.isPending}>Save workflow</button>
      <p role="alert">
        {create.error?.message ?? preview.error?.message ?? collections.error?.message}
      </p>
    </form>
  );
}
