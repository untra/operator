// A generic, context-driven right-hand detail sidepanel. Any view can open it
// with arbitrary content (a React node) and an optional title — clicking a
// kanban ticket, for example, opens it with that ticket's detail + launch form.
// Mounted once by Layout alongside SectionsProvider, mirroring that pattern; the
// <aside> that renders this state lives in Layout next to <main>.

import { createContext, useCallback, useContext, useMemo, useState } from 'react';
import type { ReactNode } from 'react';

interface RightPanelState {
  /** The node currently shown in the panel, or null when the panel is closed. */
  content: ReactNode | null;
  /** Optional heading shown in the panel's header row. */
  title: string | null;
  /** Open the panel with `content` and an optional `title`. */
  open: (content: ReactNode, title?: string) => void;
  /** Close the panel and clear its content. */
  close: () => void;
}

const RightPanelContext = createContext<RightPanelState>({
  content: null,
  title: null,
  open: () => {},
  close: () => {},
});

export function RightPanelProvider({ children }: { children: ReactNode }) {
  const [content, setContent] = useState<ReactNode | null>(null);
  const [title, setTitle] = useState<string | null>(null);

  const open = useCallback((node: ReactNode, t?: string) => {
    setContent(node);
    setTitle(t ?? null);
  }, []);
  const close = useCallback(() => {
    setContent(null);
    setTitle(null);
  }, []);

  const value = useMemo(
    () => ({ content, title, open, close }),
    [content, title, open, close],
  );

  return <RightPanelContext.Provider value={value}>{children}</RightPanelContext.Provider>;
}

export function useRightPanel(): RightPanelState {
  return useContext(RightPanelContext);
}
