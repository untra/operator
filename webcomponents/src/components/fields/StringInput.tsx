import { controlClass, FieldShell, useFieldState, type FieldBaseProps } from "./Field";

export interface StringInputProps extends FieldBaseProps<string> {
  value: string;
  onChange: (value: string) => void;
  type?: "text" | "password" | "url" | "email" | "search";
  placeholder?: string;
  autoComplete?: string;
  maxLength?: number;
}

export function StringInput({
  label,
  hint,
  value,
  onChange,
  type = "text",
  ...props
}: StringInputProps) {
  const { onBlur, message } = useFieldState(value, props);
  return (
    <FieldShell label={label} hint={hint} message={message} className={props.className}>
      {(aria) => (
        <input
          {...aria}
          className={controlClass(message)}
          type={type}
          name={props.name}
          value={value}
          placeholder={props.placeholder}
          autoComplete={props.autoComplete}
          maxLength={props.maxLength}
          disabled={props.disabled}
          onBlur={onBlur}
          onChange={(event) => onChange(event.target.value)}
        />
      )}
    </FieldShell>
  );
}
