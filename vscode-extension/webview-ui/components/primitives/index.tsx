import React, { type CSSProperties, type ReactNode } from 'react';

/*
 * MUI-free primitives styled by styles/webview.css. Colors come from the
 * host editor's `--vscode-*` variables; Operator brand appears only as
 * accents. Event handlers receive native React events so call sites read
 * like plain HTML.
 */

interface CommonProps {
  className?: string;
  style?: CSSProperties;
}

function cx(...parts: Array<string | false | undefined>): string {
  return parts.filter(Boolean).join(' ');
}

export interface ButtonProps extends CommonProps {
  variant?: 'contained' | 'outlined' | 'text';
  accent?: 'sage';
  size?: 'small' | 'medium';
  disabled?: boolean;
  title?: string;
  onClick?: React.MouseEventHandler<HTMLButtonElement>;
  children?: ReactNode;
}

export function Button({
  variant = 'text',
  accent,
  size = 'medium',
  disabled,
  title,
  onClick,
  children,
  className,
  style,
}: ButtonProps) {
  return (
    <button
      type="button"
      className={cx('op-btn', className)}
      data-variant={variant}
      data-accent={accent}
      data-size={size}
      disabled={disabled}
      title={title}
      onClick={onClick}
      style={style}
    >
      {children}
    </button>
  );
}

export interface IconButtonProps extends CommonProps {
  disabled?: boolean;
  title?: string;
  'aria-label'?: string;
  onClick?: React.MouseEventHandler<HTMLButtonElement>;
  children?: ReactNode;
}

export function IconButton({ disabled, title, onClick, children, className, style, ...rest }: IconButtonProps) {
  return (
    <button
      type="button"
      className={cx('op-icon-btn', className)}
      disabled={disabled}
      title={title}
      aria-label={rest['aria-label']}
      onClick={onClick}
      style={style}
    >
      {children}
    </button>
  );
}

export interface TextInputProps extends CommonProps {
  label?: string;
  value: string;
  onChange?: React.ChangeEventHandler<HTMLInputElement>;
  onBlur?: React.FocusEventHandler<HTMLInputElement>;
  onKeyDown?: React.KeyboardEventHandler<HTMLInputElement>;
  placeholder?: string;
  type?: string;
  helperText?: string;
  error?: boolean;
  disabled?: boolean;
}

export function TextInput({
  label,
  value,
  onChange,
  onBlur,
  onKeyDown,
  placeholder,
  type = 'text',
  helperText,
  error,
  disabled,
  className,
  style,
}: TextInputProps) {
  return (
    <label className={cx('op-field', className)} data-error={error ? 'true' : undefined} style={style}>
      {label && <span className="op-field-label">{label}</span>}
      <input
        className="op-field-input"
        type={type}
        value={value}
        onChange={onChange}
        onBlur={onBlur}
        onKeyDown={onKeyDown}
        placeholder={placeholder}
        disabled={disabled}
      />
      {helperText && <span className="op-field-helper">{helperText}</span>}
    </label>
  );
}

export interface SelectInputProps extends CommonProps {
  label?: string;
  value: string;
  onChange?: React.ChangeEventHandler<HTMLSelectElement>;
  helperText?: string;
  error?: boolean;
  disabled?: boolean;
  children?: ReactNode;
}

export function SelectInput({
  label,
  value,
  onChange,
  helperText,
  error,
  disabled,
  children,
  className,
  style,
}: SelectInputProps) {
  return (
    <label className={cx('op-field', className)} data-error={error ? 'true' : undefined} style={style}>
      {label && <span className="op-field-label">{label}</span>}
      <select className="op-field-select" value={value} onChange={onChange} disabled={disabled}>
        {children}
      </select>
      {helperText && <span className="op-field-helper">{helperText}</span>}
    </label>
  );
}

export interface ToggleProps extends CommonProps {
  checked: boolean;
  onChange?: React.ChangeEventHandler<HTMLInputElement>;
  label?: ReactNode;
  disabled?: boolean;
}

export function Toggle({ checked, onChange, label, disabled, className, style }: ToggleProps) {
  return (
    <label className={cx('op-toggle', className)} data-disabled={disabled ? 'true' : undefined} style={style}>
      <input type="checkbox" checked={checked} onChange={onChange} disabled={disabled} />
      <span className="op-toggle-track" />
      {label && <span className="op-body2">{label}</span>}
    </label>
  );
}

export interface ChipProps extends CommonProps {
  label: ReactNode;
  variant?: 'filled' | 'outlined';
  color?: 'default' | 'success' | 'warning' | 'error' | 'brand';
}

export function Chip({ label, variant = 'filled', color = 'default', className, style }: ChipProps) {
  return (
    <span
      className={cx('op-chip', className)}
      data-variant={variant === 'outlined' ? 'outlined' : undefined}
      data-color={color !== 'default' ? color : undefined}
      style={style}
    >
      {label}
    </span>
  );
}

export interface AlertProps extends CommonProps {
  severity: 'error' | 'warning' | 'info' | 'success';
  children?: ReactNode;
}

export function Alert({ severity, children, className, style }: AlertProps) {
  return (
    <div className={cx('op-alert', className)} data-severity={severity} role="alert" style={style}>
      <div>{children}</div>
    </div>
  );
}

export interface SpinnerProps extends CommonProps {
  /** Diameter in px (defaults to 24, MUI's default is 40) */
  size?: number;
}

export function Spinner({ size, className, style }: SpinnerProps) {
  const dim = size ? { width: size, height: size } : undefined;
  return <span className={cx('op-spinner', className)} style={{ ...dim, ...style }} />;
}

export interface CardProps extends CommonProps {
  accent?: 'terracotta';
  children?: ReactNode;
}

export function Card({ accent, children, className, style }: CardProps) {
  return (
    <div className={cx('op-card', className)} data-accent={accent} style={style}>
      {children}
    </div>
  );
}

export function CardContent({ children, className, style }: CommonProps & { children?: ReactNode }) {
  return (
    <div className={cx('op-card-content', className)} style={style}>
      {children}
    </div>
  );
}
