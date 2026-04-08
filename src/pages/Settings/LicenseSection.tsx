import { useEffect, useState } from "react";
import {
  getCommunicationCert,
  getLicenseInfo,
  exportMachineCredential,
  importLicenseFile,
  selectFiles,
} from "../../lib/tauri-api";
import { useAppContext } from "../../contexts/app-context";
import type { LicenseInfo, LicenseStatus } from "../../types";

/** Status badge color + label mapping */
const STATUS_CONFIG: Record<LicenseStatus, { color: string; label: string }> = {
  valid:            { color: "var(--cahtqt-color-success)",  label: "Valid" },
  expired:          { color: "var(--cahtqt-color-error)",    label: "Expired" },
  not_found:        { color: "var(--cahtqt-text-muted)",     label: "Not Found" },
  no_token:         { color: "var(--cahtqt-color-warning)",  label: "No Token" },
  token_mismatch:   { color: "var(--cahtqt-color-error)",    label: "Token Mismatch" },
  machine_mismatch: { color: "var(--cahtqt-color-error)",    label: "Machine Mismatch" },
  corrupted:        { color: "var(--cahtqt-color-error)",    label: "Corrupted" },
  no_communication_cert: { color: "var(--cahtqt-color-warning)", label: "No Comm Cert" },
};

export default function LicenseSection() {
  const { license } = useAppContext();
  const [info, setInfo] = useState<LicenseInfo | null>(null);
  const [hasCommunicationCert, setHasCommunicationCert] = useState(true); // default true to avoid flash
  const [feedback, setFeedback] = useState<{ type: "success" | "error"; msg: string } | null>(null);
  const [loading, setLoading] = useState(false);

  // Load license info + check comm cert on mount
  useEffect(() => {
    getLicenseInfo().then(setInfo).catch(() => {});
    getCommunicationCert()
      .then((cert) => setHasCommunicationCert(cert !== null))
      .catch(() => setHasCommunicationCert(false));
  }, []);

  // Auto-clear feedback after 4s
  useEffect(() => {
    if (!feedback) return;
    const t = setTimeout(() => setFeedback(null), 4000);
    return () => clearTimeout(t);
  }, [feedback]);

  const handleExport = async () => {
    setLoading(true);
    try {
      const result = await exportMachineCredential();
      setFeedback({ type: "success", msg: `Credential saved to ${result.saved_path}` });
    } catch (e) {
      setFeedback({ type: "error", msg: `Export failed: ${e}` });
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async () => {
    const files = await selectFiles([{ name: "License", extensions: ["dat"] }]);
    if (!files || files.length === 0) return;
    setLoading(true);
    try {
      const result = await importLicenseFile(files[0]);
      setInfo({ status: result.status, expires_at: result.expires_at, product: info?.product ?? null });
      setFeedback({ type: "success", msg: "License imported successfully" });
      // Sync license context so protected routes unlock immediately
      await license.recheckLicense();
    } catch (e) {
      setFeedback({ type: "error", msg: `Import failed: ${e}` });
    } finally {
      setLoading(false);
    }
  };

  const statusCfg = info ? STATUS_CONFIG[info.status] : null;
  const expiryText = info?.expires_at
    ? new Date(info.expires_at * 1000).toISOString().split("T")[0]
    : info?.status === "valid" ? "Perpetual" : null;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <div className="section-title" style={{ color: "var(--cahtqt-text-muted)" }}>
        LICENSE
      </div>
      <div style={{ fontSize: "var(--cahtqt-font-size-sm)", color: "var(--cahtqt-text-muted)" }}>
        Machine license status and management.
      </div>

      {/* Conditional banner — only when comm cert is not configured */}
      {!hasCommunicationCert && (
        <div style={{
          padding: "8px 12px",
          borderRadius: "var(--cahtqt-radius-md)",
          border: "1px solid var(--cahtqt-color-info, #3b82f6)",
          background: "rgba(59,130,246,0.08)",
          fontSize: "var(--cahtqt-font-size-sm)",
          color: "var(--cahtqt-text-on-light)",
        }}>
          ℹ Communication certificate must be configured before importing a license file.
        </div>
      )}

      {/* Status card */}
      {statusCfg && (
        <div style={{
          padding: 12,
          border: "1px solid var(--cahtqt-border-light)",
          borderRadius: "var(--cahtqt-radius-md)",
          background: "var(--cahtqt-bg-input)",
          display: "flex",
          flexDirection: "column",
          gap: 6,
        }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <span style={{
              width: 8, height: 8, borderRadius: "50%",
              background: statusCfg.color, display: "inline-block",
            }} />
            <span style={{ fontSize: "var(--cahtqt-font-size-sm)", fontWeight: "var(--cahtqt-font-weight-bold)", color: "var(--cahtqt-text-on-light)" }}>
              {statusCfg.label}
            </span>
          </div>
          {expiryText && (
            <div style={{ display: "flex", gap: 8, fontSize: "var(--cahtqt-font-size-sm)" }}>
              <span style={{ color: "var(--cahtqt-text-muted)", minWidth: 110 }}>Expires:</span>
              <span style={{ color: "var(--cahtqt-text-on-light)" }}>{expiryText}</span>
            </div>
          )}
        </div>
      )}

      {/* Action buttons */}
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <button className="btn btn-ghost" onClick={handleExport} disabled={loading}>
          {loading ? "Processing…" : "Export Machine Credential"}
        </button>
        <button className="btn btn-ghost" onClick={handleImport} disabled={loading}>
          Import License
        </button>
      </div>

      {/* Feedback message */}
      {feedback && (
        <div style={{
          fontSize: "var(--cahtqt-font-size-sm)",
          color: feedback.type === "success" ? "var(--cahtqt-color-success)" : "var(--cahtqt-color-error)",
        }}>
          {feedback.msg}
        </div>
      )}
    </div>
  );
}
