import type { ChangeEvent, ReactNode, SubmitEvent } from "react";
import type { AsyncValue } from "@operator/webcomponents";

export function AuthCard({
  title,
  error,
  children,
  actions,
  onSubmit,
}: {
  title: string;
  error?: string | null;
  children: ReactNode;
  actions?: ReactNode;
  onSubmit?: (event: SubmitEvent<HTMLFormElement>) => void;
}) {
  return (
    <form onSubmit={onSubmit}>
      <h1>{title}</h1>
      {error && <p>{error}</p>}
      {children}
      {actions}
    </form>
  );
}

export function AuthField({
  label,
  value,
  onChange,
  type,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: string;
}) {
  return (
    <label>
      {label}
      <input
        aria-label={label}
        type={type}
        value={value}
        onChange={(event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value)}
      />
    </label>
  );
}

export function AuthSubmit({ children, disabled }: { children: ReactNode; disabled?: boolean }) {
  return (
    <button type="submit" disabled={disabled}>
      {children}
    </button>
  );
}

export function TicketCreateForm({
  onChange,
  onSubmit,
}: {
  onChange: (value: { issueType: string; project: string; summary: string }) => void;
  onSubmit: () => void;
}) {
  return (
    <>
      <button
        type="button"
        onClick={() => onChange({ issueType: "TASK", project: "operator", summary: "Test" })}
      >
        Fill ticket
      </button>
      <button type="button" onClick={onSubmit}>
        Submit ticket
      </button>
    </>
  );
}

export function TicketDetailView({
  launchControls,
  launchActions,
}: {
  launchControls?: ReactNode;
  launchActions?: ReactNode;
}) {
  return (
    <div>
      {launchControls}
      {launchActions}
    </div>
  );
}

export function LaunchForm({ onSubmit }: { onSubmit: () => void }) {
  return (
    <button type="button" onClick={onSubmit}>
      Launch ticket
    </button>
  );
}

export function PageHeader({ title }: { title: string }) {
  return <h1>{title}</h1>;
}

export function BrandIcon() {
  return <span />;
}

export function ConceptIcon() {
  return <span />;
}

export function ChoiceGroup({ children }: { children: ReactNode }) {
  return <div>{children}</div>;
}

export function Choice({
  children,
  value,
  onSelect,
}: {
  children: ReactNode;
  value: string;
  onSelect: (value: string) => void;
}) {
  return (
    <button type="button" onClick={() => onSelect(value)}>
      {children}
    </button>
  );
}

export function PremiumPaywall() {
  return <div>Premium required</div>;
}

export function PremiumGate({
  license,
  children,
}: {
  license: { premium: boolean } | null | undefined;
  children: ReactNode;
}) {
  if (!license) {
    return null;
  }
  return license.premium ? children : <PremiumPaywall />;
}

export function AsyncState<T>({
  value,
  children,
}: {
  value: AsyncValue<T>;
  children: (data: T) => ReactNode;
}) {
  if (value.status === "ready") {
    return children(value.data);
  }
  return <div data-status={value.status}>{value.message}</div>;
}

export function MarkdownView({ source }: { source: string }) {
  return <article data-testid="markdown">{source}</article>;
}

export function StepProgress({
  steps,
  current,
}: {
  steps: { name: string }[];
  current?: string | null;
}) {
  return (
    <ol>
      {steps.map((s) => (
        <li key={s.name} aria-current={s.name === current ? "step" : undefined}>
          {s.name}
        </li>
      ))}
    </ol>
  );
}

export function openTargets(path: string, remoteHost?: string | null) {
  return remoteHost
    ? [{ editor: "vscode", label: "VS Code", url: `vscode://ssh/${remoteHost}${path}` }]
    : [
        { editor: "vscode", label: "VS Code", url: `vscode://file${path}` },
        { editor: "jetbrains", label: "JetBrains", url: `idea://open?file=${path}` },
      ];
}

export function ticketFileTargets(ticketId: string, path: string) {
  return [
    { editor: "vscode", label: "VS Code", url: `vscode://open-ticket/${ticketId}` },
    { editor: "jetbrains", label: "JetBrains", url: `idea://open?file=${path}` },
  ];
}

export function sessionTargets(agent: {
  session_wrapper: string | null;
  attach_command: string | null;
}) {
  if (agent.session_wrapper === "cmux") {
    return [{ kind: "focus-api", label: "Focus cmux session" }];
  }
  return agent.attach_command
    ? [{ kind: "command", label: "Attach", command: agent.attach_command }]
    : [];
}
