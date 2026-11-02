import { useEffect, useSyncExternalStore } from "react";
import { useApiQuery } from "./api";
import { licenseQuery } from "./api/definitions";

/**
 * Checkout return, operator side of the platform's contract (operator-platform CONTRACT.md,
 * "Checkout return"): the purchase link carries `return_to=<this UI's URL>`, and after payment the
 * platform navigates back to `<return_to>#license_key=<key>`. The fragment never reaches a server.
 */
export const RETURN_TO_PARAM = "return_to";
export const LICENSE_KEY_PARAM = "license_key";
export const LICENSE_RETURN_PATH = "/license/return";
/** How often a tab that opened checkout re-reads the licence until it turns Premium. */
export const CHECKOUT_POLL_MS = 3000;

type LocationLike = Pick<Location, "origin" | "pathname">;

/** The purchase URL plus where checkout should send the buyer back; null when unusable. */
export function withReturnTo(
  purchaseUrl: string | null | undefined,
  location: LocationLike = window.location,
): string | null {
  if (!purchaseUrl) {
    return null;
  }
  let url: URL;
  try {
    url = new URL(purchaseUrl);
  } catch {
    return null;
  }
  url.searchParams.set(RETURN_TO_PARAM, `${location.origin}${location.pathname}`);
  return url.toString();
}

/**
 * `#license_key=X` → `#/license/return?license_key=X`, so the hash router can route it.
 * Returns the rewritten hash, or null when the hash is not a checkout return.
 */
export function licenseReturnHash(hash: string): string | null {
  if (!hash.startsWith(`#${LICENSE_KEY_PARAM}=`)) {
    return null;
  }
  const key = new URLSearchParams(hash.slice(1)).get(LICENSE_KEY_PARAM);
  if (!key) {
    return null;
  }
  const query = new URLSearchParams({ [LICENSE_KEY_PARAM]: key });
  return `#${LICENSE_RETURN_PATH}?${query.toString()}`;
}

/** Runs before the router mounts; replaceState keeps the raw key out of history. */
export function rewriteLicenseFragment(win: Window = window): void {
  const rewritten = licenseReturnHash(win.location.hash);
  if (rewritten) {
    win.history.replaceState(
      null,
      "",
      `${win.location.pathname}${win.location.search}${rewritten}`,
    );
  }
}

/**
 * Per-tab checkout progress, shared by every component that shows the licence:
 * idle → awaiting (Buy clicked, checkout runs in another tab) → activated (licence turned Premium).
 */
export type CheckoutProgress = "idle" | "awaiting" | "activated";

let progress: CheckoutProgress = "idle";
const listeners = new Set<() => void>();

function setProgress(next: CheckoutProgress): void {
  if (progress !== next) {
    progress = next;
    listeners.forEach((listener) => listener());
  }
}

export function markCheckoutStarted(): void {
  setProgress("awaiting");
}

/** For tests. */
export function resetCheckoutProgress(): void {
  setProgress("idle");
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useCheckoutProgress(): CheckoutProgress {
  return useSyncExternalStore(subscribe, () => progress);
}

/**
 * The licence query, polled while a checkout opened from this tab is outstanding, so the page turns
 * Premium on its own once the return tab installs the key.
 */
export function useLicenseQuery() {
  const checkout = useCheckoutProgress();
  const query = useApiQuery(licenseQuery(), {
    pollIntervalMs: checkout === "awaiting" ? CHECKOUT_POLL_MS : undefined,
  });
  const premium = query.data?.premium ?? false;
  useEffect(() => {
    if (premium && progress === "awaiting") {
      setProgress("activated");
    }
  }, [premium]);
  return { ...query, checkout };
}

/** Click-capture for a wrapper around a paywall whose Buy link we do not own. */
export function onPurchaseLinkClick(event: { target: EventTarget | null }): void {
  if (event.target instanceof Element && event.target.closest("a[href]")) {
    markCheckoutStarted();
  }
}
