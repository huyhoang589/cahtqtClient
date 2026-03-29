import type { CertificateInfo } from "../../../types";

interface Props {
  cert: CertificateInfo;
  onExport: () => Promise<void>;
}

export default function CertificateDetail({ cert, onExport }: Props) {
  const rows: [string, string][] = [
    ["Full Name",    cert.subject_cn],
    ["Email",        cert.subject_email || "—"],
    ["Organization", cert.subject_org || "—"],
    ["Unit",         cert.subject_unit || "—"],
    ["Issued By",    cert.issuer_cn || "—"],
    ["Serial No.",   cert.serial_number],
    ["Valid From",   cert.valid_from],
    ["Valid Until",  cert.valid_until],
    ["Fingerprint",  cert.fingerprint_sha1],
  ];

  const monoFields = new Set(["Serial No.", "Fingerprint"]);

  return (
    <div
      style={{
        marginTop: 12,
        padding: "12px 14px",
        background: "var(--cahtqt-bg-table-row)",
        border: "1px solid var(--cahtqt-border-light)",
        borderRadius: "var(--cahtqt-radius-sm)",
        fontSize: "var(--cahtqt-font-size-sm)",
      }}
    >
      <div
        style={{
          fontWeight: "var(--cahtqt-font-weight-semibold)",
          marginBottom: 8,
          color: "var(--cahtqt-text-on-light)",
        }}
      >
        Certificate Details
      </div>

      {rows.map(([label, value]) => (
        <div key={label} style={{ display: "flex", gap: 12, marginBottom: 4 }}>
          <span
            style={{
              color: "var(--cahtqt-text-muted)",
              minWidth: 100,
              flexShrink: 0,
            }}
          >
            {label}:
          </span>
          <span
            style={{
              color: "var(--cahtqt-text-on-light)",
              wordBreak: "break-all",
              fontFamily: monoFields.has(label) ? "var(--cahtqt-font-mono)" : undefined,
              fontSize: label === "Fingerprint" ? "var(--cahtqt-font-size-xs)" : undefined,
            }}
          >
            {value}
          </span>
        </div>
      ))}

      {!cert.is_expired && (
        <button className="btn btn-success" onClick={onExport} style={{ marginTop: 12 }}>
          ✓ Use as Sender Certificate
        </button>
      )}
    </div>
  );
}
