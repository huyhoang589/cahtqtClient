import { useCallback, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { DecryptProgress, DecryptResult } from "../types";
import { decryptBatch } from "../lib/tauri-api";

export function useDecrypt() {
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [isDecrypting, setIsDecrypting] = useState(false);
  const [progress, setProgress] = useState<DecryptProgress[]>([]);
  const [result, setResult] = useState<DecryptResult | null>(null);

  /// PIN read from AppState.token_login — must be LoggedIn before calling
  const startDecrypt = useCallback(async (outputDir?: string | null) => {
    if (selectedFiles.length === 0) return;

    setIsDecrypting(true);
    setProgress([]);
    setResult(null);

    const unlisten = await listen<DecryptProgress>("decrypt-progress", (event) => {
      setProgress((prev) => [...prev.slice(-100), event.payload]);
    });

    try {
      // Partner name not used in client edition — pass empty string
      const res = await decryptBatch(selectedFiles, "", outputDir);
      setResult(res);
    } catch (e) {
      setResult({ total: 0, success_count: 0, error_count: 1, errors: [String(e)] });
    } finally {
      unlisten();
      setIsDecrypting(false);
    }
  }, [selectedFiles]);

  const reset = useCallback(() => {
    setProgress([]);
    setResult(null);
  }, []);

  return {
    selectedFiles,
    setSelectedFiles,
    isDecrypting,
    progress,
    result,
    startDecrypt,
    reset,
  };
}
