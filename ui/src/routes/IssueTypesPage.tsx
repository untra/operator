import { useEffect, useState } from 'react';
import { OperatorApi } from '../api-client';
import type { IssueTypeSummary, IssueTypeResponse } from '../api-client';
import { useHost } from '../host';
import styles from './IssueTypesPage.module.css';

export function IssueTypesPage() {
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [issueTypes, setIssueTypes] = useState<IssueTypeSummary[]>([]);
  const [selected, setSelected] = useState<IssueTypeResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .listIssueTypes()
      .then(setIssueTypes)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [api]);

  const handleSelect = async (key: string) => {
    try {
      const detail = await api.getIssueType(key);
      setSelected(detail);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load issue type');
    }
  };

  if (loading) return <div className={styles.loading}>Loading issue types...</div>;

  return (
    <div className={styles.page}>
      <h1 className={styles.title}>Issue Types</h1>

      {error && <div className={styles.error}>{error}</div>}

      <div className={styles.split}>
        <div className={styles.list}>
          {issueTypes.map((it) => (
            <button
              key={it.key}
              className={`${styles.item} ${selected?.key === it.key ? styles.selectedItem : ''}`}
              onClick={() => handleSelect(it.key)}
            >
              <span className={styles.glyph}>{it.glyph}</span>
              <div>
                <div className={styles.itemName}>{it.name}</div>
                <div className={styles.itemMeta}>
                  {it.key} &middot; {it.mode} &middot; {it.stepCount} steps
                </div>
              </div>
            </button>
          ))}
        </div>

        <div className={styles.detail}>
          {selected ? (
            <>
              <h2 className={styles.detailTitle}>
                <span className={styles.detailGlyph}>{selected.glyph}</span>
                {selected.name}
              </h2>
              <p className={styles.detailDesc}>{selected.description}</p>
              <div className={styles.kvGrid}>
                <span className={styles.label}>Key</span>
                <span>{selected.key}</span>
                <span className={styles.label}>Mode</span>
                <span>{selected.mode}</span>
                <span className={styles.label}>Source</span>
                <span>{selected.source}</span>
                <span className={styles.label}>Steps</span>
                <span>{selected.steps.length}</span>
              </div>
              {selected.steps.length > 0 && (
                <div className={styles.steps}>
                  <h3 className={styles.stepsTitle}>Workflow Steps</h3>
                  <ol className={styles.stepList}>
                    {selected.steps.map((step) => (
                      <li key={step.name} className={styles.step}>
                        <span className={styles.stepName}>{step.display_name ?? step.name}</span>
                        <span className={styles.stepMeta}>{step.review_type} review</span>
                      </li>
                    ))}
                  </ol>
                </div>
              )}
            </>
          ) : (
            <div className={styles.placeholder}>Select an issue type to view details.</div>
          )}
        </div>
      </div>
    </div>
  );
}
