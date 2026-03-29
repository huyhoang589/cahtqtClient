import type { PartnerMember } from "../types";
import CertExpiryBadge from "./cert-expiry-badge";

interface Props {
  member: PartnerMember;
  checked: boolean;
  onToggle: () => void;
}

const formatDateShort = (ts: number) => {
  const d = new Date(ts * 1000);
  return `${String(d.getDate()).padStart(2, "0")}/${String(d.getMonth() + 1).padStart(2, "0")}/${d.getFullYear()}`;
};

export default function PartnerMemberRow({ member, checked, onToggle }: Props) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        padding: "5px 10px 5px 24px",
        gap: 8,
        borderBottom: "1px solid var(--cahtqt-border-light)",
        cursor: "pointer",
        background: checked ? "var(--cahtqt-bg-selected-strong)" : "var(--cahtqt-bg-table-row)",
      }}
      onClick={onToggle}
    >
      <input
        type="checkbox"
        checked={checked}
        readOnly
        style={{ accentColor: "var(--cahtqt-color-primary)", flexShrink: 0, width: 16, height: 16 }}
      />
      {/* Name */}
      <span style={{ flex: 2, fontSize: "var(--cahtqt-font-size-base)", color: "var(--cahtqt-text-on-light)", fontWeight: 500 }}>
        {member.name}
      </span>
      {/* Organization */}
      <span style={{ flex: 2, fontSize: 11, color: "var(--cahtqt-text-on-light-2)" }}>
        {member.cert_org ?? "—"}
      </span>
      {/* Expires */}
      <span style={{ display: "flex", alignItems: "center", gap: 4, flexShrink: 0 }}>
        <span style={{ fontSize: 11, color: "var(--cahtqt-text-muted)" }}>{formatDateShort(member.cert_valid_to)}</span>
        <CertExpiryBadge valid_to={member.cert_valid_to} />
      </span>
    </div>
  );
}
