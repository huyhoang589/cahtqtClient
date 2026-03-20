import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { FolderOpen } from "lucide-react";
import DecryptProgressPanel from "../components/decrypt-progress-panel";
import FileListPanel from "../components/file-list-panel";
import TokenWarningBar from "../components/token-warning-bar";
import { useDecrypt } from "../hooks/use-decrypt";
import { useTokenStatus } from "../hooks/use-token-status";
import { useSettingsStore } from "../hooks/use-settings-store";
import { useFileStatuses } from "../hooks/use-file-statuses";
import { getAppSettings, openFolder, selectAllFiles } from "../lib/tauri-api";

type Step = "idle" | "running" | "done";

export default function DecryptPage() {
  const {
    selectedFiles, setSelectedFiles,
    isDecrypting, progress, result,
    startDecrypt, reset,
  } = useDecrypt();

  const navigate = useNavigate();
  const { dll_found, status: tokenStatus } = useTokenStatus();
  const { outputDataDir } = useSettingsStore();
  const { fileStatuses, resetStatuses } = useFileStatuses("decrypt-progress", "decrypt");
  const [step, setStep] = useState<Step>("idle");

  const canDecrypt =
    dll_found && tokenStatus === "logged_in" &&
    selectedFiles.length > 0 && !isDecrypting;

  const handleDecryptClick = async () => {
    if (!canDecrypt) return;
    if (step === "done") {
      const ok = window.confirm(
        "Output files from the previous batch may already exist. Start a new batch? (Existing files will be overwritten.)"
      );
      if (!ok) return;
      handleReset();
    }
    setStep("running");
    resetStatuses(selectedFiles);
    const outputDir = outputDataDir
      ? `${outputDataDir}/SF/DECRYPT/`  // flat — no partner subfolder
      : null;
    await startDecrypt(outputDir);
    setStep("done");
  };

  const handleReset = () => { reset(); setStep("idle"); };

  const handleSelectFiles = async () => {
    const files = await selectAllFiles();
    if (files) setSelectedFiles(files);
  };

  const handleOpenFolder = async () => {
    try {
      const settings = await getAppSettings();
      await openFolder(`${settings.output_data_dir}/SF/DECRYPT`);
    } catch {}
  };

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column" }}>
      <TokenWarningBar onLogin={() => navigate("/settings")} />

      {/* Header */}
      <div style={{ padding: "16px 20px 12px", borderBottom: "1px solid var(--color-border-light)", display: "flex", justifyContent: "space-between", alignItems: "center", flexShrink: 0 }}>
        <h2 style={{ fontSize: "var(--font-size-xl)", color: "var(--color-text-on-light)" }}>Decrypt Files</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-ghost" onClick={handleOpenFolder} title="Open output folder" style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <FolderOpen size={14} /> Open Folder
          </button>
          <button className="primary" onClick={handleDecryptClick} disabled={!canDecrypt}>
            {isDecrypting ? "Decrypting…" : "Decrypt"}
          </button>
        </div>
      </div>

      {/* File list (full width) */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
        <div style={{ flex: 1, padding: 16, overflowY: "auto" }}>
          <FileListPanel
            files={selectedFiles}
            onFilesChange={(newFiles) => { setSelectedFiles(newFiles); resetStatuses(newFiles); }}
            label="Encrypted Files (.sf)"
            fileStatuses={fileStatuses}
          />
        </div>
        {(step !== "idle" || result !== null) && (
          <div style={{ padding: "0 16px 16px", flexShrink: 0 }}>
            <DecryptProgressPanel progress={progress} result={result} isRunning={isDecrypting} />
          </div>
        )}
      </div>
    </div>
  );
}
