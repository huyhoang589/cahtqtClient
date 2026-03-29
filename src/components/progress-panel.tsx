import type { EncryptProgress, DecryptProgress } from "../types";

// Union type of supported progress shapes (both share the same field names)
type AnyProgress = EncryptProgress | DecryptProgress;

// Format a single progress entry into a human-readable log line
function formatLine(p: AnyProgress, processingVerb: string): string {
  if (p.status === "success") return `${p.file_name}: success`;
  if (p.status === "processing") return `${p.file_name}: ${processingVerb}`;
  // "warning" (encrypt only) — DecryptProgress has no warning status
  if ("status" in p && (p as EncryptProgress).status === "warning") {
    return `${p.file_name}: warning ${p.error ?? ""}`;
  }
  // "error"
  if (p.error) {
    const m = p.error.match(/^\[(-?\d+)\]\s*/);
    return m
      ? `${p.file_name}: failed (${m[1]}) ${p.error.slice(m[0].length)}`
      : `${p.file_name}: failed ${p.error}`;
  }
  return `${p.file_name}: ${p.status}`;
}

function statusIcon(status: string): string {
  if (status === "processing") return "⋯";
  if (status === "success") return "✓";
  if (status === "warning") return "⚠";
  return "✗";
}

function statusColor(status: string): string {
  if (status === "success") return "var(--cahtqt-text-log-success)";
  if (status === "warning") return "var(--cahtqt-text-log-warning)";
  if (status === "error") return "var(--cahtqt-text-log-error)";
  return "var(--cahtqt-text-log-info)";
}

interface ProgressResult {
  success_count: number;
  error_count: number;
}

interface Props<T extends AnyProgress> {
  progress: T[];
  result: ProgressResult | null;
  isRunning: boolean;
  /** Verb shown while a file is processing, e.g. "encrypting…" or "decrypting…" */
  processingVerb: string;
  /** Whether to show the currently-processing file name inline (encrypt does, decrypt doesn't) */
  showProcessingFileName?: boolean;
}

export default function ProgressPanel<T extends AnyProgress>({
  progress,
  result,
  isRunning,
  processingVerb,
  showProcessingFileName = false,
}: Props<T>) {
  const last = progress[progress.length - 1];
  const pct = last && last.total > 0 ? Math.round((last.current / last.total) * 100) : 0;

  // Dedup by file_path — later events overwrite earlier; reverse for newest-first
  const dedupedMap = new Map<string, T>();
  for (const p of progress) {
    dedupedMap.set(p.file_path, p);
  }
  const displayLines = [...dedupedMap.values()].reverse();

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <span className="section-title" style={{ color: "var(--cahtqt-text-muted)" }}>Progress</span>

      {/* Progress bar */}
      <div className="progress-track">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      {last && (
        <div style={{ fontSize: "var(--cahtqt-font-size-sm)", color: "var(--cahtqt-text-muted)" }}>
          {last.current} / {last.total} ({pct}%)
          {showProcessingFileName && isRunning && last.status === "processing" && (
            <span style={{ marginLeft: 8 }}>{last.file_name}</span>
          )}
        </div>
      )}

      {/* Live progress log with sticky summary header */}
      <div
        style={{
          background: "var(--cahtqt-bg-log)",
          border: "1px solid var(--cahtqt-border-dark)",
          borderRadius: "var(--cahtqt-radius-sm)",
          height: 120,
          overflowY: "auto",
          padding: "0",
          position: "relative",
        }}
      >
        {/* Sticky summary header — counts only */}
        {result && (
          <div style={{
            position: "sticky", top: 0, zIndex: 1,
            background: "var(--cahtqt-bg-log)",
            padding: "3px 10px",
            borderBottom: "1px solid var(--cahtqt-border-dark)",
            display: "flex", gap: 12,
          }}>
            <span style={{ color: "var(--cahtqt-color-success)", fontSize: "var(--cahtqt-font-size-xs)" }}>✓ {result.success_count} success</span>
            {result.error_count > 0 && (
              <span style={{ color: "var(--cahtqt-color-danger)", fontSize: "var(--cahtqt-font-size-xs)" }}>✗ {result.error_count} failed</span>
            )}
          </div>
        )}

        {displayLines.length === 0 ? (
          <div style={{ padding: 16, color: "var(--cahtqt-text-secondary)", textAlign: "center", fontSize: "var(--cahtqt-font-size-sm)" }}>
            No operations yet
          </div>
        ) : (
          displayLines.map((p) => (
            <div
              key={p.file_path}
              style={{
                padding: "2px 10px",
                fontSize: "var(--cahtqt-font-size-xs)",
                fontFamily: "var(--cahtqt-font-mono)",
                color: statusColor(p.status),
              }}
            >
              {statusIcon(p.status)}{" "}
              {formatLine(p, processingVerb)}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
