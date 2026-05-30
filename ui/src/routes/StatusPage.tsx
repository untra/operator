// The "All sections" overview: every status section in one scrollable list.
// Reachable from the Dashboard (not the sidebar, which now links each section to
// its own page). Consumes the shared sections context — no polling of its own.
// Keeps the legacy `?s=` deep-link scroll for backward compatibility.

import { useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useSections } from '../sections-context';
import { PageHeader } from '../components/PageHeader';
import { SectionCard } from '../components/SectionCard';
import styles from './StatusPage.module.css';

const DOCS_URL = 'https://operator.untra.io/getting-started/';

export function StatusPage() {
  const [searchParams] = useSearchParams();
  const targetSection = searchParams.get('s');
  const { sections, error } = useSections();

  // Scroll to a deep-linked section (e.g. /status?s=git) once sections load.
  useEffect(() => {
    if (!sections || !targetSection) return;
    const el = document.getElementById(targetSection);
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }, [sections, targetSection]);

  return (
    <div className={styles.page}>
      <PageHeader
        title="All Sections"
        summary="Every operator status section in one view, ordered by setup prerequisites."
        docsUrl={DOCS_URL}
      />

      {error && <div className={styles.error}>API: {error}</div>}
      {!sections && !error && <div className={styles.loading}>Loading sections…</div>}
      {sections && sections.length === 0 && !error && (
        <div className={styles.loading}>No sections available.</div>
      )}

      <div className={styles.sections}>
        {sections?.map((section) => (
          <SectionCard key={section.id} section={section} />
        ))}
      </div>
    </div>
  );
}
