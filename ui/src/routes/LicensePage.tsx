import { LicensePanel } from "../components/LicensePanel";
import { platformAccountUrl, useLicenseQuery } from "../license-return";
import styles from "./LicensePage.module.css";

/** The license as supplied to and read by this machine; managing or extending it is on the platform. */
export function LicensePage() {
  const license = useLicenseQuery();
  const manage = platformAccountUrl(license.data?.purchase_url);
  return (
    <main className={styles.page}>
      <h1>License</h1>
      <LicensePanel />
      {manage && (
        <p>
          <a href={manage} target="_blank" rel="noopener noreferrer">
            Manage license
          </a>
        </p>
      )}
    </main>
  );
}
