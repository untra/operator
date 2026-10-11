import type { ReactNode } from "react";
import type { LicenseResponse } from "../generated/LicenseResponse";
import { PremiumPaywall } from "./PremiumPaywall";

export type PremiumGateProps = {
  feature: string;
  description?: string;
  /** Absent while the license is loading. */
  license: LicenseResponse | null | undefined;
  onAddLicense: () => void;
  children: ReactNode;
};

/** Renders `children` for a premium license, the paywall otherwise. */
export function PremiumGate({
  feature,
  description,
  license,
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
        purchaseUrl={license.purchase_url}
        onAddLicense={onAddLicense}
      />
    );
  }
  return children;
}
