// Shared status-sections store. `GET /api/v1/sections` is the web-UI projection
// of the same SectionId model the TUI and VS Code extension render. Polling it in
// one place (mounted once by Layout) keeps the sidebar and every section page in
// sync off a single 3s timer instead of N drifting ones.

import { createContext, useContext, useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import { OperatorApi } from './api-client';
import type { SectionDto } from './api-client';
import { useHost } from './host';

const POLL_INTERVAL_MS = 3000;

interface SectionsState {
  sections: SectionDto[] | null;
  error: string | null;
}

const SectionsContext = createContext<SectionsState>({ sections: null, error: null });

export function SectionsProvider({ children }: { children: ReactNode }) {
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [sections, setSections] = useState<SectionDto[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const refresh = () => {
      api
        .sections()
        .then((s) => {
          if (cancelled) return;
          setSections(s);
          setError(null);
        })
        .catch((e) => {
          if (!cancelled) setError(e.message);
        });
    };
    refresh();
    const timer = setInterval(refresh, POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [api]);

  return <SectionsContext.Provider value={{ sections, error }}>{children}</SectionsContext.Provider>;
}

export function useSections(): SectionsState {
  return useContext(SectionsContext);
}

/** Convenience selector: the section whose id matches `id`, if loaded. */
export function useSection(id: string): SectionDto | undefined {
  const { sections } = useSections();
  return sections?.find((s) => s.id === id);
}
