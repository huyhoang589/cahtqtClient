import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { FolderOpen } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import EncryptProgressPanel from "../components/encrypt-progress-panel";
import FileListPanel from "../components/file-list-panel";
import TokenWarningBar from "../components/token-warning-bar";
import { useEncrypt } from "../hooks/use-encrypt";
import { useTokenStatus } from "../hooks/use-token-status";
import { useSettingsStore } from "../hooks/use-settings-store";
import { useFileStatuses } from "../hooks/use-file-statuses";
import { getCommunicationCert, getAppSettings, openFolder } from "../lib/tauri-api";
import type { CommunicationCertInfo } from "../types";

type Step = "idle" | "running" | "done";

export default function EncryptPage() {
  const {
    selectedFiles, setSelectedFiles,
    isEncrypting, progress, result,
    startEncrypt, reset,
  } = useEncrypt();

  const navigate = useNavigate();
  const { dll_found, status: tokenStatus } = useTokenStatus();
  const { outputDataDir } = useSettingsStore();
  const { fileStatuses, resetStatuses } = useFileStatuses("encrypt-progress", "encrypt");

  const [commCert, setCommCert] = useState<CommunicationCertInfo | null>(null);
  const [step, setStep] = useState<Step>("idle");

  // Load communication cert on mount; refresh when Settings page saves/clears cert
  useEffect(() => {
    getCommunicationCert().then(setCommCert).catch(() => {});
    const unlisten = listen("communication-cert-changed", () => {
      getCommunicationCert().then(setCommCert).catch(() => {});
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  const handleEncryptClick = () => {
    if (selectedFiles.length === 0 || !commCert) return;
    if (step === "done") {
      const ok = window.confirm(
        "Output files from the previous batch may already exist. Start a new batch? (Existing files will be overwritten.)"
      );
      if (!ok) return;
      handleReset();
    }
    const ok = window.confirm(
      `Encrypt ${selectedFiles.length} file(s) to ${commCert.cn}?`
    );
    if (ok) handleConfirm();
  };

  const handleConfirm = async () => {
    if (!commCert) return;
    setStep("running");
    resetStatuses(selectedFiles);
    const outputDir = outputDataDir
      ? `${outputDataDir}/SF/ENCRYPT/`  // flat — no partner subfolder
      : null;
    await startEncrypt([commCert.file_path], commCert.cn, outputDir);
    setStep("done");
  };

  const handleReset = () => { reset(); setStep("idle"); };

  const handleOpenFolder = async () => {
    try {
      const settings = await getAppSettings();
      await openFolder(`${settings.output_data_dir}/SF/ENCRYPT`);
    } catch {}
  };

  // [HIGH RISK] canEncrypt MUST include commCert !== null — no cert = no encryption allowed
  const canEncrypt =
    dll_found && tokenStatus === "logged_in" &&
    selectedFiles.length > 0 &&
    commCert !== null &&
    !isEncrypting;

  const showProgress = step !== "idle" || result !== null;

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      <TokenWarningBar onLogin={() => navigate("/settings")} />

      {/* Recipient banner: green = cert configured, amber = not configured */}
      <div style={{
        padding: "8px 20px",
        fontSize: "var(--font-size-sm)",
        background: commCert ? "rgba(0, 180, 100, 0.08)" : "rgba(255, 180, 0, 0.08)",
        borderBottom: `2px solid ${commCert ? "#00b464" : "#f0a000"}`,
        color: commCert ? "#00875a" : "#a06800",
        display: "flex",
        alignItems: "center",
        gap: 8,
        flexShrink: 0,
      }}>
        {commCert
          ? `Recipient: ${commCert.cn} (${commCert.serial})`
          : "No communication certificate configured — go to Settings to set one"}
      </div>

      {/* Header */}
      <div style={{ padding: "16px 20px 12px", borderBottom: "1px solid var(--color-border-light)", display: "flex", justifyContent: "space-between", alignItems: "center", flexShrink: 0 }}>
        <h2 style={{ fontSize: "var(--font-size-xl)", color: "var(--color-text-on-light)" }}>Encrypt Files</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-ghost" onClick={handleOpenFolder} title="Open output folder" style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <FolderOpen size={14} /> Open Folder
          </button>
          <button className="primary" onClick={handleEncryptClick} disabled={!canEncrypt}>
            {isEncrypting ? "Encrypting…" : "Encrypt"}
          </button>
        </div>
      </div>

      {/* File list (full width) */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        <div style={{ flex: 1, padding: 16, overflowY: "auto" }}>
          <FileListPanel
            files={selectedFiles}
            onFilesChange={(newFiles) => { setSelectedFiles(newFiles); resetStatuses(newFiles); }}
            label="Source Files"
            fileStatuses={fileStatuses}
          />
        </div>
        {showProgress && (
          <div style={{ padding: "0 16px 16px", flexShrink: 0 }}>
            <EncryptProgressPanel progress={progress} result={result} isRunning={isEncrypting} />
          </div>
        )}
      </div>
    </div>
  );
}
