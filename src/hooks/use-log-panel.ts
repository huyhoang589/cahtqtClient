import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { AppLogPayload, EncryptProgress } from "../types";

/** Extract a human-readable error message from an encrypt error string. */
function formatErrorMsg(e: unknown): string {
  const err = e as string;
  const m = err.match(/^\[(-?\d+)\]\s*/);
  return m ? `(${m[1]}) ${err.slice(m[0].length)}` : err;
}



export interface LogEntry {
  id: string;
  timestamp: string;
  level: "info" | "success" | "warning" | "error";
  message: string;
}

const MAX_ENTRIES = 200;

// D: suppress noisy settings-persistence logs from the log panel
const FILTERED_PATTERNS = [
  "encrypt_panel_split_ratio",
  "decrypt_panel_split_ratio",
];

export function useLogPanel() {
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const idRef = useRef(0);
  const unlistenRefs = useRef<Array<(() => void) | null>>([null, null]);

  const addEntry = useCallback((level: LogEntry["level"], message: string, ts?: string) => {
    if (FILTERED_PATTERNS.some((p) => message.includes(p))) return;  // D: drop noisy settings logs
    const timestamp = ts ?? new Date().toTimeString().slice(0, 8);
    setEntries((prev) => {
      const next = [...prev, { id: String(++idRef.current), timestamp, level, message }];
      return next.length > MAX_ENTRIES ? next.slice(-MAX_ENTRIES) : next;
    });
  }, []);

  const clearEntries = useCallback(() => setEntries([]), []);

  useEffect(() => {
    let cancelled = false;

    (async () => {
      const unlistenEncrypt = await listen<EncryptProgress>("encrypt-progress", (e) => {
        const p = e.payload;
        if (typeof p.status !== "string") return;  // filter raw DLL ProgressPayload events (numeric status, no file info)
        if (p.status === "processing") return;  // suppress noisy in-progress events
        const lvl = p.status === "error" ? "error" : p.status === "success" ? "success" : "info";
        const line = p.status === "success"
          ? `[Encrypt] ${p.file_name}: success`
          : p.error
            ? `[Encrypt] ${p.file_name}: failed ${formatErrorMsg(p.error)}`
            : `[Encrypt] ${p.file_name}: ${p.status}`;
        addEntry(lvl, line);
      });
      const unlistenAppLog = await listen<AppLogPayload>("app_log", (e) => {
        addEntry(e.payload.level, e.payload.message, e.payload.timestamp);
      });

      if (cancelled) {
        unlistenEncrypt();
        unlistenAppLog();
      } else {
        unlistenRefs.current[0] = unlistenEncrypt;
        unlistenRefs.current[1] = unlistenAppLog;
      }
    })();

    return () => {
      cancelled = true;
      unlistenRefs.current.forEach((fn) => fn?.());
      unlistenRefs.current = [null, null];
    };
  }, [addEntry]);

  return { entries, addEntry, clearEntries };
}
