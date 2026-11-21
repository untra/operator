import { isValid, pattern, required, url, type Validator } from "@operator/webcomponents";
import type { GitExecutionConfig } from "@operator/bindings/GitExecutionConfig";
import type { TargetDef } from "@operator/bindings/TargetDef";

export const requiredRules: readonly Validator<string>[] = [required()];

export const httpUrlRules: readonly Validator<string>[] = [required(), url()];

export const envVarRules: readonly Validator<string>[] = [
  required(),
  pattern(/^[A-Z_][A-Z0-9_]*$/, "Use an UPPER_SNAKE_CASE variable name"),
];

/** Whether a remote target draft has every field its kind needs to save. */
export function targetDraftValid(draft: TargetDef): boolean {
  if (!isValid(draft.name, requiredRules)) {
    return false;
  }
  switch (draft.kind) {
    case "ssh":
      return isValid(draft.ssh_alias, requiredRules) && isValid(draft.workdir, requiredRules);
    case "coder":
      return (
        isValid(draft.template, requiredRules) &&
        isValid(draft.url_env, envVarRules) &&
        isValid(draft.token_env, envVarRules)
      );
    default:
      return true;
  }
}

/** Whether a delegator's optional Git customization is complete. */
export function gitDraftValid(draft: GitExecutionConfig | null): boolean {
  if (!draft) {
    return true;
  }
  const { identity, credentials, settings } = draft;
  return (
    (!identity ||
      (isValid(identity.name, requiredRules) && isValid(identity.email, requiredRules))) &&
    (!credentials ||
      (isValid(credentials.repository_url, httpUrlRules) &&
        isValid(credentials.username, requiredRules) &&
        isValid(credentials.token_env, envVarRules))) &&
    settings.every((entry) => isValid(entry.key, requiredRules))
  );
}
