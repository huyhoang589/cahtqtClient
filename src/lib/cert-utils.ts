import type { CertExpiryStatus } from "../types";

/** Returns expiry status for a cert given its Unix-timestamp valid_to. */
export function getCertExpiryStatus(valid_to: number): CertExpiryStatus {
  const now = Math.floor(Date.now() / 1000);
  const thirtyDays = 30 * 24 * 60 * 60;
  if (valid_to < now) return "expired";
  if (valid_to < now + thirtyDays) return "expiring_soon";
  return "valid";
}
