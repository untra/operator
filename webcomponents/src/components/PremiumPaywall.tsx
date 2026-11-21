import { useId } from "react";
import styles from "./PremiumPaywall.module.css";

export type PremiumPaywallProps = {
  feature: string;
  description?: string;
  purchaseUrl?: string | null;
  /** In-app Premium page (`#/…`); when set it replaces the checkout link. */
  premiumHref?: string | null;
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

/** An in-app hash route, or null; the paywall never sends an in-app link off the page. */
export function safeInAppDestination(href: string | null | undefined): string | null {
  return href && /^#\/./.test(href) ? href : null;
}

export function PremiumPaywall({
  feature,
  description,
  purchaseUrl,
  premiumHref,
  onAddLicense,
  inline = false,
}: PremiumPaywallProps) {
  const headingId = useId();
  const inApp = safeInAppDestination(premiumHref);
  const safePurchaseUrl = inApp ? null : safePurchaseDestination(purchaseUrl);
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
        {inApp && <a href={inApp}>See Premium</a>}
        {safePurchaseUrl && (
          <a href={safePurchaseUrl} target="_blank" rel="noopener noreferrer">
            Buy Premium
          </a>
        )}
      </div>
    </section>
  );
}
