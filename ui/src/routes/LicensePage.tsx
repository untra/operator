import { LicensePanel } from "../components/LicensePanel";
import styles from "./LicensePage.module.css";

export function LicensePage() {
  return (
    <main className={styles.page}>
      <h1>License</h1>
      <LicensePanel />
    </main>
  );
}
