import { useEffect, useState } from "react";
import {
  getTokenStatus,
  exportMachineCredential,
  importLicenseFile,
  selectFiles,
} from "../lib/tauri-api";

// ---- NoTokenScreen -----------------------------------------------------------

interface NoTokenScreenProps {
  onTokenDetected: () => void;
}

export function NoTokenScreen({ onTokenDetected }: NoTokenScreenProps) {
  // Poll token status every 2s — auto-transition when detected
  useEffect(() => {
    const id = setInterval(async () => {
      try {
        const res = await getTokenStatus();
        if (res.status !== "disconnected") onTokenDetected();
      } catch { /* ignore */ }
    }, 2000);
    return () => clearInterval(id);
  }, [onTokenDetected]);

  return (
    <div style={screenStyle}>
      <div style={cardStyle}>
        <div style={pulseStyle} />
        <h2 style={headingStyle}>Token Required</h2>
        <p style={bodyStyle}>Please insert your Bit4ID token to continue.</p>
        <p style={hintStyle}>Waiting for token…</p>
      </div>
    </div>
  );
}

// ---- NoLicenseScreen ---------------------------------------------------------

interface NoLicenseScreenProps {
  onLicenseImported: () => void;
}

export function NoLicenseScreen({ onLicenseImported }: NoLicenseScreenProps) {
  const [feedback, setFeedback] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleExport = async () => {
    setLoading(true);
    try {
      const result = await exportMachineCredential();
      setFeedback(`Credential saved to ${result.saved_path}`);
    } catch (e) {
      setFeedback(`Export failed: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async () => {
    const files = await selectFiles([{ name: "License", extensions: ["dat"] }]);
    if (!files || files.length === 0) return;
    setLoading(true);
    try {
      await importLicenseFile(files[0]);
      onLicenseImported();
    } catch (e) {
      setFeedback(`Import failed: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={screenStyle}>
      <div style={cardStyle}>
        <h2 style={headingStyle}>License Not Found</h2>
        <p style={bodyStyle}>
          This application is not licensed for this machine. Please contact your IT department or use the buttons below.
        </p>
        <div style={{ display: "flex", gap: 10, marginTop: 16 }}>
          <button className="btn btn-secondary" onClick={handleExport} disabled={loading}>
            Export Machine Credential
          </button>
          <button className="btn btn-secondary" onClick={handleImport} disabled={loading}>
            Import License
          </button>
        </div>
        {feedback && <p style={{ ...hintStyle, marginTop: 12 }}>{feedback}</p>}
      </div>
    </div>
  );
}

// ---- ErrorScreen -------------------------------------------------------------

interface ErrorScreenProps {
  errorMsg: string | null;
}

export function ErrorScreen({ errorMsg }: ErrorScreenProps) {
  return (
    <div style={screenStyle}>
      <div style={cardStyle}>
        <h2 style={{ ...headingStyle, color: "var(--cahtqt-color-error)" }}>License Error</h2>
        <p style={bodyStyle}>{errorMsg ?? "An unknown error occurred. Please restart the application."}</p>
        <p style={hintStyle}>Please contact your IT department for assistance.</p>
      </div>
    </div>
  );
}

// ---- Shared styles -----------------------------------------------------------

const screenStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "100vw",
  height: "100vh",
  background: "var(--cahtqt-bg-app, #f5f5f5)",
};

const cardStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  padding: 40,
  maxWidth: 460,
  borderRadius: "var(--cahtqt-radius-md, 8px)",
  border: "1px solid var(--cahtqt-border-light, #ddd)",
  background: "var(--cahtqt-bg-card, #fff)",
  textAlign: "center",
};

const headingStyle: React.CSSProperties = {
  fontSize: 20,
  fontWeight: 600,
  color: "var(--cahtqt-text-on-light, #222)",
  marginBottom: 8,
};

const bodyStyle: React.CSSProperties = {
  fontSize: 14,
  color: "var(--cahtqt-text-muted, #666)",
  lineHeight: 1.5,
};

const hintStyle: React.CSSProperties = {
  fontSize: 12,
  color: "var(--cahtqt-text-muted, #999)",
};

const pulseStyle: React.CSSProperties = {
  width: 12,
  height: 12,
  borderRadius: "50%",
  background: "var(--cahtqt-color-warning, #f59e0b)",
  marginBottom: 16,
  animation: "pulse 1.5s ease-in-out infinite",
};
