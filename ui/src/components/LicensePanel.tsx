import { useCallback, useState } from "react";
import type { LicenseResponse } from "../api-client";
import { useApiMutation, useApiQuery } from "../api";
import { installLicenseMutation, licenseQuery, removeLicenseMutation } from "../api/definitions";
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

export function LicensePanel() {
  const query = useApiQuery(licenseQuery());
  const install = useApiMutation(installLicenseMutation);
  const remove = useApiMutation(removeLicenseMutation);
  const [key, setKey] = useState("");
  const [error, setError] = useState<string | null>(null);
  const license = query.data;
  const busy = install.isPending || remove.isPending;

  const onRemove = useCallback(() => {
    setError(null);
    remove.mutate(
      {},
      {
        onError: (cause) => {
          setError(cause.message);
        },
      },
    );
  }, [remove]);

  const onSubmit = useCallback(
    (event: React.SubmitEvent<HTMLFormElement>) => {
      event.preventDefault();
      setError(null);
      install.mutate(
        { licenseKey: key.trim() },
        {
          onSuccess: () => {
            setKey("");
          },
          onError: (cause) => {
            setError(cause.message);
          },
        },
      );
    },
    [install, key],
  );

  const displayError = error ?? query.error?.message ?? null;

  return (
    <section className={styles.panel} aria-label="Operator license">
      <h2>Operator Premium</h2>
      <p>Premium enables remote targets. Multiple agents on this machine are available free.</p>
      {displayError && (
        <p role="alert" className={styles.error}>
          {displayError}
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
