// A single status section, rendered identically on the unified Status overview
// and on each per-section page. Lifted out of StatusPage so both consume one
// component. Locked sections (prerequisites not met) render collapsed with the
// missing prerequisites shown as links to their pages - the "next steps."

import { useMemo } from "react";
import { Link } from "react-router-dom";
import { SectionCard as SharedSectionCard } from "@operator/webcomponents";
import type { SectionDto } from "../api-client";
import { useHost } from "../host";
import { connectionUrl } from "../connections";
import { CONCEPTS } from "../concepts";

const brandIconSrc = (name: string) => `/icons/${name}.svg`;

function renderPrerequisite(id: string) {
  const concept = CONCEPTS[id];
  return concept ? <Link to={concept.route}>{concept.label}</Link> : id;
}

export function SectionCard({ section }: { section: SectionDto }) {
  const host = useHost();
  const resolved = useMemo(
    () => ({
      ...section,
      children: section.children.map((row) => ({
        ...row,
        actions: row.actions.map((action) => ({
          ...action,
          url: action.url.startsWith("/") ? connectionUrl(host.baseUrl(), action.url) : action.url,
        })),
      })),
    }),
    [section, host],
  );
  return (
    <SharedSectionCard
      section={resolved}
      brandIconSrc={brandIconSrc}
      renderPrerequisite={renderPrerequisite}
    />
  );
}
