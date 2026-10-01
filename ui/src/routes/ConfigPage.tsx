import { useApiMutation, useApiQuery } from "../api";
import {
  activateCollectionMutation,
  collectionsQuery,
  projectsQuery,
  statusQuery,
} from "../api/definitions";
import { CONCEPTS } from "../concepts";
import { PageHeader } from "../components/PageHeader";
import { ProfileSelector } from "../profiles-context";
import styles from "./ConfigPage.module.css";

const CONFIG = CONCEPTS.config;

export function ConfigPage() {
  const status = useApiQuery(statusQuery());
  const collections = useApiQuery(collectionsQuery());
  const projects = useApiQuery(projectsQuery());
  const activate = useApiMutation(activateCollectionMutation);
  const loading = status.isLoading || collections.isLoading || projects.isLoading;
  const error =
    status.error?.message ??
    collections.error?.message ??
    projects.error?.message ??
    activate.error?.message ??
    null;

  const handleActivateCollection = (name: string) => {
    activate.mutate({ name });
  };

  if (loading) {
    return <div className={styles.loading}>Loading configuration...</div>;
  }

  return (
    <div className={styles.page}>
      <PageHeader
        title={CONFIG.label}
        summary={CONFIG.summary}
        docsUrl={CONFIG.docsUrl}
        icon={CONFIG.icon}
      />

      {error && <div className={styles.error}>{error}</div>}

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Configurations</h2>
        <ProfileSelector />
      </section>

      {status.data && (
        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>Status</h2>
          <div className={styles.kvGrid}>
            <span className={styles.label}>Version</span>
            <span>{status.data.version}</span>
            <span className={styles.label}>Issue Types</span>
            <span>{status.data.issuetype_count}</span>
            <span className={styles.label}>Collections</span>
            <span>{status.data.collection_count}</span>
            <span className={styles.label}>Active Collection</span>
            <span>{status.data.active_collection}</span>
          </div>
        </section>
      )}

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Collections</h2>
        {(collections.data ?? []).length === 0 ? (
          <p className={styles.empty}>No collections configured.</p>
        ) : (
          <div className={styles.collectionList}>
            {(collections.data ?? []).map((c) => (
              <div
                key={c.name}
                className={`${styles.collectionCard} ${c.is_active ? styles.activeCollection : ""}`}
              >
                <div className={styles.collectionHeader}>
                  <span className={styles.collectionName}>{c.name}</span>
                  {c.is_active ? (
                    <span className={styles.activeBadge}>Active</span>
                  ) : (
                    <button
                      className={styles.activateBtn}
                      onClick={() => handleActivateCollection(c.name)}
                    >
                      Activate
                    </button>
                  )}
                </div>
                <p className={styles.collectionDesc}>{c.description}</p>
                <div className={styles.collectionTypes}>
                  {c.types.map((t) => (
                    <span key={t} className={styles.typeTag}>
                      {t}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Projects ({(projects.data ?? []).length})</h2>
        {(projects.data ?? []).length === 0 ? (
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
              {(projects.data ?? []).map((p) => (
                <tr key={p.project_name}>
                  <td>{p.project_name}</td>
                  <td>{p.kind ?? "-"}</td>
                  <td>{p.languages.join(", ") || "-"}</td>
                  <td>{p.has_catalog_info ? "Yes" : "No"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}
