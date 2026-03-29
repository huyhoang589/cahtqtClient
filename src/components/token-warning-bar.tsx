import { useAppContext } from "../contexts/app-context";

interface Props {
  onLogin: () => void;
}

/// Orange warning bar shown when token is not logged in.
/// Returns null when status === "logged_in" (no warning needed).
/// Shared by EncryptPage and DecryptPage.
export default function TokenWarningBar({ onLogin }: Props) {
  const { status } = useAppContext().tokenStatus;

  if (status === "logged_in") return null;

  return (
    <div
      style={{
        background: "var(--cahtqt-bg-dialog-token)",
        borderBottom: "1px solid var(--cahtqt-border-dialog-token)",
        padding: "8px 20px",
        display: "flex",
        alignItems: "center",
        gap: 12,
        flexShrink: 0,
        color: "var(--cahtqt-text-white)",
      }}
    >
      <span style={{ fontSize: "var(--cahtqt-font-size-sm)", flex: 1 }}>
        Token not logged in — login required to encrypt or decrypt
      </span>
      <button
        onClick={onLogin}
        style={{
          height: 28,
          padding: "0 12px",
          background: "rgba(0,0,0,0.3)",
          color: "var(--cahtqt-text-white)",
          border: "1px solid rgba(255,255,255,0.3)",
          borderRadius: "var(--cahtqt-radius-sm)",
          fontSize: "var(--cahtqt-font-size-sm)",
          cursor: "pointer",
          whiteSpace: "nowrap",
        }}
      >
        Login Now
      </button>
    </div>
  );
}
