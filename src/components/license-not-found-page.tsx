import { useState } from "react";
import { exportMachineCredential } from "../lib/tauri-api";
import type { LicenseGateState } from "../hooks/use-license-status";

interface Props {
  reason?: LicenseGateState;
}

/** Shown inside app-content area when license is missing or invalid */
export default function LicenseNotFoundPage({ reason }: Props) {
  const [feedback, setFeedback] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const isPending = reason === "pending";

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

  return (
    <div style={containerStyle}>
      <div style={cardStyle}>
        <div style={iconStyle} />
        <h2 style={headingStyle}>
          {isPending ? "Token Login Required" : "License Not Found"}
        </h2>
        <p style={bodyStyle}>
          {isPending
            ? "Your communication key is set but token is not logged in. Please login your token to activate the license."
            : "This application is not licensed for this machine. Use the button below to Export Machine Credential then contact your admin department."}
        </p>
        {!isPending && (
          <>
            <button
              className="btn btn-secondary"
              onClick={handleExport}
              disabled={loading}
              style={{ marginTop: 16 }}
            >
              {loading ? "Exporting…" : "Export Machine Credential"}
            </button>
            {feedback && <p style={feedbackStyle}>{feedback}</p>}
          </>
        )}
        <p style={hintStyle}>
          {isPending
            ? "Use the Login Token button in the sidebar or Settings page."
            : "To import a license file, go to Settings."}
        </p>
      </div>
    </div>
  );
}

// -- Styles (renders inside app-content area, not fullscreen) --

const containerStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "100%",
  height: "100%",
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

const iconStyle: React.CSSProperties = {
  width: 12,
  height: 12,
  borderRadius: "50%",
  background: "var(--cahtqt-color-warning, #f59e0b)",
  marginBottom: 16,
  animation: "pulse 1.5s ease-in-out infinite",
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

const feedbackStyle: React.CSSProperties = {
  fontSize: 12,
  color: "var(--cahtqt-text-muted, #999)",
  marginTop: 12,
};

const hintStyle: React.CSSProperties = {
  fontSize: 12,
  color: "var(--cahtqt-text-muted, #999)",
  marginTop: 16,
  fontStyle: "italic",
};
