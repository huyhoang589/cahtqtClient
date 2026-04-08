import { useCallback, useEffect, useState } from "react";
import { checkLicense } from "../lib/tauri-api";
import type { LicenseCheckResult } from "../types";

/** Backend license check states + client-only "loading" state */
export type LicenseGateState = "loading" | LicenseCheckResult["state"];

export function useLicenseStatus() {
  const [licenseState, setLicenseState] = useState<LicenseGateState>("loading");
  const [licenseErrorMsg, setLicenseErrorMsg] = useState<string | null>(null);

  const recheckLicense = useCallback(async () => {
    setLicenseState("loading");
    try {
      const result = await checkLicense();
      setLicenseState(result.state);
      setLicenseErrorMsg(result.error_msg);
    } catch {
      setLicenseState("error");
      setLicenseErrorMsg("Failed to verify license.");
    }
  }, []);

  useEffect(() => {
    recheckLicense();
  }, [recheckLicense]);

  return { licenseState, licenseErrorMsg, recheckLicense };
}
