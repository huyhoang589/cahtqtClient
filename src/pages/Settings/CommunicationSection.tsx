import { useEffect, useState } from "react";
import {
  selectCommKeyFile,
  getCommunicationCert,
  previewCommunicationKey,
  confirmSetCommunicationKey,
  cancelPreviewCommunicationKey,
  removeCommunicationKey,
} from "../../lib/tauri-api";
import type { CommunicationCertInfo } from "../../types";

type Status = "not_set" | "previewing" | "valid";

export default function CommunicationSection() {
  const [commCert, setCommCert] = useState<CommunicationCertInfo | null>(null);
  const [previewCert, setPreviewCert] = useState<CommunicationCertInfo | null>(null);
  const [previewSf1Path, setPreviewSf1Path] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const status: Status = previewCert ? "previewing" : commCert ? "valid" : "not_set";

  // Load saved cert on mount
  useEffect(() => {
    getCommunicationCert().then(setCommCert).catch(() => {});
  }, []);

  const handleBrowse = async () => {
    setError(null);
    const files = await selectCommKeyFile();
    if (!files || files.length === 0) return;
    const path = files[0];
    setLoading(true);
    try {
      const info = await previewCommunicationKey(path);
      setPreviewCert(info);
      setPreviewSf1Path(path);
    } catch (e) {
      setError(`Failed to decrypt communication key: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleConfirm = async () => {
    if (!previewCert || !previewSf1Path) return;
    setLoading(true);
    try {
      const saved = await confirmSetCommunicationKey(
        previewSf1Path,
        previewCert.cn,
        previewCert.org ?? null,
        previewCert.serial,
        previewCert.valid_until,
      );
      setCommCert(saved);
      setPreviewCert(null);
      setPreviewSf1Path("");
    } catch (e) {
      setError(`Failed to save: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCancelPreview = async () => {
    try {
      await cancelPreviewCommunicationKey();
    } catch {}
    setPreviewCert(null);
    setPreviewSf1Path("");
    setError(null);
  };

  const handleRemove = async () => {
    if (!window.confirm("Remove communication key? You will need to re-set it after.")) return;
    try {
      await removeCommunicationKey();
      setCommCert(null);
    } catch (e) {
      setError(`Failed to remove: ${e}`);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <div className="section-title" style={{ color: "var(--cahtqt-text-muted)" }}>
        SET COMMUNICATION
      </div>
      <div style={{ fontSize: "var(--cahtqt-font-size-sm)", color: "var(--cahtqt-text-muted)" }}>
        Select the encrypted communication key (.sf1) for recipient verification.
      </div>

      {/* Error display */}
      {error && (
        <div style={{
          padding: 8, fontSize: "var(--cahtqt-font-size-sm)",
          color: "var(--cahtqt-color-error)", background: "rgba(255,0,0,0.05)",
          borderRadius: "var(--cahtqt-radius-md)",
        }}>
          {error}
        </div>
      )}

      {/* Browse button — shown when no preview active */}
      {status !== "previewing" && (
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <button className="btn btn-ghost" onClick={handleBrowse} disabled={loading}>
            {loading ? "Decrypting…" : "Browse Communication Key…"}
          </button>
        </div>
      )}

      {/* Preview card — two-step: decrypt → show info → confirm/cancel */}
      {previewCert && (
        <div style={{
          padding: 12,
          border: "1px solid var(--cahtqt-border-light)",
          borderRadius: "var(--cahtqt-radius-md)",
          background: "var(--cahtqt-bg-input)",
          display: "flex", flexDirection: "column", gap: 6,
        }}>
          <div style={{
            fontSize: "var(--cahtqt-font-size-sm)",
            fontWeight: "var(--cahtqt-font-weight-bold)",
            color: "var(--cahtqt-text-on-light)", marginBottom: 4,
          }}>
            Communication Key Preview
          </div>
          <CertField label="Common Name" value={previewCert.cn} />
          <CertField label="Organization" value={previewCert.org ?? "—"} />
          <CertField label="Serial" value={previewCert.serial} mono />
          <CertField label="Valid Until" value={previewCert.valid_until} />
          <div style={{ display: "flex", gap: 8, marginTop: 6 }}>
            <button className="btn btn-ghost" onClick={handleConfirm} disabled={loading}
              style={{ color: "var(--cahtqt-color-success-dark)" }}>
              {loading ? "Saving…" : "Confirm SET KEY"}
            </button>
            <button className="btn btn-ghost" onClick={handleCancelPreview}
              style={{ color: "var(--cahtqt-text-muted)" }}>
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Current Recipient card */}
      {commCert && !previewCert && (
        <div style={{
          padding: 12,
          border: "1px solid var(--cahtqt-color-success-border)",
          borderRadius: "var(--cahtqt-radius-md)",
          background: "rgba(0, 180, 100, 0.05)",
          display: "flex", flexDirection: "column", gap: 6,
        }}>
          <div style={{
            fontSize: "var(--cahtqt-font-size-sm)",
            fontWeight: "var(--cahtqt-font-weight-bold)",
            color: "var(--cahtqt-color-success-dark)", marginBottom: 4,
          }}>
            Current Recipient — Valid
          </div>
          <CertField label="Common Name" value={commCert.cn} />
          <CertField label="Organization" value={commCert.org ?? "—"} />
          <CertField label="Serial" value={commCert.serial} mono />
          <CertField label="Valid Until" value={commCert.valid_until} />
          <button
            className="btn btn-ghost"
            onClick={handleRemove}
            style={{ marginTop: 6, alignSelf: "flex-start", color: "var(--cahtqt-text-muted)" }}
          >
            REMOVE KEY
          </button>
        </div>
      )}
    </div>
  );
}

function CertField({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div style={{ display: "flex", gap: 8, fontSize: "var(--cahtqt-font-size-sm)" }}>
      <span style={{ color: "var(--cahtqt-text-muted)", minWidth: 110 }}>{label}:</span>
      <span style={{ fontFamily: mono ? "var(--cahtqt-font-mono)" : undefined, color: "var(--cahtqt-text-on-light)" }}>
        {value}
      </span>
    </div>
  );
}
