import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import type { SetupStatusResponse, SetupStepResponse } from "../../api-client";
import { ApiError, OperatorApi } from "../../api-client";
import { useHost } from "../../host";
import { STEP_COMPONENTS, stepRows, visibleSteps } from "./steps";
import type { SetupStep } from "@operator/bindings/SetupStep";
import type { StepRow, WizardDraft } from "./types";
import { useProfiles } from "../../profiles-context";
import styles from "./OnboardingPage.module.css";

const WIZARD_DRAFT_VERSION = 1;
const WIZARD_DRAFT_PREFIX = "operator.onboarding-draft";

type SavedDraft = {
  version?: number;
  draft?: Partial<WizardDraft>;
  currentSlug?: SetupStep;
};

function emptyDraft(configurationName: string, acceptanceCriteria = ""): WizardDraft {
  return {
    configurationName,
    executionMode: "local",
    premium: false,
    preset: "devops_kanban",
    taskFields: ["priority", "points", "user_story"],
    wrapper: "tmux",
    executionTarget: { kind: "local" },
    coderParameters: [],
    useWorktrees: false,
    acceptanceCriteria,
    modelServers: [],
    hostedCollectionIds: [],
  };
}

function draftStorageKey(profileId: string): string {
  return `${WIZARD_DRAFT_PREFIX}.${profileId}`;
}

/**
 * The one trainstop for the whole of setup. Numbered rows are the steps this
 * draft will actually walk; a `number: null` row is an optional step the draft
 * currently skips, shown dimmed so the numbering never shifts under the user.
 */
function SetupProgress({
  rows,
  steps,
  currentSlug,
}: {
  rows: StepRow[];
  steps: SetupStepResponse[];
  currentSlug: SetupStep;
}) {
  const currentRow = rows.findIndex((row) => row.slug === currentSlug);
  return (
    <aside className={styles.sidebar}>
      <div className={styles.brand}>Operator</div>
      <ol>
        {rows.map((row, index) => (
          <li
            key={row.slug}
            className={
              row.number === null
                ? styles.pending
                : index === currentRow
                  ? styles.active
                  : index < currentRow
                    ? styles.complete
                    : ""
            }
          >
            <span>{row.number ?? "·"}</span>
            <div>
              {steps.find((step) => step.slug === row.slug)?.name ?? row.slug}
              {row.hint && <small>{row.hint}</small>}
            </div>
          </li>
        ))}
      </ol>
    </aside>
  );
}

export function OnboardingPage() {
  const host = useHost();
  const { selected, create, refresh } = useProfiles();
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const [api] = useState(() => new OperatorApi(host));
  // Mount-scoped, not a live read of the query string: the step sync below
  // strips `?new=1` as soon as the catalog loads, and a failed create must
  // still leave the user creating rather than renaming the selected profile.
  const [creating] = useState(() => !selected || params.get("new") === "1");
  const [initialStepParam] = useState(() => params.get("step"));
  const [status, setStatus] = useState<SetupStatusResponse | null>(null);
  const [steps, setSteps] = useState<Awaited<ReturnType<typeof api.setupSteps>>>([]);
  const [integrations, setIntegrations] = useState<Awaited<ReturnType<typeof api.integrations>>>(
    [],
  );
  const [collections, setCollections] = useState<Awaited<ReturnType<typeof api.setupCollections>>>(
    [],
  );
  const [draft, setDraft] = useState<WizardDraft>(() => emptyDraft(""));
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
        // While creating, the status describes whichever configuration the
        // server defaulted to, so its `initialized` flag says nothing about the
        // one being created.
        if (nextStatus.initialized && !creating) {
          void navigate("/", { replace: true });
          return undefined;
        }
        setStatus(nextStatus);
        setSteps(nextSteps);
        setIntegrations(nextIntegrations);
        setCollections(nextCollections);
        const initial = emptyDraft(
          creating ? "" : (selected?.name ?? ""),
          nextStatus.default_acceptance_criteria,
        );
        const known = new Set(nextSteps.map((step) => step.slug));
        let saved: SavedDraft | null = null;
        if (selected && !creating) {
          try {
            saved = JSON.parse(
              sessionStorage.getItem(draftStorageKey(selected.id)) ?? "null",
            ) as SavedDraft | null;
          } catch {
            sessionStorage.removeItem(draftStorageKey(selected.id));
          }
        }
        setDraft(
          saved?.version === WIZARD_DRAFT_VERSION && saved.draft
            ? { ...initial, ...saved.draft }
            : initial,
        );
        const fromUrl = initialStepParam as SetupStep | null;
        setCurrentSlug(
          fromUrl && known.has(fromUrl)
            ? fromUrl
            : saved?.currentSlug && known.has(saved.currentSlug)
              ? saved.currentSlug
              : "welcome",
        );
        return undefined;
      })
      .catch(
        (cause: unknown) =>
          active && setError(cause instanceof Error ? cause.message : "Could not load setup"),
      );
    return () => {
      active = false;
    };
  }, [api, creating, initialStepParam, navigate, selected]);

  // The address bar names the step, so a refresh resumes where the user was and
  // `?new=1` falls away once the wizard has taken over.
  useEffect(() => {
    if (status) {
      void navigate(`/onboarding?step=${currentSlug}`, { replace: true });
    }
  }, [currentSlug, navigate, status]);

  useEffect(() => {
    if (!selected || !status || status.initialized || creating) {
      return;
    }
    sessionStorage.setItem(
      draftStorageKey(selected.id),
      JSON.stringify({ version: WIZARD_DRAFT_VERSION, draft, currentSlug }),
    );
  }, [creating, currentSlug, draft, selected, status]);

  const slugs = useMemo(() => steps.map((step) => step.slug), [steps]);
  const walk = useMemo(() => visibleSteps(slugs, draft), [draft, slugs]);
  const rows = useMemo(() => stepRows(slugs, draft), [draft, slugs]);
  if (walk.length > 0 && !walk.includes(currentSlug)) {
    setCurrentSlug(walk[0]);
  }
  const currentIndex = Math.max(0, walk.indexOf(currentSlug));

  const current = steps.find((step) => step.slug === walk[currentIndex]);
  const Step = current ? STEP_COMPONENTS[current.slug] : null;

  const addExport = useCallback(
    (value: string) => {
      setExports((currentExports) =>
        currentExports.includes(value) ? currentExports : [...currentExports, value],
      );
    },
    [setExports],
  );

  /**
   * Create the configuration the welcome step just named, hand the draft over
   * under its id, and point the URL at the next step. `create` selects the new
   * profile, which remounts this wizard through `ProfileScope`; both writes
   * have to land before that commit for the handoff to survive.
   */
  async function createConfiguration(nextSlug: SetupStep): Promise<boolean> {
    setBusy(true);
    try {
      const profile = await create(draft.configurationName);
      sessionStorage.setItem(
        draftStorageKey(profile.id),
        JSON.stringify({
          version: WIZARD_DRAFT_VERSION,
          draft: { ...draft, configurationName: profile.name },
          currentSlug: nextSlug,
        }),
      );
      void navigate(`/onboarding?step=${nextSlug}`, { replace: true });
      return true;
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not create configuration");
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function next() {
    if (!current) {
      return;
    }
    const nextSlug = walk[Math.min(currentIndex + 1, walk.length - 1)];
    if (current.slug === "welcome") {
      if (!/^[a-z0-9_-]{1,64}$/.test(draft.configurationName)) {
        setError("Use 1-64 lowercase letters, numbers, hyphens, or underscores.");
        return;
      }
      if (creating) {
        setError(null);
        await createConfiguration(nextSlug);
        return;
      }
      if (selected && selected.name !== draft.configurationName) {
        setBusy(true);
        try {
          await api.renameProfile(selected.id, draft.configurationName);
          await refresh();
        } catch (cause) {
          setError(cause instanceof Error ? cause.message : "Could not rename configuration");
          return;
        } finally {
          setBusy(false);
        }
      }
    }
    if (current.slug === "execution-mode" && draft.executionMode === "remote") {
      try {
        const license = await api.license();
        if (!license.premium) {
          setError("A valid Premium license is required for remote targets.");
          return;
        }
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : "Could not verify license");
        return;
      }
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
    setCurrentSlug(nextSlug);
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
      if (selected) {
        sessionStorage.removeItem(draftStorageKey(selected.id));
      }
      await refresh();
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

  const onNext = () => {
    void next();
  };

  if (!status || !current || !Step) {
    return <main className={styles.loading}>{error ?? "Loading workspace setup…"}</main>;
  }

  return (
    <main className={styles.page}>
      <SetupProgress rows={rows} steps={steps} currentSlug={currentSlug} />
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
            creating={creating}
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
            <button type="button" className={styles.primary} disabled={busy} onClick={onNext}>
              Continue
            </button>
          )}
        </footer>
      </section>
    </main>
  );
}
