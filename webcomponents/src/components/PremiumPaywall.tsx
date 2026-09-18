import { useId } from "react";
import styles from "./PremiumPaywall.module.css";

export type PremiumPaywallProps = {
  feature: string;
  description?: string;
  purchaseUrl?: string | null;
  onAddLicense: () => void;
  inline?: boolean;
};

/**
 * The purchase destination, or null if it is not one we will link to.
 *
 * The URL is a build-time input rather than something this component controls,
 * so an http, javascript: or data: value must never become an anchor href.
 */
export function safePurchaseDestination(url: string | null | undefined): string | null {
  return url && /^https:\/\//i.test(url) ? url : null;
}

export function PremiumPaywall({
  feature,
  description,
  purchaseUrl,
  onAddLicense,
  inline = false,
}: PremiumPaywallProps) {
  const headingId = useId();
  const safePurchaseUrl = safePurchaseDestination(purchaseUrl);
  return (
    <section
      className={`${styles.panel} ${inline ? styles.inline : ""}`}
      aria-labelledby={headingId}
    >
      <div className={styles.heading}>
        <h2 id={headingId}>{feature}</h2>
        <span className={styles.badge}>Premium</span>
      </div>
      <p>{description ?? "Available with an Operator Premium license for this configuration."}</p>
      <div className={styles.actions}>
        <button type="button" onClick={onAddLicense}>
          Add license
        </button>
        {safePurchaseUrl && (
          <a href={safePurchaseUrl} target="_blank" rel="noopener noreferrer">
            View Premium
          </a>
        )}
      </div>
    </section>
  );
}
