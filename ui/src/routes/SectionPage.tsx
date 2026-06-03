// Generic page for a single status section, driven by a concept key. Renders the
// standard PageHeader (title / rule / summary + docs link) above the live
// SectionCard for that section, pulled from the shared sections context. Used by
// the per-section sidebar routes (connections, kanban, llm, …).

import { CONCEPTS } from '../concepts';
import { useSection } from '../sections-context';
import { PageHeader } from '../components/PageHeader';
import { SectionCard } from '../components/SectionCard';
import styles from './SectionPage.module.css';

export function SectionPage({ conceptKey }: { conceptKey: string }) {
  const concept = CONCEPTS[conceptKey];
  const section = useSection(conceptKey);

  return (
    <div className={styles.page}>
      <PageHeader
        title={concept.label}
        summary={concept.summary}
        docsUrl={concept.docsUrl}
        icon={concept.icon}
      />
      {section ? (
        <SectionCard section={section} />
      ) : (
        <div className={styles.loading}>Loading section…</div>
      )}
    </div>
  );
}
