import { controlClass, FieldShell, useFieldState, type FieldBaseProps } from "./Field";

export interface SelectOption<T extends string> {
  value: T;
  label: string;
  disabled?: boolean;
}

export interface SelectInputProps<T extends string> extends FieldBaseProps<string> {
  value: T | "";
  onChange: (value: T) => void;
  options: readonly SelectOption<T>[];
  /** Label of an empty leading option; omit when a value is always selected. */
  placeholder?: string;
}

export function SelectInput<T extends string>({
  label,
  hint,
  value,
  onChange,
  options,
  placeholder,
  ...props
}: SelectInputProps<T>) {
  const { onBlur, message } = useFieldState<string>(value, props);
  const byValue = new Map(options.map((option) => [option.value as string, option.value]));
  return (
    <FieldShell label={label} hint={hint} message={message} className={props.className}>
      {(aria) => (
        <select
          {...aria}
          className={controlClass(message)}
          name={props.name}
          value={value}
          disabled={props.disabled}
          onBlur={onBlur}
          onChange={(event) => {
            const next = byValue.get(event.target.value);
            if (next !== undefined) {
              onChange(next);
            }
          }}
        >
          {placeholder !== undefined && <option value="">{placeholder}</option>}
          {options.map((option) => (
            <option key={option.value} value={option.value} disabled={option.disabled}>
              {option.label}
            </option>
          ))}
        </select>
      )}
    </FieldShell>
  );
}
