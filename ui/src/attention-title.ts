import { useEffect } from "react";
import type { ActiveAgentsResponse } from "@operator/bindings/ActiveAgentsResponse";
import { AGENT_POLL_MS, useApiQuery } from "./api";
import { activeAgentsQuery } from "./api/definitions";

export function attentionCount(res: ActiveAgentsResponse | null | undefined): number {
  return res?.agents.filter((a) => a.attention).length ?? 0;
}

export function titleWithCount(title: string, count: number): string {
  const base = title.replace(/^\(\d+\) /, "");
  return count > 0 ? `(${count}) ${base}` : base;
}

/** Prefixes the document title with the number of agents needing attention. */
export function useAttentionTitle(): void {
  const agents = useApiQuery(activeAgentsQuery(), { pollIntervalMs: AGENT_POLL_MS });
  const count = attentionCount(agents.data);
  useEffect(() => {
    document.title = titleWithCount(document.title, count);
  }, [count]);
}
