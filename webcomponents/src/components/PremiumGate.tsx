import type { ReactNode } from "react";
import type { LicenseResponse } from "../generated/LicenseResponse";
import { PremiumPaywall } from "./PremiumPaywall";

export type PremiumGateProps = {
  feature: string;
  description?: string;
  /** Absent while the license is loading. */
  license: LicenseResponse | null | undefined;
  /** In-app Premium page the paywall links to; checkout starts there, not from the gate. */
  premiumHref?: string | null;
  onAddLicense: () => void;
  children: ReactNode;
};

/** Renders `children` for a premium license, the paywall otherwise. */
export function PremiumGate({
  feature,
  description,
  license,
  premiumHref,
  onAddLicense,
  children,
}: PremiumGateProps) {
  if (!license) {
    return null;
  }
  if (!license.premium) {
    return (
      <PremiumPaywall
        feature={feature}
        description={description}
        premiumHref={premiumHref}
        onAddLicense={onAddLicense}
      />
    );
  }
  return children;
}
