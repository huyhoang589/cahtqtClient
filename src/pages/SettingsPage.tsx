import { X } from "lucide-react";
import TokenSection from "./Settings/TokenSection/TokenSection";
import CommunicationSection from "./Settings/CommunicationSection";
import LicenseSection from "./Settings/LicenseSection";
import { useAppContext } from "../contexts/app-context";
import { selectOutputDir } from "../lib/tauri-api";

export default function SettingsPage() {
  const { outputDataDir, pkcs11Mode, pkcs11ManualPath, saveOutputDataDir, savePkcs11Mode, savePkcs11ManualPath } = useAppContext().settings;
  const hasOutputDir = outputDataDir.trim().length > 0;

  const handleBrowseOutputDir = async () => {
    const dir = await selectOutputDir();
    if (dir) await saveOutputDataDir(dir);
  };

  const handleClearOutputDir = async () => {
    await saveOutputDataDir("");
  };

  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      {/* Header */}
      <div style={{ marginBottom: 24, paddingBottom: 16, borderBottom: "1px solid var(--cahtqt-border-light)" }}>
        <h2 style={{ fontSize: "var(--cahtqt-font-size-xl)", color: "var(--cahtqt-text-on-light)" }}>Settings</h2>
      </div>

      {/* OUTPUT DATA DIR section */}
      <div style={{ display: "flex", flexDirection: "column", gap: 6, marginBottom: 24 }}>
        <div className="section-title" style={{ color: "var(--cahtqt-text-muted)" }}>
          OUTPUT DATA DIR
        </div>
        <div style={{ fontSize: "var(--cahtqt-font-size-sm)", color: "var(--cahtqt-text-muted)" }}>
          Directory where encrypted/decrypted files are saved. Leave empty to use Desktop.
        </div>
        <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <input
            type="text"
            value={outputDataDir}
            onChange={(e) => saveOutputDataDir(e.target.value).catch(() => {})}
            onBlur={(e) => saveOutputDataDir(e.target.value.trim()).catch(() => {})}
            placeholder="Default: Desktop folder"
            style={{
              flex: 1,
              fontFamily: "var(--cahtqt-font-mono)",
              fontSize: "var(--cahtqt-font-size-sm)",
              border: `1px solid ${hasOutputDir ? "var(--cahtqt-color-primary-alt)" : "var(--cahtqt-border-light)"}`,
              borderRadius: "var(--cahtqt-radius-sm)",
              padding: "6px 10px",
              background: "var(--cahtqt-bg-input)",
              color: "var(--cahtqt-text-on-light)",
            }}
          />
          {hasOutputDir && (
            <button className="btn btn-ghost" onClick={handleClearOutputDir} title="Clear output dir">
              <X size={14} />
            </button>
          )}
          <button className="btn btn-secondary" onClick={handleBrowseOutputDir}>Browse…</button>
        </div>
        <div style={{ fontSize: "var(--cahtqt-font-size-xs)", color: hasOutputDir ? "var(--cahtqt-color-success)" : "var(--cahtqt-text-muted)" }}>
          {hasOutputDir ? `✓ Output directory: ${outputDataDir}` : "Default: Desktop folder"}
        </div>
      </div>

      <div style={{ borderBottom: "1px solid var(--cahtqt-border-light)", marginBottom: 24 }} />

      {/* eToken / PKCS#11 hardware token section */}
      <TokenSection
        pkcs11Mode={pkcs11Mode}
        pkcs11ManualPath={pkcs11ManualPath}
        onPkcs11ModeChange={savePkcs11Mode}
        onPkcs11ManualPathChange={savePkcs11ManualPath}
      />

      <div style={{ borderBottom: "1px solid var(--cahtqt-border-light)", marginBottom: 24, marginTop: 24 }} />

      {/* Communication Certificate Configuration */}
      <CommunicationSection />

      <div style={{ borderBottom: "1px solid var(--cahtqt-border-light)", marginBottom: 24, marginTop: 24 }} />

      {/* License Management */}
      <LicenseSection />
    </div>
  );
}
