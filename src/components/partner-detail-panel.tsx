import type { Partner, PartnerMember } from "../types";
import CertExpiryBadge from "./cert-expiry-badge";

interface Props {
  partner: Partner | null;
  member: PartnerMember | null;
  members: PartnerMember[];
}

const formatDate = (ts: number) => new Date(ts * 1000).toLocaleDateString();

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", marginBottom: 8 }}>
      <span style={{ fontSize: "var(--cahtqt-font-size-xs)", color: "var(--cahtqt-text-muted)", marginBottom: 2 }}>
        {label}
      </span>
      <span style={{ fontSize: "var(--cahtqt-font-size-sm)", color: "var(--cahtqt-text-on-light)", wordBreak: "break-all" }}>
        {value}
      </span>
    </div>
  );
}

export default function PartnerDetailPanel({ partner, member, members }: Props) {
  if (!partner) {
    return (
      <div
        style={{
          width: 280,
          minWidth: 280,
          borderLeft: "1px solid var(--cahtqt-border-light)",
          background: "var(--cahtqt-bg-surface-subtle)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "var(--cahtqt-text-muted)",
          fontSize: "var(--cahtqt-font-size-sm)",
          padding: 16,
          textAlign: "center",
        }}
      >
        Select a partner to see details
      </div>
    );
  }

  return (
    <div
      style={{
        width: 280,
        minWidth: 280,
        borderLeft: "1px solid var(--cahtqt-border-light)",
        background: "var(--cahtqt-bg-surface-subtle)",
        display: "flex",
        flexDirection: "column",
        overflowY: "auto",
      }}
    >
      {/* Panel header */}
      <div
        style={{
          padding: "12px 16px",
          borderBottom: "1px solid var(--cahtqt-border-light)",
          background: "var(--cahtqt-bg-surface-muted)",
        }}
      >
        <span className="section-title" style={{ marginBottom: 0, color: "var(--cahtqt-text-on-light-2)" }}>
          {member ? "Member Details" : "Partner Details"}
        </span>
      </div>

      <div style={{ padding: 16 }}>
        {member ? (
          /* View 2: Member selected */
          <>
            <InfoRow label="Name" value={member.name} />
            <InfoRow label="Email" value={member.email ?? "—"} />
            <InfoRow label="Certificate CN" value={member.cert_cn} />
            {member.cert_org && <InfoRow label="Organization" value={member.cert_org} />}
            <InfoRow label="Serial Number" value={member.cert_serial} />
            <InfoRow label="Valid From" value={formatDate(member.cert_valid_from)} />
            <div style={{ marginBottom: 8 }}>
              <span style={{ fontSize: "var(--cahtqt-font-size-xs)", color: "var(--cahtqt-text-muted)", marginBottom: 2, display: "block" }}>
                Valid Until
              </span>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span style={{ fontSize: "var(--cahtqt-font-size-sm)", color: "var(--cahtqt-text-on-light)" }}>
                  {formatDate(member.cert_valid_to)}
                </span>
                <CertExpiryBadge valid_to={member.cert_valid_to} />
              </div>
            </div>
          </>
        ) : (
          /* View 1: Partner selected, no member */
          <>
            <div style={{ marginBottom: 16 }}>
              <div style={{ fontSize: "var(--cahtqt-font-size-lg)", fontWeight: "var(--cahtqt-font-weight-semibold)", color: "var(--cahtqt-text-on-light)", marginBottom: 4 }}>
                {partner.name}
              </div>
              <span className="badge badge-default">
                {members.length} member{members.length !== 1 ? "s" : ""}
              </span>
            </div>
            <InfoRow label="Created" value={formatDate(partner.created_at)} />
            {members.length > 0 && (
              <div>
                <span style={{ fontSize: "var(--cahtqt-font-size-xs)", color: "var(--cahtqt-text-muted)", marginBottom: 6, display: "block" }}>
                  Members
                </span>
                {members.slice(0, 5).map((m) => (
                  <div
                    key={m.id}
                    style={{
                      padding: "4px 0",
                      borderBottom: "1px solid var(--cahtqt-border-subtle)",
                      fontSize: "var(--cahtqt-font-size-sm)",
                      color: "var(--cahtqt-text-on-light-2)",
                    }}
                  >
                    <div>{m.name}</div>
                    {m.email && (
                      <div style={{ fontSize: "var(--cahtqt-font-size-xs)", color: "var(--cahtqt-text-muted)" }}>{m.email}</div>
                    )}
                  </div>
                ))}
                {members.length > 5 && (
                  <div style={{ fontSize: "var(--cahtqt-font-size-xs)", color: "var(--cahtqt-text-muted)", marginTop: 4 }}>
                    …and {members.length - 5} more
                  </div>
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
