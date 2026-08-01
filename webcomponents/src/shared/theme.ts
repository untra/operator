/**
 * Theme plumbing shared by every surface that embeds these components.
 *
 * Both the SPA and the Jekyll docs site stamp `data-theme` on `<html>` and load
 * the same brand tokens (`docs/assets/css/tokens.css`), so components read
 * colors from CSS custom properties rather than carrying their own palette.
 */

import { useEffect, useMemo, useState } from 'react';

export type Theme = 'light' | 'dark';

/** Brand tokens used as per-phase accents, in application order. */
const PHASE_COLOR_TOKENS = [
  '--color-cornflower',
  '--color-teal',
  '--color-salmon',
  '--color-coral',
  '--color-green-l2',
];

function readTheme(): Theme {
  if (typeof document === 'undefined') return 'light';
  return document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light';
}

/** Track the document's `data-theme` so embedded graphs follow the host toggle. */
export function useDocumentTheme(): Theme {
  const [theme, setTheme] = useState<Theme>(readTheme);
  useEffect(() => {
    const observer = new MutationObserver(() => setTheme(readTheme()));
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['data-theme'],
    });
    return () => observer.disconnect();
  }, []);
  return theme;
}

/** Per-phase accent colors resolved from the brand tokens. */
export function usePhaseColors(): string[] {
  const theme = useDocumentTheme();
  return useMemo(() => {
    if (typeof document === 'undefined') return [];
    const css = getComputedStyle(document.documentElement);
    return PHASE_COLOR_TOKENS.map((token) => css.getPropertyValue(token).trim()).filter(Boolean);
    // Re-resolve when the theme flips: the tokens themselves change value.
  }, [theme]);
}
