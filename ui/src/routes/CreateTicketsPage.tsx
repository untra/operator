import { useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import type { WorkflowRecipe } from "@operator/bindings/WorkflowRecipe";
import type { BatchPreview } from "@operator/bindings/BatchPreview";
import { useApiMutation, useApiQuery } from "../api";
import {
  boardRecordsMutation,
  boardMappingMutation,
  workflowBoardsQuery,
  parseRecordsMutation,
  batchCreateMutation,
  batchPreviewMutation,
  collectionsQuery,
  saveRecipeMutation,
  selectedWorkflowQuery,
  workflowRecipesQuery,
} from "../api/definitions";
import type { WorkflowDataset } from "@operator/bindings/WorkflowDataset";
import styles from "./WorkflowForms.module.css";

function emptyRecipe(id: string): WorkflowRecipe {
  const [collection = "", workflow = ""] = id.split("/");
  return { collection, workflow, source: "", shared: {}, key_columns: [], repeat: {}, columns: {} };
}

export function CreateTicketsPage() {
  const [params] = useSearchParams();
  const [recipe, setRecipe] = useState(() => emptyRecipe(params.get("workflow") ?? ""));
  const [boardId, setBoardId] = useState("");
  const [chosen, setChosen] = useState<string[]>([]);
  const [writers, setWriters] = useState<Record<string, string>>({});
  const [boardRows, setBoardRows] = useState<{ id: string; rows: WorkflowDataset["rows"] }>();
  const [mode, setMode] = useState("one");
  const [text, setText] = useState("");
  const [name, setName] = useState("");
  const [error, setError] = useState("");
  const [review, setReview] = useState<{ input: string; result: BatchPreview }>();
  const [created, setCreated] = useState<number>();
  const [sample, setSample] = useState(0);
  const id = recipe.workflow ? `${recipe.collection}/${recipe.workflow}` : "";
  const boards = useApiQuery(workflowBoardsQuery(), { enabled: mode === "board" });
  const board = boards.data?.find((option) => JSON.stringify(option) === boardId);
  const provider = useApiMutation(boardRecordsMutation);
  const mapping = useApiMutation(boardMappingMutation);
  const collections = useApiQuery(collectionsQuery());
  const detail = useApiQuery(selectedWorkflowQuery(id), { enabled: Boolean(id) });
  const recipes = useApiQuery(workflowRecipesQuery());
  const preview = useApiMutation(batchPreviewMutation);
  const create = useApiMutation(batchCreateMutation);
  const save = useApiMutation(saveRecipeMutation);
  const patch = (values: Partial<WorkflowRecipe>) => setRecipe({ ...recipe, ...values });
  const fieldNames = (detail.data?.fields ?? []).filter((f) => f.user_editable).map((f) => f.name);
  if (detail.data?.project_required && !fieldNames.includes("project")) {
    fieldNames.push("project");
  }
  const parser = useApiMutation(parseRecordsMutation);
  const [parsed, setParsed] = useState<{ text: string; rows: WorkflowDataset["rows"] }>();
  const records = boardRows?.id === boardId ? boardRows.rows : [];
  const rows =
    mode === "board"
      ? records.filter((row) => chosen.includes(row.id))
      : mode === "one"
        ? [{}]
        : parsed?.text === text
          ? parsed.rows
          : [];
  function readRecords(source: string) {
    parser.mutate(
      { text: source },
      { onSuccess: (data) => setParsed({ text: source, rows: data }) },
    );
  }
  const headers = Array.from(new Set(rows.flatMap((row) => Object.keys(row))));
  const effectiveRecipe =
    mode === "one" ? { ...recipe, key_columns: [], columns: {}, repeat: {} } : recipe;
  const request =
    mode === "board" && board
      ? {
          campaign: null,
          tickets: [],
          board: {
            scope: board,
            recipe,
            issue_ids: rows.map((row) => row.id),
            status_writers: writers,
          },
        }
      : { campaign: null, tickets: [], dataset: { recipe: effectiveRecipe, rows } };
  const input = JSON.stringify(request);
  const result = review?.input === input ? review.result : undefined;
  const current = result?.tickets[sample];
  function mapField(field: string, kind: string, value: string) {
    const { columns, shared, repeat } = structuredClone(recipe);
    delete columns[field];
    delete shared[field];
    delete repeat[field];
    if (kind === "column") {
      columns[field] = value;
    }
    if (kind === "shared") {
      shared[field] = value;
    }
    if (kind === "repeat") {
      repeat[field] = value.split("\n");
    }
    patch({ columns, shared, repeat });
  }
  if (created !== undefined) {
    return (
      <section className={styles.page}>
        <h1>
          Created {created} {created === 1 ? "ticket" : "tickets"}
        </h1>
        <Link to="/queue">Open queue</Link>
      </section>
    );
  }
  return (
    <section className={styles.page}>
      <h1>Create tickets</h1>
      <label>
        Saved recipe
        <select
          value=""
          onChange={(e) => {
            const selected = recipes.data?.[e.target.value];
            if (selected) {
              setRecipe(selected);
              setName(e.target.value);
              setMode("rows");
            }
          }}
        >
          <option value="">Choose a recipe</option>
          {Object.keys(recipes.data ?? {}).map((key) => (
            <option key={key}>{key}</option>
          ))}
        </select>
      </label>
      <label>
        Workflow
        <select value={id} onChange={(e) => setRecipe(emptyRecipe(e.target.value))}>
          <option value="">Choose a workflow</option>
          {collections.data?.map((c) => (
            <optgroup key={c.name} label={c.name}>
              {c.types.map((key) => (
                <option key={key} value={`${c.name}/${key}`}>
                  {key}
                </option>
              ))}
            </optgroup>
          ))}
        </select>
      </label>
      <Link to="/workflows/new">New workflow</Link>
      {detail.data && (
        <>
          <h2>{detail.data.name}</h2>
          <p>{detail.data.description}</p>
          <label>
            Ticket source
            <select value={mode} onChange={(e) => setMode(e.target.value)}>
              <option value="one">One ticket</option>
              <option value="rows">Dataset rows</option>
              <option value="board">Provider selection</option>
            </select>
          </label>
          {mode === "board" && (
            <fieldset>
              <legend>Provider records</legend>
              <label>
                Connected board
                <select
                  value={boardId}
                  onChange={(event) => {
                    setBoardId(event.target.value);
                    setChosen([]);
                    setWriters({});
                  }}
                >
                  <option value="">Choose a Linear team or GitHub Project</option>
                  {boards.data?.map((option) => (
                    <option key={JSON.stringify(option)} value={JSON.stringify(option)}>
                      {option.provider} / {option.instance} / {option.project}
                    </option>
                  ))}
                </select>
              </label>
              <button
                disabled={!board || provider.isPending}
                onClick={() =>
                  board &&
                  provider.mutate(board, {
                    onSuccess: (data) => {
                      setBoardRows({ id: boardId, rows: data });
                      setChosen(data.map((row) => row.id));
                      setWriters({});
                    },
                  })
                }
              >
                Load issues
              </button>
              <label>
                Workflow selection
                <select
                  value={recipe.unresolved_labels ?? ""}
                  onChange={(event) => {
                    const policy = event.target.value;
                    patch({
                      unresolved_labels:
                        policy === "review" || policy === "skip" || policy === "use_selected"
                          ? policy
                          : undefined,
                    });
                  }}
                >
                  <option value="">Use the selected workflow for every issue</option>
                  <option value="review">Match labels and review unresolved issues</option>
                  <option value="skip">Match labels and skip unresolved issues</option>
                  <option value="use_selected">
                    Match labels and use the selected workflow as default
                  </option>
                </select>
              </label>
              <label>
                <input
                  type="checkbox"
                  checked={Boolean(recipe.include_legacy)}
                  onChange={(event) => patch({ include_legacy: event.target.checked })}
                />
                Include new variants of previously imported issues
              </label>
              {records.map((row) => (
                <label key={row.id}>
                  <input
                    type="checkbox"
                    checked={chosen.includes(row.id)}
                    onChange={(event) => {
                      setChosen(
                        event.target.checked
                          ? [...chosen, row.id]
                          : chosen.filter((issueId) => issueId !== row.id),
                      );
                      setWriters({});
                    }}
                  />
                  {row.key} / {row.summary}{" "}
                  {recipe.unresolved_labels && (row.mapped_workflows || "(unmapped)")}
                </label>
              ))}
            </fieldset>
          )}
          {mode === "rows" && (
            <>
              <label>
                CSV or JSON file
                <input
                  type="file"
                  accept=".csv,.json,text/csv,application/json"
                  onChange={async (e) => {
                    const file = e.target.files?.[0];
                    if (file) {
                      try {
                        const content = await file.text();
                        setText(content);
                        readRecords(content);
                        setError("");
                      } catch {
                        setError("Could not read the file.");
                      }
                    }
                  }}
                />
              </label>
              <label>
                Paste records
                <textarea value={text} onChange={(e) => setText(e.target.value)} />
              </label>
              <button disabled={!text.trim() || parser.isPending} onClick={() => readRecords(text)}>
                Read records
              </button>
              <label>
                Source name
                <input value={recipe.source} onChange={(e) => patch({ source: e.target.value })} />
              </label>
              <fieldset>
                <legend>Stable key columns for repeat imports</legend>
                {headers.map((h) => (
                  <label key={h}>
                    <input
                      type="checkbox"
                      checked={recipe.key_columns.includes(h)}
                      onChange={(e) =>
                        patch({
                          key_columns: e.target.checked
                            ? [...recipe.key_columns, h]
                            : recipe.key_columns.filter((k) => k !== h),
                        })
                      }
                    />
                    {h}
                  </label>
                ))}
              </fieldset>
            </>
          )}
          <div className={styles.columns}>
            <div>
              {fieldNames.map((field) => {
                const kind =
                  mode === "one"
                    ? "shared"
                    : Object.hasOwn(recipe.repeat, field)
                      ? "repeat"
                      : Object.hasOwn(recipe.columns, field)
                        ? "column"
                        : Object.hasOwn(recipe.shared, field)
                          ? "shared"
                          : "default";
                return (
                  <fieldset key={field}>
                    <legend>{field}</legend>
                    {mode !== "one" && (
                      <label>
                        Value from
                        <select
                          value={kind}
                          onChange={(e) =>
                            mapField(
                              field,
                              e.target.value,
                              e.target.value === "column"
                                ? headers.includes(field)
                                  ? field
                                  : (headers[0] ?? "")
                                : "",
                            )
                          }
                        >
                          <option value="default">Workflow default</option>
                          <option value="column">Row column</option>
                          <option value="shared">Shared value</option>
                          <option value="repeat">Repeat each row for these values</option>
                        </select>
                      </label>
                    )}
                    {kind === "column" && (
                      <label>
                        Column
                        <select
                          value={recipe.columns[field]}
                          onChange={(e) => mapField(field, kind, e.target.value)}
                        >
                          {headers.map((h) => (
                            <option key={h}>{h}</option>
                          ))}
                        </select>
                      </label>
                    )}
                    {kind === "shared" && (
                      <label>
                        {field} value
                        <input
                          value={recipe.shared[field] ?? ""}
                          onChange={(e) => mapField(field, kind, e.target.value)}
                        />
                      </label>
                    )}
                    {kind === "repeat" && (
                      <label>
                        One value per line
                        <textarea
                          value={recipe.repeat[field]?.join("\n") ?? ""}
                          onChange={(e) => mapField(field, kind, e.target.value)}
                        />
                      </label>
                    )}
                  </fieldset>
                );
              })}
            </div>
            <div aria-live="polite">
              {result && (
                <>
                  <h2>Review tickets</h2>
                  <p>
                    {rows.length} source rows / {result.tickets.length} planned ·{" "}
                    {Object.keys(result.previously_created).length} previously created
                  </p>
                  {result.errors.map((message) => (
                    <p key={message}>{message}</p>
                  ))}
                  {Object.entries(result.previously_created).map(([row, ticket]) => (
                    <p key={row}>
                      {row} / {ticket}
                    </p>
                  ))}
                  {result.tickets.length > 0 && (
                    <label>
                      Inspect ticket
                      <select value={sample} onChange={(e) => setSample(Number(e.target.value))}>
                        {result.tickets.map((item, index) => (
                          <option key={item.ref} value={index}>
                            {item.ref} / {item.errors.length ? "Needs review" : item.id}
                          </option>
                        ))}
                      </select>
                    </label>
                  )}
                  {mode === "board" && board && current && (
                    <button
                      onClick={() => {
                        const issueId = current.source_record;
                        if (issueId) {
                          const next = { ...writers };
                          if (next[issueId] === current.ref) {
                            delete next[issueId];
                          } else {
                            next[issueId] = current.ref;
                          }
                          setWriters(next);
                        }
                      }}
                    >
                      {current.source_record && writers[current.source_record] === current.ref
                        ? "Keep this issue local-only"
                        : "Use this ticket for upstream status updates"}
                    </button>
                  )}
                  {current && (
                    <>
                      <p>{current.errors.join(", ")}</p>
                      <pre>{current.markdown}</pre>
                    </>
                  )}
                </>
              )}
            </div>
          </div>
          {mode === "board" && board && (
            <button
              disabled={!result?.valid || mapping.isPending}
              onClick={() => mapping.mutate({ scope: board, recipe })}
            >
              Save mapping for future syncs
            </button>
          )}
          {mapping.data && <output>{mapping.data}</output>}
          {mode === "rows" && (
            <fieldset>
              <legend>Reuse this mapping</legend>
              <label>
                Recipe name
                <input value={name} onChange={(e) => setName(e.target.value)} />
              </label>
              <button
                disabled={!name.trim() || !recipe.key_columns.length || save.isPending}
                onClick={() => save.mutate({ name, recipe })}
              >
                Save recipe
              </button>
            </fieldset>
          )}
          <button
            disabled={!rows.length || preview.isPending || create.isPending}
            onClick={() =>
              preview.mutate(request, {
                onSuccess: (data) => {
                  setReview({ input, result: data });
                  setSample(0);
                },
              })
            }
          >
            Preview tickets
          </button>
          {result?.valid && result.tickets.length > 0 && (
            <button
              disabled={create.isPending}
              onClick={() =>
                create.mutate(
                  { ...request, confirmation: result.confirmation },
                  {
                    onSuccess: (data) => setCreated(data.tickets.length),
                    onError: () => setReview(undefined),
                  },
                )
              }
            >
              {mode === "board"
                ? "Import this selection"
                : `Create ${result.tickets.length} ${result.tickets.length === 1 ? "ticket" : "tickets"}`}
            </button>
          )}
        </>
      )}
      <p role="alert">
        {error ||
          boards.error?.message ||
          provider.error?.message ||
          mapping.error?.message ||
          parser.error?.message ||
          detail.error?.message ||
          collections.error?.message ||
          create.error?.message ||
          preview.error?.message ||
          save.error?.message ||
          recipes.error?.message}
      </p>
    </section>
  );
}
