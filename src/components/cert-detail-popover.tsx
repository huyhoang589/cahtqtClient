import * as Popover from "@radix-ui/react-popover";
import type { PartnerMember } from "../types";

interface Props {
  recipient: PartnerMember;
}

export default function CertDetailPopover({ recipient: r }: Props) {
  const shortSerial =
    r.cert_serial.length > 12 ? `${r.cert_serial.slice(0, 12)}…` : r.cert_serial;

  const rows: [string, string][] = [
    ["CN", r.cert_cn],
    ["Email", r.email ?? "—"],
    ["Serial", r.cert_serial],
    ["Valid From", new Date(r.cert_valid_from * 1000).toLocaleDateString()],
    ["Valid To", new Date(r.cert_valid_to * 1000).toLocaleDateString()],
  ];

  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <span
          style={{
            fontSize: "var(--font-size-xs)",
            fontFamily: "var(--font-mono)",
            cursor: "pointer",
            color: "var(--color-text-link)",
            textDecoration: "underline dotted",
          }}
        >
          {shortSerial}
        </span>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          className="popover-content"
          sideOffset={8}
          align="start"
        >
          <div
            style={{
              marginBottom: 8,
              fontWeight: "var(--font-weight-semibold)",
              fontSize: "var(--font-size-sm)",
            }}
          >
            Certificate Details
          </div>
          {rows.map(([label, value]) => (
            <div
              key={label}
              style={{ display: "flex", gap: 8, marginBottom: 4, fontSize: "var(--font-size-sm)" }}
            >
              <span style={{ color: "var(--color-text-secondary)", minWidth: 80 }}>{label}:</span>
              <span
                style={{
                  fontFamily: label === "Serial" ? "var(--font-mono)" : undefined,
                  wordBreak: "break-all",
                }}
              >
                {value}
              </span>
            </div>
          ))}
          <Popover.Arrow className="popover-arrow" />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
