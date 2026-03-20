import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { AppLogPayload, EncryptProgress } from "../types";

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
    const promises = [
      listen<EncryptProgress>("encrypt-progress", (e) => {
        const p = e.payload;
        if (typeof p.status !== "string") return;  // filter raw DLL ProgressPayload events (numeric status, no file info)
        if (p.status === "processing") return;  // suppress noisy in-progress events
        const lvl = p.status === "error" ? "error" : p.status === "success" ? "success" : "info";
        const line = p.status === "success"
          ? `[Encrypt] ${p.file_name}: success`
          : p.error
            ? (() => {
                const m = p.error!.match(/^\[(-?\d+)\]\s*/);
                return m
                  ? `[Encrypt] ${p.file_name}: failed (${m[1]}) ${p.error!.slice(m[0].length)}`
                  : `[Encrypt] ${p.file_name}: failed ${p.error}`;
              })()
            : `[Encrypt] ${p.file_name}: ${p.status}`;
        addEntry(lvl, line);
      }),
      listen<AppLogPayload>("app_log", (e) => {
        addEntry(e.payload.level, e.payload.message, e.payload.timestamp);
      }),
    ];
    return () => {
      Promise.all(promises).then((fns) => fns.forEach((fn) => fn()));
    };
  }, [addEntry]);

  return { entries, addEntry, clearEntries };
}
