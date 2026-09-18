import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import type { SetupStatusResponse } from "../../api-client";
import { ApiError, OperatorApi } from "../../api-client";
import { useHost } from "../../host";
import { STEP_COMPONENTS, visibleSteps } from "./steps";
import type { SetupStep } from "@operator/bindings/SetupStep";
import type { WizardDraft } from "./types";
import { useProfiles } from "../../profiles-context";
import styles from "./OnboardingPage.module.css";

const WIZARD_DRAFT_VERSION = 1;
const WIZARD_DRAFT_PREFIX = "operator.onboarding-draft";

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

export function OnboardingPage() {
  const { selected } = useProfiles();
  const [params] = useSearchParams();
  return !selected || params.get("new") === "1" ? <NewConfiguration /> : <OnboardingWizard />;
}

function NewConfiguration() {
  const { create, selected } = useProfiles();
  const navigate = useNavigate();
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const onCancel = () => {
    void navigate(selected?.initialized ? "/" : "/onboarding");
  };
  return (
    <main className={styles.page}>
      <aside className={styles.sidebar}>
        <div className={styles.brand}>Operator</div>
        <p>1. Name configuration</p>
        <p>2. Operator Premium</p>
        <p>3. Execution mode</p>
      </aside>
      <section className={styles.content}>
        <header>
          <p>Step 1</p>
          <h1>Welcome to Operator</h1>
          <p>Name this configuration.</p>
        </header>
        <form
          className={styles.body}
          onSubmit={(event) => {
            event.preventDefault();
            setBusy(true);
            setError(null);
            void create(name)
              .then(() => navigate("/onboarding?step=license", { replace: true }))
              .catch((cause: unknown) => {
                setError(cause instanceof Error ? cause.message : "Could not create configuration");
                setBusy(false);
              });
          }}
        >
          <label className={styles.form}>
            Configuration name
            <input
              required
              pattern="[a-z0-9_-]+"
              maxLength={64}
              value={name}
              disabled={busy}
              onChange={(event) => setName(event.target.value)}
            />
            <span>Use lowercase letters, numbers, hyphens, and underscores.</span>
          </label>
          {error && <p role="alert">{error}</p>}
          <button type="submit" disabled={busy}>
            {busy ? "Creating…" : "Continue"}
          </button>
          {selected && (
            <button type="button" disabled={busy} onClick={onCancel}>
              Cancel
            </button>
          )}
        </form>
      </section>
    </main>
  );
}

function OnboardingWizard() {
  const host = useHost();
  const { selected, refresh } = useProfiles();
  const [params] = useSearchParams();
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
  const [draft, setDraft] = useState<WizardDraft>(() => emptyDraft(selected?.name ?? ""));
  const [currentSlug, setCurrentSlug] = useState<SetupStep>(
    params.get("step") === "license" ? "license" : "welcome",
  );
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
        const initial = emptyDraft(selected?.name ?? "", nextStatus.default_acceptance_criteria);
        if (selected) {
          try {
            const saved = JSON.parse(
              sessionStorage.getItem(draftStorageKey(selected.id)) ?? "null",
            ) as {
              version?: number;
              draft?: Partial<WizardDraft>;
              currentSlug?: SetupStep;
            } | null;
            if (saved?.version === WIZARD_DRAFT_VERSION && saved.draft) {
              setDraft({ ...initial, ...saved.draft });
              if (saved.currentSlug) {
                setCurrentSlug(saved.currentSlug);
              }
            } else {
              setDraft(initial);
            }
          } catch {
            sessionStorage.removeItem(draftStorageKey(selected.id));
            setDraft(initial);
          }
        } else {
          setDraft(initial);
        }
        return undefined;
      })
      .catch(
        (cause: unknown) =>
          active && setError(cause instanceof Error ? cause.message : "Could not load setup"),
      );
    return () => {
      active = false;
    };
  }, [api, navigate, selected]);

  useEffect(() => {
    if (!selected || !status || status.initialized) {
      return;
    }
    sessionStorage.setItem(
      draftStorageKey(selected.id),
      JSON.stringify({ version: WIZARD_DRAFT_VERSION, draft, currentSlug }),
    );
  }, [currentSlug, draft, selected, status]);

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

  const addExport = useCallback(
    (value: string) => {
      setExports((currentExports) =>
        currentExports.includes(value) ? currentExports : [...currentExports, value],
      );
    },
    [setExports],
  );

  async function next() {
    if (!current) {
      return;
    }
    if (current.slug === "welcome") {
      if (!/^[a-z0-9_-]{1,64}$/.test(draft.configurationName)) {
        setError("Use 1-64 lowercase letters, numbers, hyphens, or underscores.");
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
            <button type="button" className={styles.primary} disabled={busy} onClick={onNext}>
              Continue
            </button>
          )}
        </footer>
      </section>
    </main>
  );
}
