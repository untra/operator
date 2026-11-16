import { isValid, maxLength, pattern, required, type Validator } from "@operator/webcomponents";
import { httpUrlRules, requiredRules } from "../../field-rules";
import type { KanbanProviderKind } from "@operator/bindings/KanbanProviderKind";
import type { SetupStep } from "@operator/bindings/SetupStep";
import type { WizardDraft } from "./types";

export const configurationNameRules: readonly Validator<string>[] = [
  required("Name the configuration"),
  maxLength(64),
  pattern(/^[a-z0-9_-]+$/, "Use lowercase letters, numbers, hyphens, or underscores"),
];

export const coderTargetNameRules: readonly Validator<string>[] = [
  required("Target name is required"),
];

export const coderTemplateRules: readonly Validator<string>[] = [
  required("Coder template is required"),
];

export const repositoryRules: readonly Validator<string>[] = [
  required(),
  pattern(/^[^/\s]+\/[^/\s]+$/, "Use owner/repository"),
];

export const emailRules: readonly Validator<string>[] = [
  required(),
  pattern(/^[^@\s]+@[^@\s]+\.[^@\s]+$/, "Enter an email address"),
];

export type KanbanCredentialFields = {
  domain: string;
  email: string;
  instance: string;
  repository: string;
  rootPath: string;
  token: string;
};

type FieldRules = ReadonlyArray<[keyof KanbanCredentialFields, readonly Validator<string>[]]>;

const KANBAN_FIELD_RULES: Record<KanbanProviderKind, FieldRules> = {
  gitea: [
    ["instance", requiredRules],
    ["domain", httpUrlRules],
    ["repository", repositoryRules],
    ["token", requiredRules],
  ],
  jira: [
    ["domain", requiredRules],
    ["email", emailRules],
    ["token", requiredRules],
  ],
  linear: [["token", requiredRules]],
  github: [["token", requiredRules]],
  openspec: [
    ["instance", requiredRules],
    ["rootPath", requiredRules],
  ],
};

/** Whether `fields` hold every credential `provider` needs before validation. */
export function kanbanCredentialsValid(
  provider: KanbanProviderKind,
  fields: KanbanCredentialFields,
): boolean {
  return KANBAN_FIELD_RULES[provider].every(([key, rules]) => isValid(fields[key], rules));
}

/** Rules for the parameter name at `index`, unique among its siblings. */
export function coderParameterNameRules(
  names: readonly string[],
  index: number,
): readonly Validator<string>[] {
  const others = new Set(names.filter((_, i) => i !== index).map((name) => name.trim()));
  return [
    required("Parameter name is required"),
    (value) => (others.has(value.trim()) ? "Parameter names must be unique" : null),
  ];
}

function executionTargetValid(draft: WizardDraft): boolean {
  const target = draft.executionTarget;
  if (target.kind !== "coder") {
    return true;
  }
  const names = draft.coderParameters.map((parameter) => parameter.name);
  return (
    isValid(target.name, coderTargetNameRules) &&
    isValid(target.template, coderTemplateRules) &&
    names.every((name, index) => isValid(name, coderParameterNameRules(names, index)))
  );
}

/** Whether the draft satisfies every field rule on `slug`, gating Continue. */
export function stepValid(slug: SetupStep, draft: WizardDraft): boolean {
  switch (slug) {
    case "welcome":
      return isValid(draft.configurationName, configurationNameRules);
    case "hosted-collections":
      return draft.hostedCollectionIds.length > 0;
    case "execution-target":
      return executionTargetValid(draft);
    default:
      return true;
  }
}
