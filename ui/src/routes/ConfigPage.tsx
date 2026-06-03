import { useEffect, useState } from 'react';
import { OperatorApi } from '../api-client';
import type { StatusResponse, CollectionResponse, ProjectSummary } from '../api-client';
import { useHost } from '../host';
import { CONCEPTS } from '../concepts';
import { PageHeader } from '../components/PageHeader';
import styles from './ConfigPage.module.css';

const CONFIG = CONCEPTS.config;

export function ConfigPage() {
  const host = useHost();
  const [api] = useState(() => new OperatorApi(host));
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [collections, setCollections] = useState<CollectionResponse[]>([]);
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      api.status().then(setStatus),
      api.listCollections().then(setCollections),
      api.listProjects().then(setProjects),
    ])
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [api]);

  const handleActivateCollection = async (name: string) => {
    try {
      await api.activateCollection(name);
      const updated = await api.listCollections();
      setCollections(updated);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to activate collection');
    }
  };

  if (loading) return <div className={styles.loading}>Loading configuration...</div>;

  return (
    <div className={styles.page}>
      <PageHeader
        title={CONFIG.label}
        summary={CONFIG.summary}
        docsUrl={CONFIG.docsUrl}
        icon={CONFIG.icon}
      />

      {error && <div className={styles.error}>{error}</div>}

      {status && (
        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>Status</h2>
          <div className={styles.kvGrid}>
            <span className={styles.label}>Version</span>
            <span>{status.version}</span>
            <span className={styles.label}>Issue Types</span>
            <span>{status.issuetype_count}</span>
            <span className={styles.label}>Collections</span>
            <span>{status.collection_count}</span>
            <span className={styles.label}>Active Collection</span>
            <span>{status.active_collection}</span>
          </div>
        </section>
      )}

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Collections</h2>
        {collections.length === 0 ? (
          <p className={styles.empty}>No collections configured.</p>
        ) : (
          <div className={styles.collectionList}>
            {collections.map((c) => (
              <div key={c.name} className={`${styles.collectionCard} ${c.is_active ? styles.activeCollection : ''}`}>
                <div className={styles.collectionHeader}>
                  <span className={styles.collectionName}>{c.name}</span>
                  {c.is_active ? (
                    <span className={styles.activeBadge}>Active</span>
                  ) : (
                    <button className={styles.activateBtn} onClick={() => handleActivateCollection(c.name)}>
                      Activate
                    </button>
                  )}
                </div>
                <p className={styles.collectionDesc}>{c.description}</p>
                <div className={styles.collectionTypes}>
                  {c.types.map((t) => (
                    <span key={t} className={styles.typeTag}>{t}</span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Projects ({projects.length})</h2>
        {projects.length === 0 ? (
          <p className={styles.empty}>No projects discovered.</p>
        ) : (
          <table className={styles.table}>
            <thead>
              <tr>
                <th>Name</th>
                <th>Kind</th>
                <th>Languages</th>
                <th>Catalog</th>
              </tr>
            </thead>
            <tbody>
              {projects.map((p) => (
                <tr key={p.project_name}>
                  <td>{p.project_name}</td>
                  <td>{p.kind ?? '—'}</td>
                  <td>{p.languages.join(', ') || '—'}</td>
                  <td>{p.has_catalog_info ? 'Yes' : 'No'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}
