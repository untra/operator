// A single status section, rendered identically on the unified Status overview
// and on each per-section page. Lifted out of StatusPage so both consume one
// component. Locked sections (prerequisites not met) render collapsed with the
// missing prerequisites shown as links to their pages - the "next steps."

import { Link } from "react-router-dom";
import { SectionCard as SharedSectionCard } from "@operator/webcomponents";
import type { SectionDto } from "../api-client";
import { CONCEPTS } from "../concepts";

const brandIconSrc = (name: string) => `/icons/${name}.svg`;

function renderPrerequisite(id: string) {
  const concept = CONCEPTS[id];
  return concept ? <Link to={concept.route}>{concept.label}</Link> : id;
}

export function SectionCard({ section }: { section: SectionDto }) {
  return (
    <SharedSectionCard
      section={section}
      brandIconSrc={brandIconSrc}
      renderPrerequisite={renderPrerequisite}
    />
  );
}
