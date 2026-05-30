// Renders a single codicon glyph. `name` is a codicon name without the
// `codicon-` prefix (e.g. "git-branch"); see ui/src/concepts.ts for the
// canonical concept→icon mapping. The font is loaded once in main.tsx via
// `@vscode/codicons/dist/codicon.css`. Icons inherit `currentColor`, so they
// recolor with surrounding text in both light and dark themes.

interface ConceptIconProps {
  name: string;
  className?: string;
}

export function ConceptIcon({ name, className }: ConceptIconProps) {
  const cls = className ? `codicon codicon-${name} ${className}` : `codicon codicon-${name}`;
  return <i className={cls} aria-hidden="true" />;
}
