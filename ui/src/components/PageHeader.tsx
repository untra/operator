// The standard top-of-page header shared by every route: an h1 title, a rule,
// then a small tag carrying the page's one-line summary and a link to the docs.
// This is the only place rich text lives on a page — everything below is the
// functional view.

import { ConceptIcon } from './ConceptIcon';
import styles from './PageHeader.module.css';

interface PageHeaderProps {
  title: string;
  summary: string;
  docsUrl: string;
  /** Optional codicon name shown before the title. */
  icon?: string;
}

export function PageHeader({ title, summary, docsUrl, icon }: PageHeaderProps) {
  return (
    <header className={styles.header}>
      <h1 className={styles.title}>
        {icon && <ConceptIcon name={icon} className={styles.titleIcon} />}
        {title}
      </h1>
      <hr className={styles.rule} />
      <small className={styles.tag}>
        {summary}{' '}
        <a href={docsUrl} target="_blank" rel="noopener noreferrer" className={styles.docsLink}>
          Docs ↗
        </a>
      </small>
    </header>
  );
}
