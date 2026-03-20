import type { EncryptProgress, EncryptResult } from "../types";

// Format a single progress entry into a human-readable log line
function formatLine(p: EncryptProgress): string {
  if (p.status === "success") return `${p.file_name}: success`;
  if (p.status === "warning") return `${p.file_name}: warning ${p.error ?? ""}`;
  if (p.status === "processing") return `${p.file_name}: encrypting…`;
  // "error"
  if (p.error) {
    const m = p.error.match(/^\[(-?\d+)\]\s*/);
    return m
      ? `${p.file_name}: failed (${m[1]}) ${p.error.slice(m[0].length)}`
      : `${p.file_name}: failed ${p.error}`;
  }
  return `${p.file_name}: ${p.status}`;
}

interface Props {
  progress: EncryptProgress[];
  result: EncryptResult | null;
  isRunning: boolean;
}

export default function EncryptProgressPanel({ progress, result, isRunning }: Props) {
  const last = progress[progress.length - 1];
  const pct = last && last.total > 0 ? Math.round((last.current / last.total) * 100) : 0;

  // Dedup by file_path — later events overwrite earlier; reverse for newest-first
  const dedupedMap = new Map<string, EncryptProgress>();
  for (const p of progress) {
    dedupedMap.set(p.file_path, p);
  }
  const displayLines = [...dedupedMap.values()].reverse();

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <span className="section-title" style={{ color: "var(--color-text-muted-light)" }}>Progress</span>

      {/* Progress bar */}
      <div className="progress-track">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      {last && (
        <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-muted-light)" }}>
          {last.current} / {last.total} ({pct}%)
          {isRunning && last.status === "processing" && (
            <span style={{ marginLeft: 8 }}>{last.file_name}</span>
          )}
        </div>
      )}

      {/* Live progress log with sticky summary header */}
      <div
        style={{
          background: "var(--color-bg-log)",
          border: "1px solid var(--color-border-dark)",
          borderRadius: "var(--radius-sm)",
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
            background: "var(--color-bg-log)",
            padding: "3px 10px",
            borderBottom: "1px solid var(--color-border-dark)",
            display: "flex", gap: 12,
          }}>
            <span style={{ color: "var(--color-accent-success)", fontSize: "var(--font-size-xs)" }}>✓ {result.success_count} success</span>
            {result.error_count > 0 && (
              <span style={{ color: "var(--color-accent-danger)", fontSize: "var(--font-size-xs)" }}>✗ {result.error_count} failed</span>
            )}
          </div>
        )}

        {displayLines.length === 0 ? (
          <div style={{ padding: 16, color: "var(--color-text-secondary)", textAlign: "center", fontSize: "var(--font-size-sm)" }}>
            No operations yet
          </div>
        ) : (
          displayLines.map((p) => (
            <div
              key={p.file_path}
              style={{
                padding: "2px 10px",
                fontSize: "var(--font-size-xs)",
                fontFamily: "var(--font-mono)",
                color:
                  p.status === "success" ? "var(--color-text-log-success)" :
                  p.status === "warning" ? "var(--color-text-log-warning)" :
                  p.status === "error"   ? "var(--color-text-log-error)" :
                  "var(--color-text-log-info)",
              }}
            >
              {p.status === "processing" ? "⋯" :
               p.status === "success" ? "✓" :
               p.status === "warning" ? "⚠" : "✗"}{" "}
              {formatLine(p)}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
