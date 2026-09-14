import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { SetupStep } from "@operator/bindings/SetupStep";
import type { SetupStatusResponse } from "../../api-client";
import { ApiError, OperatorApi } from "../../api-client";
import { useHost } from "../../host";
import { STEP_COMPONENTS, visibleSteps } from "./steps";
import type { WizardDraft } from "./types";
import styles from "./OnboardingPage.module.css";

export function OnboardingPage() {
  const host = useHost();
  const navigate = useNavigate();
  const [api] = useState(() => new OperatorApi(host));
  const [status, setStatus] = useState<SetupStatusResponse | null>(null);
  const [steps, setSteps] = useState<Awaited<ReturnType<typeof api.setupSteps>>>([]);
  const [integrations, setIntegrations] = useState<Awaited<ReturnType<typeof api.integrations>>>(
    [],
  );
  const [collections, setCollections] = useState<Awaited<ReturnType<typeof api.setupCollections>>>(
    [],
  );
  const [draft, setDraft] = useState<WizardDraft>({
    preset: "devops_kanban",
    taskFields: ["priority", "points", "user_story"],
    wrapper: "tmux",
    executionTarget: { kind: "local" },
    coderParameters: [],
    useWorktrees: false,
    acceptanceCriteria: "",
    modelServers: [],
    hostedCollectionIds: [],
  });
  const [currentSlug, setCurrentSlug] = useState<SetupStep>("welcome");
  const [exports, setExports] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let active = true;
    Promise.all([
      api.refreshCsrf(),
      api.setupStatus(),
      api.setupSteps(),
      api.integrations(),
      api.setupCollections(),
    ])
      .then(([, nextStatus, nextSteps, nextIntegrations, nextCollections]) => {
        if (!active) {
          return undefined;
        }
        if (nextStatus.initialized) {
          void navigate("/", { replace: true });
          return undefined;
        }
        setStatus(nextStatus);
        setSteps(nextSteps);
        setIntegrations(nextIntegrations);
        setCollections(nextCollections);
        setDraft({
          preset: "devops_kanban",
          taskFields: ["priority", "points", "user_story"],
          wrapper: "tmux",
          executionTarget: { kind: "local" },
          coderParameters: [],
          useWorktrees: false,
          acceptanceCriteria: nextStatus.default_acceptance_criteria,
          modelServers: [],
          hostedCollectionIds: [],
        });
        return undefined;
      })
      .catch(
        (cause: unknown) =>
          active && setError(cause instanceof Error ? cause.message : "Could not load setup"),
      );
    return () => {
      active = false;
    };
  }, [api, navigate]);

  const walk = useMemo(
    () =>
      visibleSteps(
        steps.map((step) => step.slug),
        draft,
      ),
    [draft, steps],
  );
  const currentIndex = Math.max(0, walk.indexOf(currentSlug));
  const current = steps.find((step) => step.slug === walk[currentIndex]);
  const Step = current ? STEP_COMPONENTS[current.slug] : null;

  function addExport(value: string) {
    setExports((currentExports) =>
      currentExports.includes(value) ? currentExports : [...currentExports, value],
    );
  }

  function next() {
    if (!current) {
      return;
    }
    if (current.slug === "hosted-collections" && draft.hostedCollectionIds.length === 0) {
      setError("Select at least one collection.");
      return;
    }
    if (
      current.slug === "execution-target" &&
      draft.executionTarget.kind === "coder" &&
      (!draft.executionTarget.name.trim() || !draft.executionTarget.template.trim())
    ) {
      setError("Coder target name and template are required.");
      return;
    }
    if (current.slug === "execution-target" && draft.executionTarget.kind === "coder") {
      const names = draft.coderParameters.map((parameter) => parameter.name.trim());
      if (names.some((name) => !name.trim())) {
        setError("Coder parameter names cannot be empty.");
        return;
      }
      if (new Set(names).size !== names.length) {
        setError("Coder parameter names must be unique.");
        return;
      }
    }
    setError(null);
    setCurrentSlug(walk[Math.min(currentIndex + 1, walk.length - 1)]);
  }

  async function initialize() {
    setBusy(true);
    setError(null);
    try {
      const executionTarget: WizardDraft["executionTarget"] =
        draft.executionTarget.kind === "coder"
          ? {
              ...draft.executionTarget,
              parameters: Object.fromEntries(
                draft.coderParameters.map(({ name, value }) => [name.trim(), value]),
              ),
            }
          : draft.executionTarget;
      await api.initializeSetup({
        preset: draft.preset,
        task_fields: draft.taskFields,
        wrapper: draft.wrapper,
        execution_target: executionTarget,
        use_worktrees: draft.executionTarget.kind === "coder" ? false : draft.useWorktrees,
        acceptance_criteria: draft.acceptanceCriteria,
        model_servers: draft.modelServers,
        hosted_collections: draft.hostedCollectionIds.map((id) => {
          const collection = collections.find((item) => item.id === id);
          if (!collection) {
            throw new Error(`Collection ${id} is no longer available`);
          }
          return { id, checksum: collection.checksum };
        }),
      });
      void navigate("/", { replace: true });
    } catch (cause) {
      setError(
        cause instanceof ApiError
          ? cause.message
          : cause instanceof Error
            ? cause.message
            : "Initialization failed",
      );
    } finally {
      setBusy(false);
    }
  }

  if (!status || !current || !Step) {
    return <main className={styles.loading}>{error ?? "Loading workspace setup…"}</main>;
  }

  return (
    <main className={styles.page}>
      <aside className={styles.sidebar}>
        <div className={styles.brand}>Operator</div>
        <ol>
          {walk.map((slug, index) => {
            const metadata = steps.find((step) => step.slug === slug);
            return (
              <li
                key={slug}
                className={
                  index === currentIndex
                    ? styles.active
                    : index < currentIndex
                      ? styles.complete
                      : ""
                }
              >
                <span>{index + 1}</span>
                {metadata?.name ?? slug}
              </li>
            );
          })}
        </ol>
      </aside>
      <section className={styles.content}>
        <header>
          <p>
            Step {currentIndex + 1} of {walk.length}
          </p>
          <h1>{current.name}</h1>
          <p>{current.description}</p>
        </header>
        {error && <div className={styles.error}>{error}</div>}
        <div className={styles.body}>
          <Step
            api={api}
            status={status}
            integrations={integrations}
            collections={collections}
            draft={draft}
            setDraft={setDraft}
            exports={exports}
            addExport={addExport}
          />
        </div>
        <footer>
          <button
            type="button"
            disabled={currentIndex === 0 || busy}
            onClick={() => {
              setError(null);
              setCurrentSlug(walk[currentIndex - 1]);
            }}
          >
            Back
          </button>
          {currentIndex === walk.length - 1 ? (
            <button type="button" className={styles.primary} disabled={busy} onClick={initialize}>
              {busy ? "Initializing…" : "Initialize workspace"}
            </button>
          ) : (
            <button type="button" className={styles.primary} onClick={next}>
              Continue
            </button>
          )}
        </footer>
      </section>
    </main>
  );
}
