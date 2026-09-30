import type { ChangeEvent, FormEvent, ReactNode } from "react";

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
  onSubmit?: (event: FormEvent<HTMLFormElement>) => void;
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
