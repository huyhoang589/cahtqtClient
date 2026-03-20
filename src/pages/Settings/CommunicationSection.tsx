import { useEffect, useState } from "react";
import { emit } from "@tauri-apps/api/event";
import {
  selectCertFile,
  importCertPreview,
  getCommunicationCert,
  saveCommunicationCert,
  clearCommunicationCert,
} from "../../lib/tauri-api";
import type { CertInfo, CommunicationCertInfo } from "../../types";

export default function CommunicationSection() {
  const [commCert, setCommCert] = useState<CommunicationCertInfo | null>(null);
  const [previewCert, setPreviewCert] = useState<CertInfo | null>(null);
  const [previewPath, setPreviewPath] = useState<string>("");
  const [loading, setLoading] = useState(false);

  // Load saved cert on mount
  useEffect(() => {
    getCommunicationCert().then(setCommCert).catch(() => {});
  }, []);

  const handleBrowse = async () => {
    const files = await selectCertFile();
    if (!files || files.length === 0) return;
    const path = files[0];
    try {
      const info = await importCertPreview(path);
      setPreviewCert(info);
      setPreviewPath(path);
    } catch (e) {
      alert(`Failed to read certificate: ${e}`);
    }
  };

  const handleSave = async () => {
    if (!previewPath) return;
    setLoading(true);
    try {
      const saved = await saveCommunicationCert(previewPath);
      setCommCert(saved);
      setPreviewCert(null);
      setPreviewPath("");
      // Notify other pages (e.g. EncryptPage) that cert changed
      await emit("communication-cert-changed");
    } catch (e) {
      alert(`Failed to save: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleClear = async () => {
    if (!window.confirm("Remove communication certificate configuration?")) return;
    try {
      await clearCommunicationCert();
      setCommCert(null);
      // Notify other pages that cert was cleared
      await emit("communication-cert-changed");
    } catch (e) {
      alert(`Failed to clear: ${e}`);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <div className="section-title" style={{ color: "var(--color-text-muted-light)" }}>
        SET COMMUNICATION
      </div>
      <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-muted-light)" }}>
        Select the recipient certificate for encrypted communication files.
      </div>

      {/* Browse button */}
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <button className="btn btn-ghost" onClick={handleBrowse}>
          Browse Certificate…
        </button>
      </div>

      {/* Certificate Preview card */}
      {previewCert && (
        <div style={{
          padding: 12,
          border: "1px solid var(--color-border-light)",
          borderRadius: "var(--radius-md)",
          background: "var(--color-bg-input, #fff)",
          display: "flex",
          flexDirection: "column",
          gap: 6,
        }}>
          <div style={{ fontSize: "var(--font-size-sm)", fontWeight: "var(--font-weight-bold)", color: "var(--color-text-on-light)", marginBottom: 4 }}>
            Certificate Preview
          </div>
          <CertField label="Common Name" value={previewCert.cn} />
          <CertField label="Organization" value={previewCert.org ?? "—"} />
          <CertField label="Serial" value={previewCert.serial} mono />
          <CertField label="Valid Until" value={new Date(previewCert.valid_to * 1000).toISOString().split("T")[0]} />
          <button
            className="btn btn-ghost"
            onClick={handleSave}
            disabled={loading}
            style={{ marginTop: 6, alignSelf: "flex-start" }}
          >
            {loading ? "Saving…" : "Save Communication Cert"}
          </button>
        </div>
      )}

      {/* Current Recipient card */}
      {commCert && (
        <div style={{
          padding: 12,
          border: "1px solid #00b464",
          borderRadius: "var(--radius-md)",
          background: "rgba(0, 180, 100, 0.05)",
          display: "flex",
          flexDirection: "column",
          gap: 6,
        }}>
          <div style={{ fontSize: "var(--font-size-sm)", fontWeight: "var(--font-weight-bold)", color: "#00875a", marginBottom: 4 }}>
            Current Recipient
          </div>
          <CertField label="Common Name" value={commCert.cn} />
          <CertField label="Organization" value={commCert.org ?? "—"} />
          <CertField label="Serial" value={commCert.serial} mono />
          <CertField label="Valid Until" value={commCert.valid_until} />
          <button
            className="btn btn-ghost"
            onClick={handleClear}
            style={{ marginTop: 6, alignSelf: "flex-start", color: "var(--color-text-muted-light)" }}
          >
            Clear
          </button>
        </div>
      )}
    </div>
  );
}

function CertField({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div style={{ display: "flex", gap: 8, fontSize: "var(--font-size-sm)" }}>
      <span style={{ color: "var(--color-text-muted-light)", minWidth: 110 }}>{label}:</span>
      <span style={{ fontFamily: mono ? "var(--font-mono)" : undefined, color: "var(--color-text-on-light)" }}>
        {value}
      </span>
    </div>
  );
}
