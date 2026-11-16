/** A rule returns an error message, or null when the value is valid. */
export type Validator<T> = (value: T) => string | null;

/** Format rules pass empty values so only `required` reports a missing value. */
const blank = (value: string) => value.trim() === "";

export const required =
  (message = "Required"): Validator<string> =>
  (value) =>
    blank(value) ? message : null;

export const pattern =
  (re: RegExp, message: string): Validator<string> =>
  (value) =>
    blank(value) || re.test(value) ? null : message;

export const maxLength =
  (max: number, message = `At most ${max} characters`): Validator<string> =>
  (value) =>
    value.length > max ? message : null;

export const minLength =
  (min: number, message = `At least ${min} characters`): Validator<string> =>
  (value) =>
    blank(value) || value.length >= min ? null : message;

export const url =
  (message = "Enter an http(s) URL"): Validator<string> =>
  (value) => {
    if (blank(value)) {
      return null;
    }
    if (!URL.canParse(value)) {
      return message;
    }
    const { protocol } = new URL(value);
    return protocol === "http:" || protocol === "https:" ? null : message;
  };

export const oneOf =
  (values: readonly string[], message = "Choose an option"): Validator<string> =>
  (value) =>
    values.includes(value) ? null : message;

export function validate<T>(value: T, rules: readonly Validator<T>[]): string | null {
  for (const rule of rules) {
    const message = rule(value);
    if (message !== null) {
      return message;
    }
  }
  return null;
}

export function isValid<T>(value: T, rules: readonly Validator<T>[]): boolean {
  return validate(value, rules) === null;
}
