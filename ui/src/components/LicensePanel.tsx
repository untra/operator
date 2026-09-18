import { useCallback, useEffect, useState } from "react";
import type { LicenseResponse, OperatorApi } from "../api-client";
import styles from "./LicensePanel.module.css";

const STATUS_LABELS: Record<LicenseResponse["status"], string> = {
  missing: "Free",
  valid: "Premium",
  expired: "Expired",
  not_yet_valid: "Not yet valid",
  invalid: "Invalid",
};
/// Licence timestamps are i64 seconds, which cross the wire as bigint.
const date = (seconds: bigint) => new Date(Number(seconds) * 1000).toLocaleString();

export function LicensePanel({
  api,
  onChange,
}: {
  api: OperatorApi;
  onChange?: (license: LicenseResponse) => void;
}) {
  const [license, setLicense] = useState<LicenseResponse | null>(null);
  const [key, setKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    api
      .license()
      .then((value) => {
        if (active) {
          setLicense(value);
          onChange?.(value);
        }
        return undefined;
      })
      .catch((cause: unknown) => {
        if (active) {
          setError(cause instanceof Error ? cause.message : "Could not load license");
        }
      });
    return () => {
      active = false;
    };
  }, [api, onChange]);

  const update = useCallback(
    async (remove: boolean) => {
      setBusy(true);
      setError(null);
      try {
        await api.refreshCsrf();
        const value = remove ? await api.removeLicense() : await api.installLicense(key.trim());
        setLicense(value);
        setKey("");
        onChange?.(value);
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : "Could not update license");
      } finally {
        setBusy(false);
      }
    },
    [api, key, onChange],
  );

  const onRemove = useCallback(() => {
    void update(true);
  }, [update]);

  const onSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();
      void update(false);
    },
    [update],
  );

  return (
    <section className={styles.panel} aria-label="Operator license">
      <h2>Operator Premium</h2>
      <p>Premium enables remote targets. Multiple agents on this machine are available free.</p>
      {error && (
        <p role="alert" className={styles.error}>
          {error}
        </p>
      )}
      {license ? (
        <>
          <dl className={styles.terms}>
            <dt>Status</dt>
            <dd>{STATUS_LABELS[license.status]}</dd>
            <dt>Configuration ID</dt>
            <dd>
              <code>{license.profile_id}</code>
            </dd>
            {license.terms && (
              <>
                <dt>Licensed to</dt>
                <dd>{license.terms.sub}</dd>
                <dt>License ID</dt>
                <dd>{license.terms.jti}</dd>
                <dt>Tier</dt>
                <dd>{license.terms.tier}</dd>
                <dt>Issued</dt>
                <dd>{date(license.terms.iat)}</dd>
                <dt>Valid from</dt>
                <dd>{date(license.terms.nbf)}</dd>
                <dt>Expires</dt>
                <dd>{date(license.terms.exp)}</dd>
              </>
            )}
          </dl>
          {license.purchase_url && /^https:\/\//i.test(license.purchase_url) && (
            <p>
              <a href={license.purchase_url} target="_blank" rel="noopener noreferrer">
                View Operator Premium
              </a>
            </p>
          )}
        </>
      ) : (
        <p>Loading license…</p>
      )}
      <form className={styles.form} onSubmit={onSubmit}>
        <label>
          License key
          <input
            type="password"
            autoComplete="off"
            value={key}
            onChange={(event) => setKey(event.target.value)}
            disabled={busy}
          />
        </label>
        <button type="submit" disabled={busy || !key.trim()}>
          {busy ? "Saving…" : "Apply license"}
        </button>
      </form>
      {license && license.status !== "missing" && (
        <div className={styles.actions}>
          <button type="button" disabled={busy} onClick={onRemove}>
            Remove license
          </button>
        </div>
      )}
    </section>
  );
}
