import type { TokenCertEntry } from "../../../types";
import CertificateDetail from "./CertificateDetail";

interface Props {
  entries: TokenCertEntry[];
  selectedCertId: string | null;
  senderCertSerial: string | null;
  onSelect: (certObjectId: string, slotId: number) => void;
  onExport: () => Promise<void>;
}

export default function CertificateTable({
  entries,
  selectedCertId,
  senderCertSerial,
  onSelect,
  onExport,
}: Props) {
  if (entries.length === 0) {
    return (
      <div
        style={{
          color: "var(--cahtqt-text-muted)",
          fontSize: "var(--cahtqt-font-size-sm)",
          padding: "12px 0",
        }}
      >
        No certificates found on token.
      </div>
    );
  }

  const selectedEntry = entries.find(
    (e) => e.certificate.object_id === selectedCertId,
  );

  return (
    <div>
      <div className="table-container">
        <table>
          <thead>
            <tr>
              <th>Common Name</th>
              <th>Organization</th>
              <th>Valid Until</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {entries.map(({ slot_id, certificate: cert }) => {
              const isSelected = cert.object_id === selectedCertId;
              const isSender = cert.serial_number === senderCertSerial;

              const rowStyle: React.CSSProperties = isSelected
                ? { borderLeft: "3px solid var(--cahtqt-color-primary)", background: "var(--cahtqt-bg-selected-accent)" }
                : {};
              const mutedStyle: React.CSSProperties = cert.is_expired
                ? { color: "var(--cahtqt-text-muted)" }
                : {};

              return (
                <tr
                  key={cert.object_id}
                  style={{ cursor: "pointer", ...rowStyle }}
                  onClick={() => onSelect(cert.object_id, slot_id)}
                >
                  <td
                    style={{
                      ...mutedStyle,
                      fontWeight: isSender ? "var(--cahtqt-font-weight-semibold)" : undefined,
                    }}
                  >
                    {isSender && (
                      <span
                        style={{
                          color: "var(--cahtqt-color-success)",
                          marginRight: 4,
                        }}
                      >
                        ✓
                      </span>
                    )}
                    {cert.subject_cn}
                  </td>
                  <td style={mutedStyle}>{cert.subject_org || "—"}</td>
                  <td style={mutedStyle}>{cert.valid_until}</td>
                  <td>
                    {cert.is_expired ? (
                      <span
                        style={{
                          color: "var(--cahtqt-color-danger)",
                          fontSize: "var(--cahtqt-font-size-xs)",
                        }}
                      >
                        ✗ Expired
                      </span>
                    ) : (
                      <span
                        style={{
                          color: "var(--cahtqt-color-success)",
                          fontSize: "var(--cahtqt-font-size-xs)",
                        }}
                      >
                        ✓ Valid
                      </span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {selectedEntry && (
        <CertificateDetail cert={selectedEntry.certificate} onExport={onExport} />
      )}
    </div>
  );
}
