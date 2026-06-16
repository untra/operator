// A single status section, rendered identically on the unified Status overview
// and on each per-section page. Lifted out of StatusPage so both consume one
// component. Locked sections (prerequisites not met) render collapsed with the
// missing prerequisites shown as links to their pages — the "next steps."

import { Link } from 'react-router-dom';
import type { SectionDto } from '../api-client';
import { CONCEPTS } from '../concepts';
import { BrandIcon } from './BrandIcon';
import styles from './SectionCard.module.css';

export function SectionCard({ section }: { section: SectionDto }) {
  return (
    <section id={section.id} className={styles.card} data-locked={!section.met ? 'true' : undefined}>
      <details open={section.met}>
        <summary className={styles.header}>
          <span className={styles.dot} data-health={section.health} />
          <span className={styles.label}>{section.label}</span>
          <span className={styles.description}>{section.description}</span>
          {!section.met && (
            <span className={styles.lock} title="Prerequisites not met">
              🔒
            </span>
          )}
        </summary>

        {!section.met && section.prerequisites.length > 0 && (
          <p className={styles.prereq}>
            Requires:{' '}
            {section.prerequisites.map((id, i) => {
              const concept = CONCEPTS[id];
              return (
                <span key={id}>
                  {i > 0 && ', '}
                  {concept ? (
                    <Link to={concept.route} className={styles.prereqLink}>
                      {concept.label}
                    </Link>
                  ) : (
                    id
                  )}
                </span>
              );
            })}
          </p>
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
                {row.brand_icon && <BrandIcon name={row.brand_icon} />}
                <span className={styles.rowLabel}>{row.label}</span>
                {row.description && <span className={styles.rowDesc}>{row.description}</span>}
                {row.actions.length > 0 && (
                  <span className={styles.rowActions}>
                    {row.actions.map((action) => (
                      <a
                        key={action.url}
                        className={styles.rowAction}
                        href={action.url}
                        target="_blank"
                        rel="noreferrer"
                      >
                        {action.label}
                      </a>
                    ))}
                  </span>
                )}
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
