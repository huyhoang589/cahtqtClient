import { useAppContext } from "../contexts/app-context";
import LicenseNotFoundPage from "./license-not-found-page";

/** Route guard — renders children only when license is valid, otherwise shows license prompt */
export default function LicenseRequired({ children }: { children: React.ReactNode }) {
  const { license } = useAppContext();

  if (license.licenseState === "loading") {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", width: "100%", height: "100%" }}>
        <span style={{ color: "var(--cahtqt-text-muted, #999)", fontSize: 14 }}>
          Verifying license…
        </span>
      </div>
    );
  }

  if (license.licenseState !== "ok") {
    return <LicenseNotFoundPage reason={license.licenseState} />;
  }

  return <>{children}</>;
}
