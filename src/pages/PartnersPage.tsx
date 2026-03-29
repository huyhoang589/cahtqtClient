import { useCallback, useEffect, useState } from "react";
import AddRecipientDialog from "../components/add-recipient-dialog";
import GroupListSidebar from "../components/group-list-sidebar";
import PartnerDetailPanel from "../components/partner-detail-panel";
import RecipientTable from "../components/recipient-table";
import { listPartners, listPartnerMembers, getAppSettings } from "../lib/tauri-api";
import { useAppContext } from "../contexts/app-context";
import type { Partner, PartnerMember } from "../types";

export default function PartnersPage() {
  const [groups, setGroups] = useState<Partner[]>([]);
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [recipients, setRecipients] = useState<PartnerMember[]>([]);
  const [selectedMemberId, setSelectedMemberId] = useState<string | null>(null);
  const [showAddRecipient, setShowAddRecipient] = useState(false);
  const [loading, setLoading] = useState(false);
  const [desktopPath, setDesktopPath] = useState("");

  const { outputDataDir } = useAppContext().settings;

  // Get Rust-resolved desktop path as fallback when outputDataDir is empty
  useEffect(() => {
    getAppSettings()
      .then((s) => setDesktopPath(s.output_data_dir || ""))
      .catch(() => {});
  }, []);

  const loadRecipients = useCallback(async (groupId: string) => {
    setLoading(true);
    try {
      setRecipients(await listPartnerMembers(groupId));
    } catch {
      setRecipients([]);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadGroups = useCallback(async () => {
    try {
      const gs = await listPartners();
      setGroups(gs);
      if (gs.length > 0 && !selectedGroupId) setSelectedGroupId(gs[0].id);
    } catch { /* ignore */ }
  }, [selectedGroupId]);

  useEffect(() => { loadGroups(); }, [loadGroups]);
  useEffect(() => {
    setSelectedMemberId(null);
    if (selectedGroupId) loadRecipients(selectedGroupId);
    else setRecipients([]);
  }, [selectedGroupId, loadRecipients]);

  const handleGroupRefresh = async () => {
    await loadGroups();
    if (selectedGroupId) await loadRecipients(selectedGroupId);
  };

  const selectedGroup = groups.find((g) => g.id === selectedGroupId) ?? null;
  const selectedMember = recipients.find((r) => r.id === selectedMemberId) ?? null;
  const effectiveBaseDir = outputDataDir || desktopPath;

  return (
    <div style={{ height: "100%", display: "flex", margin: "-24px" }}>
      <GroupListSidebar
        groups={groups}
        selectedId={selectedGroupId}
        onSelect={setSelectedGroupId}
        onRefresh={handleGroupRefresh}
      />

      <div style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column" }}>
        {selectedGroup ? (
          <>
            <div style={{ padding: "12px 16px", borderBottom: "1px solid var(--cahtqt-border-light)", background: "var(--cahtqt-bg-content)" }}>
              <h3 style={{ fontSize: "var(--cahtqt-font-size-lg)", color: "var(--cahtqt-text-on-light)" }}>
                {selectedGroup.name}
              </h3>
            </div>
            {loading ? (
              <div style={{ padding: 24, color: "var(--cahtqt-text-muted)" }}>Loading…</div>
            ) : (
              <RecipientTable
                recipients={recipients}
                onRefresh={() => loadRecipients(selectedGroupId!)}
                onAddRecipient={() => setShowAddRecipient(true)}
                onRowSelect={setSelectedMemberId}
                selectedMemberId={selectedMemberId}
                partnerName={selectedGroup.name}
                outputDataDir={effectiveBaseDir}
                desktopPath={desktopPath}
              />
            )}
          </>
        ) : (
          <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--cahtqt-text-muted)", fontSize: "var(--cahtqt-font-size-md)" }}>
            Select or create a partner to manage members
          </div>
        )}
      </div>

      <PartnerDetailPanel
        partner={selectedGroup}
        member={selectedMember}
        members={recipients}
      />

      {showAddRecipient && selectedGroupId && (
        <AddRecipientDialog
          partnerId={selectedGroupId}
          onAdded={() => {
            setShowAddRecipient(false);
            loadRecipients(selectedGroupId);
            loadGroups();
          }}
          onCancel={() => setShowAddRecipient(false)}
        />
      )}
    </div>
  );
}
