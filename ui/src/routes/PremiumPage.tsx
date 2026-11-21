import { Link } from "react-router-dom";
import { EXTERNAL_LINKS } from "../concepts";
import { onPurchaseLinkClick, useLicenseQuery, withReturnTo } from "../license-return";
import styles from "./PremiumPage.module.css";

/** Licence timestamps are i64 seconds, which cross the wire as bigint. */
const date = (seconds: bigint) => new Date(Number(seconds) * 1000).toLocaleDateString();

const UNLOCKS = [
  {
    route: "/remote-targets",
    label: "Remote targets",
    detail: "Run agents on SSH hosts and Coder workspaces, reporting back here.",
  },
  {
    route: "/campaigns",
    label: "Campaigns",
    detail: "Plan epics into dependency waves and launch them together.",
  },
] as const;

function ExternalLinks() {
  return (
    <section aria-label="Operator online">
      <h2>Operator online</h2>
      <ul className={styles.links}>
        {EXTERNAL_LINKS.map((link) => (
          <li key={link.href}>
            <a href={link.href} target="_blank" rel="noopener noreferrer">
              {link.label}
            </a>
          </li>
        ))}
      </ul>
    </section>
  );
}

/**
 * The upsell for a workspace that is already running, and the only in-app place checkout starts.
 * With Premium it shows what this instance has unlocked instead.
 */
export function PremiumPage() {
  const license = useLicenseQuery();
  const data = license.data;
  const buy = data?.premium ? null : withReturnTo(data?.purchase_url, "upgrade");
  const secureBuy = buy && /^https:\/\//i.test(buy) ? buy : null;
  const awaiting = license.checkout === "awaiting" && !data?.premium;

  return (
    <main className={styles.page}>
      <h1>Operator Premium</h1>
      {license.error && <p role="alert">{license.error.message}</p>}
      {!data && !license.error && <p>Loading…</p>}
      {data?.premium && (
        <section aria-label="Premium active" className={styles.active}>
          <output>Operator Premium is active on this configuration.</output>
          {data.terms && (
            <p>
              {data.terms.tier} license for {data.terms.sub}, valid until {date(data.terms.exp)}.
            </p>
          )}
          <h2>Unlocked here</h2>
          <ul className={styles.unlocks}>
            {UNLOCKS.map((unlock) => (
              <li key={unlock.route}>
                <Link to={unlock.route}>{unlock.label}</Link>
                <span>{unlock.detail}</span>
              </li>
            ))}
          </ul>
          <p>
            <Link to="/settings/license">License details</Link>
          </p>
        </section>
      )}
      {data && !data.premium && (
        <section aria-label="Upgrade" className={styles.upsell}>
          <p>Multiple agents on this machine are free. Premium takes Operator further:</p>
          <ul className={styles.unlocks}>
            {UNLOCKS.map((unlock) => (
              <li key={unlock.route}>
                <strong>{unlock.label}</strong>
                <span>{unlock.detail}</span>
              </li>
            ))}
          </ul>
          {awaiting && (
            <output>
              Finish checkout in the new tab. This page updates automatically once your license is
              applied.
            </output>
          )}
          {secureBuy ? (
            <div onClickCapture={onPurchaseLinkClick}>
              <a className={styles.buy} href={secureBuy} target="_blank" rel="noopener noreferrer">
                Buy Operator Premium
              </a>
            </div>
          ) : (
            <p>This build has no storefront configured.</p>
          )}
          <p>
            Already have a key? <Link to="/settings/license">Add it on the License page</Link>.
          </p>
        </section>
      )}
      <ExternalLinks />
    </main>
  );
}
