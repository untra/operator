// Shared status-sections store. `GET /api/v1/sections` is the web-UI projection
// of the same SectionId model the TUI and VS Code extension render. Polling it in
// one place (mounted once by Layout) keeps the sidebar and every section page in
// sync off a single 3s timer instead of N drifting ones.

import { createContext, useContext, useMemo } from "react";
import type { ReactNode } from "react";
import type { SectionDto } from "./api-client";
import { STATUS_POLL_MS, useApiQuery } from "./api";
import { sectionsQuery } from "./api/definitions";

interface SectionsState {
  sections: SectionDto[] | null;
  error: string | null;
}

const SectionsContext = createContext<SectionsState>({ sections: null, error: null });

export function SectionsProvider({ children }: { children: ReactNode }) {
  const { data, error } = useApiQuery(sectionsQuery(), { pollIntervalMs: STATUS_POLL_MS });
  const value = useMemo(() => ({ sections: data, error: error?.message ?? null }), [data, error]);

  return <SectionsContext.Provider value={value}>{children}</SectionsContext.Provider>;
}

export function useSections(): SectionsState {
  return useContext(SectionsContext);
}

/** Convenience selector: the section whose id matches `id`, if loaded. */
export function useSection(id: string): SectionDto | undefined {
  const { sections } = useSections();
  return sections?.find((s) => s.id === id);
}
