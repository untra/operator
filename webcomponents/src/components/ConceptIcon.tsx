export interface ConceptIconProps {
  name: string;
  className?: string;
}

export function ConceptIcon({ name, className }: ConceptIconProps) {
  const classes = className ? `codicon codicon-${name} ${className}` : `codicon codicon-${name}`;
  return <i className={classes} aria-hidden="true" />;
}
