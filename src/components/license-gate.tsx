import { useCallback, useEffect, useState } from "react";
import { checkLicense } from "../lib/tauri-api";
import { NoTokenScreen, NoLicenseScreen, ErrorScreen } from "./license-screens";

type GateState = "loading" | "ok" | "no_token" | "no_license" | "error";

export default function LicenseGate({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<GateState>("loading");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const runCheck = useCallback(async () => {
    setState("loading");
    try {
      const result = await checkLicense();
      setState(result.state as GateState);
      setErrorMsg(result.error_msg);
    } catch {
      setState("error");
      setErrorMsg("Failed to verify license. Please restart the application.");
    }
  }, []);

  useEffect(() => { runCheck(); }, [runCheck]);

  if (state === "loading") {
    return (
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "center",
        width: "100vw", height: "100vh", background: "var(--cahtqt-bg-app, #f5f5f5)",
      }}>
        <span style={{ color: "var(--cahtqt-text-muted, #999)", fontSize: 14 }}>
          Verifying license…
        </span>
      </div>
    );
  }

  if (state === "no_token") return <NoTokenScreen onTokenDetected={runCheck} />;
  if (state === "no_license") return <NoLicenseScreen onLicenseImported={runCheck} />;
  if (state === "error") return <ErrorScreen errorMsg={errorMsg} />;

  return <>{children}</>;
}
