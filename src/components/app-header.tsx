import { Shield } from "lucide-react";
import { useTokenStatus } from "../hooks/use-token-status";

export default function AppHeader() {
  const { dll_found, status, cert_cn } = useTokenStatus();

  // 4-state dot: no dll → red, disconnected → gray, connected → orange, logged_in → green
  const dotClass = !dll_found
    ? "status-dot status-dot-disconnected"
    : status === "logged_in"
    ? "status-dot status-dot-connected"
    : status === "connected"
    ? "status-dot status-dot-warning"
    : "status-dot status-dot-idle";

  const label = !dll_found
    ? "htqt lib not found"
    : status === "logged_in"
    ? "Token logged in"
    : status === "connected"
    ? "Token connected"
    : "Token not found";

  return (
    <header style={{
      height: 56,
      flexShrink: 0,
      background: "#ffffff",
      borderBottom: "1px solid var(--color-border-light)",
      boxShadow: "0 1px 4px rgba(0,0,0,0.06)",
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      padding: "0 20px",
    }}>
      {/* Left: gradient logo tile + stacked app name */}
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <div style={{
          width: 32,
          height: 32,
          borderRadius: 8,
          background: "linear-gradient(135deg, #00c6e0, #0098b5)",
          boxShadow: "0 2px 8px rgba(0,198,224,0.35)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexShrink: 0,
        }}>
          <Shield size={18} color="#ffffff" />
        </div>
        <div>
          <div style={{
            fontSize: "var(--font-size-base)",
            fontWeight: "var(--font-weight-bold)",
            color: "var(--color-text-on-light)",
            lineHeight: "var(--line-height-tight)",
          }}>
            CAHTQT PKI
          </div>
          <div style={{
            fontSize: "var(--font-size-xs)",
            color: "var(--color-text-muted-light)",
            lineHeight: "var(--line-height-tight)",
          }}>
            PKI Encryption
          </div>
        </div>
      </div>

      {/* Right: token status pill */}
      <div style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        padding: "4px 10px",
        borderRadius: "var(--radius-full)",
        background: status === "logged_in"
          ? "rgba(0,200,83,0.08)"
          : "rgba(0,0,0,0.04)",
        border: `1px solid ${status === "logged_in"
          ? "rgba(0,200,83,0.2)"
          : "var(--color-border-light)"}`,
        fontSize: "var(--font-size-sm)",
        color: "var(--color-text-on-light)",
      }}>
        <span className={dotClass} />
        <span>{label}</span>
        {status === "logged_in" && cert_cn && (
          <span className="cert-cn-badge">{cert_cn}</span>
        )}
      </div>
    </header>
  );
}
