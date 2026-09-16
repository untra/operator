import { useMemo } from "react";
import { OperatorApi } from "../api-client";
import { useHost } from "../host";
import { LicensePanel } from "../components/LicensePanel";
import styles from "./LicensePage.module.css";

export function LicensePage() {
  const host = useHost();
  const api = useMemo(() => new OperatorApi(host), [host]);
  return (
    <main className={styles.page}>
      <h1>License</h1>
      <LicensePanel api={api} />
    </main>
  );
}
