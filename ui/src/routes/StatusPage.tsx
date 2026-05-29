import { useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { OperatorApi } from '../api-client';
import type { SectionDto } from '../api-client';
import { useHost } from '../host';
import styles from './StatusPage.module.css';

const POLL_INTERVAL_MS = 3000;

export function StatusPage() {
  const host = useHost();
  const [searchParams] = useSearchParams();
  const targetSection = searchParams.get('s');
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

  // Scroll to the targeted section (e.g. /status?s=git) once sections load.
  useEffect(() => {
    if (!sections || !targetSection) return;
    const el = document.getElementById(targetSection);
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }, [sections, targetSection]);

  return (
    <div className={styles.page}>
      <h1 className={styles.title}>Status</h1>

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

function SectionCard({ section }: { section: SectionDto }) {
  return (
    <section
      id={section.id}
      className={styles.card}
      data-locked={!section.met ? 'true' : undefined}
    >
      <details open={section.met}>
        <summary className={styles.header}>
          <span className={styles.dot} data-health={section.health} />
          <span className={styles.label}>{section.label}</span>
          <span className={styles.description}>{section.description}</span>
          {!section.met && <span className={styles.lock} title="Prerequisites not met">🔒</span>}
        </summary>

        {!section.met && section.prerequisites.length > 0 && (
          <p className={styles.prereq}>Requires: {section.prerequisites.join(', ')}</p>
        )}

        {section.children.length > 0 ? (
          <ul className={styles.rows}>
            {section.children.map((row, i) => (
              <li
                key={`${row.id}-${i}`}
                className={styles.row}
                style={{ paddingLeft: `${Math.max(0, row.depth - 1) * 1.25}rem` }}
              >
                <span className={styles.dot} data-health={row.health} />
                <span className={styles.rowLabel}>{row.label}</span>
                {row.description && <span className={styles.rowDesc}>{row.description}</span>}
              </li>
            ))}
          </ul>
        ) : (
          <p className={styles.empty}>No details.</p>
        )}
      </details>
    </section>
  );
}
