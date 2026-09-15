import { ConceptIcon } from "./ConceptIcon";
import styles from "./PageHeader.module.css";

export interface PageHeaderProps {
  title: string;
  summary: string;
  docsUrl: string;
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
        {summary}{" "}
        <a href={docsUrl} target="_blank" rel="noopener noreferrer" className={styles.docsLink}>
          Docs ↗
        </a>
      </small>
    </header>
  );
}
