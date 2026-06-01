import { useEffect, useMemo, useState } from 'react';
import { compile } from '@untra/naiveworkflow-compiler';
import { WorkflowGraph } from '@untra/naiveworkflow-react';
import '@xyflow/react/dist/style.css';
import '@untra/naiveworkflow-react/styles.css';
import styles from './WorkflowGraphView.module.css';

/**
 * Renders a Claude dynamic-workflow `.js` source as an interactive graph.
 *
 * The source is compiled to an IR *in the browser* by `@untra/naiveworkflow-
 * compiler` (acorn-based; it parses, never executes) and drawn with
 * `@untra/naiveworkflow-react` (React Flow + dagre). This is display-only:
 * nothing flows back into operator's domain model.
 */
export function WorkflowGraphView({ contents }: { contents: string }) {
  const theme = useDocumentTheme();
  const phaseColors = usePhaseColors();

  const { graph, meta, diagnostics } = useMemo(() => compile(contents), [contents]);

  return (
    <div className={styles.wrap}>
      {diagnostics.length > 0 && (
        <div className={styles.diagnostics}>
          <strong>{diagnostics.length} note(s)</strong> while compiling — some constructs may not be
          drawn:
          <ul>
            {diagnostics.map((d, i) => (
              <li key={i}>{d.message}</li>
            ))}
          </ul>
        </div>
      )}
      <div className={styles.canvas}>
        <WorkflowGraph
          graph={graph}
          meta={meta ?? undefined}
          theme={theme}
          phaseColors={phaseColors}
          fitView
        />
      </div>
    </div>
  );
}

/** Track the document's `data-theme` so the graph follows the operator theme toggle. */
function useDocumentTheme(): 'light' | 'dark' {
  const read = (): 'light' | 'dark' =>
    document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light';
  const [theme, setTheme] = useState<'light' | 'dark'>(read);
  useEffect(() => {
    const observer = new MutationObserver(() => setTheme(read()));
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
    return () => observer.disconnect();
  }, []);
  return theme;
}

/** Per-phase accent colors drawn from the operator brand palette tokens. */
function usePhaseColors(): string[] {
  return useMemo(() => {
    const tokens = [
      '--color-cornflower',
      '--color-teal',
      '--color-salmon',
      '--color-coral',
      '--color-green-l2',
    ];
    const css = getComputedStyle(document.documentElement);
    return tokens.map((t) => css.getPropertyValue(t).trim()).filter(Boolean);
  }, []);
}
