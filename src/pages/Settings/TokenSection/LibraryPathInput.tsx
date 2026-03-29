import { selectDllFile } from "../../../lib/tauri-api";

interface Props {
  mode: "auto" | "manual";
  manualPath: string;
  onModeChange: (mode: "auto" | "manual") => void;
  onManualPathChange: (path: string) => void;
  onManualPathBrowse: () => Promise<void>;
  onManualPathClear: () => void;
}

const radioStyle = (selected: boolean): React.CSSProperties => ({
  display: "flex",
  alignItems: "center",
  gap: 8,
  padding: "7px 10px",
  border: `1px solid ${selected ? "var(--cahtqt-color-primary-alt)" : "var(--cahtqt-border-light)"}`,
  borderRadius: "var(--cahtqt-radius-sm)",
  cursor: "pointer",
  background: selected ? "rgba(0,180,216,0.07)" : "var(--cahtqt-bg-input)",
  fontSize: "var(--cahtqt-font-size-sm)",
  color: selected ? "var(--cahtqt-color-primary-alt)" : "var(--cahtqt-text-on-light)",
  fontWeight: selected ? 600 : 400,
  userSelect: "none",
});

export default function LibraryPathInput({ mode, manualPath, onModeChange, onManualPathChange, onManualPathBrowse, onManualPathClear }: Props) {
  const hasManualPath = manualPath.trim().length > 0;

  const handleBrowse = async () => {
    const files = await selectDllFile();
    if (files && files[0]) {
      onManualPathChange(files[0]);
      await onManualPathBrowse();
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
      <div className="section-title" style={{ color: "var(--cahtqt-text-muted)" }}>
        PKCS#11 LIBRARY PATH
      </div>
      <div style={{ fontSize: "var(--cahtqt-font-size-sm)", color: "var(--cahtqt-text-muted)" }}>
        Specify the PKCS#11 middleware DLL path. Example: bit4xpki.dll, eTPKCS11.dll
      </div>

      {/* Mode radio buttons */}
      <div style={{ display: "flex", gap: 8 }}>
        <div style={radioStyle(mode === "auto")} onClick={() => onModeChange("auto")}>
          <input type="radio" readOnly checked={mode === "auto"} style={{ accentColor: "var(--cahtqt-color-primary-alt)" }} />
          Auto Select
        </div>
        <div style={radioStyle(mode === "manual")} onClick={() => onModeChange("manual")}>
          <input type="radio" readOnly checked={mode === "manual"} style={{ accentColor: "var(--cahtqt-color-primary-alt)" }} />
          Manual Select
        </div>
      </div>

      {/* Manual path input — shown only in manual mode */}
      {mode === "manual" && (
        <div style={{ display: "flex", gap: 6, alignItems: "center", marginTop: 4 }}>
          <input
            type="text"
            value={manualPath}
            onChange={(e) => onManualPathChange(e.target.value)}
            onBlur={(e) => { if (e.target.value.trim()) onManualPathChange(e.target.value.trim()); }}
            placeholder="Path to .dll file"
            style={{
              flex: 1,
              fontFamily: "var(--cahtqt-font-mono)",
              fontSize: "var(--cahtqt-font-size-sm)",
              border: `1px solid ${hasManualPath ? "var(--cahtqt-color-primary-alt)" : "var(--cahtqt-border-light)"}`,
              borderRadius: "var(--cahtqt-radius-sm)",
              padding: "6px 10px",
              background: "var(--cahtqt-bg-input)",
              color: "var(--cahtqt-text-on-light)",
            }}
          />
          {hasManualPath && (
            <button className="btn btn-ghost" onClick={onManualPathClear} title="Clear path">✕</button>
          )}
          <button className="btn btn-secondary" onClick={handleBrowse}>Browse…</button>
        </div>
      )}

      <div style={{ fontSize: "var(--cahtqt-font-size-xs)", color: mode === "manual" && hasManualPath ? "var(--cahtqt-color-success)" : "var(--cahtqt-text-muted)" }}>
        {mode === "auto"
          ? "Auto-detect mode — Scan Token will search known system paths."
          : hasManualPath
            ? "✓ Manual path configured — Scan Token will use this library."
            : "Enter or browse to the PKCS#11 .dll file."}
      </div>
    </div>
  );
}
