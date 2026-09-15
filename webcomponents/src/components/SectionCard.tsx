import { useMemo, type ReactNode } from "react";
import type { SectionDto } from "../generated/SectionDto";
import { BrandIcon } from "./BrandIcon";
import styles from "./SectionCard.module.css";

export interface SectionCardProps {
  section: SectionDto;
  renderPrerequisite?: (id: string) => ReactNode;
  brandIconSrc?: (name: string) => string;
}

export function SectionCard({ section, renderPrerequisite, brandIconSrc }: SectionCardProps) {
  return (
    <section
      id={section.id}
      className={styles.card}
      data-locked={!section.met ? "true" : undefined}
    >
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
            Requires:{" "}
            {section.prerequisites.map((id, index) => (
              <span key={id}>
                {index > 0 && ", "}
                {renderPrerequisite?.(id) ?? id}
              </span>
            ))}
          </p>
        )}
        {section.children.length > 0 ? (
          <ul className={styles.rows}>
            {section.children.map((row) => (
              <SectionRow key={row.id} row={row} brandIconSrc={brandIconSrc} />
            ))}
          </ul>
        ) : (
          <p className={styles.empty}>No details.</p>
        )}
      </details>
    </section>
  );
}

const INDENT_REM = 1.25;

function SectionRow({
  row,
  brandIconSrc,
}: {
  row: SectionDto["children"][number];
  brandIconSrc?: (name: string) => string;
}) {
  const style = useMemo(
    () => ({ paddingLeft: `${Math.max(0, row.depth - 1) * INDENT_REM}rem` }),
    [row.depth],
  );
  return (
    <li className={styles.row} style={style}>
      <span className={styles.dot} data-health={row.health} />
      {row.brand_icon && brandIconSrc && <BrandIcon src={brandIconSrc(row.brand_icon)} />}
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
  );
}
