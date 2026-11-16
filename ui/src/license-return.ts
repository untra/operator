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

/** Which purchase flow sent the buyer to checkout; it rides back in `return_to`'s query. */
export type CheckoutFlow = "setup" | "upgrade";
export const CHECKOUT_FLOW_PARAM = "checkout";
const DESTINATIONS: Record<CheckoutFlow, string> = {
  setup: "/onboarding?step=license",
  upgrade: "/settings/premium",
};

export function parseCheckoutFlow(value: string | null | undefined): CheckoutFlow | null {
  return value === "setup" || value === "upgrade" ? value : null;
}

/** Where a completed checkout continues once its key is installed; null stays on the return page. */
export function checkoutDestination(flow: string | null | undefined): string | null {
  const parsed = parseCheckoutFlow(flow);
  return parsed ? DESTINATIONS[parsed] : null;
}

const PLATFORM_ACCOUNT_PATH = "/account";
/** In-app paywalls link here; it is the only in-app place checkout starts. */
export const PREMIUM_PAGE_HREF = `#${DESTINATIONS.upgrade}`;

/** The platform page that manages and extends licenses, on the storefront this build buys from. */
export function platformAccountUrl(purchaseUrl: string | null | undefined): string | null {
  if (!purchaseUrl) {
    return null;
  }
  try {
    const url = new URL(purchaseUrl);
    return url.protocol === "https:" ? `${url.origin}${PLATFORM_ACCOUNT_PATH}` : null;
  } catch {
    return null;
  }
}

type LocationLike = Pick<Location, "origin" | "pathname">;

/** The purchase URL plus where checkout should send the buyer back; null when unusable. */
export function withReturnTo(
  purchaseUrl: string | null | undefined,
  flow: CheckoutFlow,
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
  const back = new URLSearchParams({ [CHECKOUT_FLOW_PARAM]: flow });
  url.searchParams.set(
    RETURN_TO_PARAM,
    `${location.origin}${location.pathname}?${back.toString()}`,
  );
  return url.toString();
}

/**
 * `#license_key=X` → `#/license/return?license_key=X[&checkout=flow]`, so the hash router can route
 * it. Returns the rewritten hash, or null when the hash is not a checkout return.
 */
export function licenseReturnHash(hash: string, search = ""): string | null {
  if (!hash.startsWith(`#${LICENSE_KEY_PARAM}=`)) {
    return null;
  }
  const key = new URLSearchParams(hash.slice(1)).get(LICENSE_KEY_PARAM);
  if (!key) {
    return null;
  }
  const query = new URLSearchParams({ [LICENSE_KEY_PARAM]: key });
  const flow = parseCheckoutFlow(new URLSearchParams(search).get(CHECKOUT_FLOW_PARAM));
  if (flow) {
    query.set(CHECKOUT_FLOW_PARAM, flow);
  }
  return `#${LICENSE_RETURN_PATH}?${query.toString()}`;
}

type ReturnLocation = Pick<Location, "pathname" | "search" | "hash">;

/** The routed URL for a checkout return, with the checkout tag moved out of the query; else null. */
export function returnLocation(location: ReturnLocation): string | null {
  const rewritten = licenseReturnHash(location.hash, location.search);
  if (!rewritten) {
    return null;
  }
  const search = new URLSearchParams(location.search);
  search.delete(CHECKOUT_FLOW_PARAM);
  const rest = search.toString();
  return `${location.pathname}${rest ? `?${rest}` : ""}${rewritten}`;
}

/** Runs before the router mounts; replaceState keeps the raw key out of history. */
export function rewriteLicenseFragment(win: Window = window): void {
  const rewritten = returnLocation(win.location);
  if (rewritten) {
    win.history.replaceState(null, "", rewritten);
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
