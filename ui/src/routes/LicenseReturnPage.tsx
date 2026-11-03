import { useCallback, useEffect, useRef } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useApiMutation, useApiQuery } from "../api";
import { installLicenseMutation, setupStatusQuery } from "../api/definitions";
import { LicensePanel } from "../components/LicensePanel";
import { LICENSE_KEY_PARAM, LICENSE_RETURN_PATH } from "../license-return";
import styles from "./LicensePage.module.css";

/** Licence timestamps are i64 seconds, which cross the wire as bigint. */
const date = (seconds: bigint) => new Date(Number(seconds) * 1000).toLocaleString();

/**
 * Where checkout sends the buyer back (`#license_key=…`, rewritten in main.tsx). It sits outside
 * WorkspaceGate so a purchase made during onboarding lands here too, then hands back to onboarding.
 */
export function LicenseReturnPage() {
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const install = useApiMutation(installLicenseMutation);
  const setup = useApiQuery(setupStatusQuery());
  const key = params.get(LICENSE_KEY_PARAM)?.trim() ?? "";
  const attempted = useRef(false);

  useEffect(() => {
    if (!key || attempted.current) {
      return;
    }
    attempted.current = true;
    const forgetKey = () => {
      // Once applied, the key has no reason to stay in the address bar or history.
      Promise.resolve(navigate(LICENSE_RETURN_PATH, { replace: true })).catch(() => undefined);
    };
    install.mutate({ licenseKey: key }, { onSuccess: forgetKey });
  }, [install, key, navigate]);

  const onContinue = useCallback(() => {
    void navigate(setup.data?.initialized ? "/settings/license" : "/onboarding");
  }, [navigate, setup.data?.initialized]);

  const license = install.data;
  const continueLabel = setup.data?.initialized ? "Go to license settings" : "Continue setup";

  return (
    <main className={styles.page}>
      <h1>Operator Premium</h1>
      {!key && !license && !install.isPending && (
        <>
          <p role="alert">No license key came back from checkout. Paste your key below.</p>
          <LicensePanel />
        </>
      )}
      {install.isPending && <output>Activating your license…</output>}
      {install.error && (
        <>
          <p role="alert">Your license could not be applied: {install.error.message}</p>
          <LicensePanel initialKey={key} />
        </>
      )}
      {license && (
        <section aria-label="Activated license">
          <output>
            {license.premium
              ? "Operator Premium is active."
              : `License applied, but its status is ${license.status}.`}
          </output>
          {license.terms && (
            <dl>
              <dt>Licensed to</dt>
              <dd>{license.terms.sub}</dd>
              <dt>Tier</dt>
              <dd>{license.terms.tier}</dd>
              <dt>Expires</dt>
              <dd>{date(license.terms.exp)}</dd>
            </dl>
          )}
          <button type="button" onClick={onContinue}>
            {continueLabel}
          </button>
        </section>
      )}
    </main>
  );
}
