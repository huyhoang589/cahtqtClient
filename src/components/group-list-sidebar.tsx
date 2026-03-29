import { useState } from "react";
import { Plus, X, Pencil } from "lucide-react";
import type { Partner } from "../types";
import CreateGroupDialog from "./create-group-dialog";
import { createPartner, deletePartner, renamePartner } from "../lib/tauri-api";

interface Props {
  groups: Partner[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onRefresh: () => void;
}

export default function GroupListSidebar({ groups, selectedId, onSelect, onRefresh }: Props) {
  const [showCreate, setShowCreate] = useState(false);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const handleCreate = async (name: string) => {
    await createPartner(name);
    onRefresh();
    setShowCreate(false);
  };

  const startRename = (g: Partner) => {
    setRenamingId(g.id);
    setRenameValue(g.name);
  };

  const submitRename = async (id: string) => {
    if (renameValue.trim()) {
      await renamePartner(id, renameValue.trim());
      onRefresh();
    }
    setRenamingId(null);
  };

  const handleDelete = async (id: string, name: string) => {
    if (!window.confirm(`Delete partner "${name}" and all its members?`)) return;
    await deletePartner(id);
    onRefresh();
  };

  return (
    <div
      style={{
        width: 200,
        minWidth: 200,
        background: "var(--cahtqt-bg-surface-subtle)",
        borderRight: "1px solid var(--cahtqt-text-on-light-2)",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "12px 12px 8px",
          borderBottom: "1px solid var(--cahtqt-text-on-light-2)",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <span
          className="section-header"
          style={{ marginBottom: 0, color: "var(--cahtqt-text-on-light)" }}
        >
          Partner List
        </span>
        <button
          className="btn-icon"
          onClick={() => setShowCreate(true)}
          title="Create partner"
          style={{ width: 24, height: 24, color: "var(--cahtqt-text-on-light)" }}
        >
          <Plus size={14} />
        </button>
      </div>

      {/* Partner list */}
      <div style={{ flex: 1, overflowY: "auto" }}>
        {groups.length === 0 ? (
          <div style={{ padding: 16, color: "var(--cahtqt-text-muted)", fontSize: "var(--cahtqt-font-size-sm)" }}>
            No partners yet
          </div>
        ) : (
          groups.map((g) => (
            <div
              key={g.id}
              style={{
                padding: "8px 12px",
                cursor: "pointer",
                background: selectedId === g.id ? "var(--cahtqt-bg-selected)" : "transparent",
                borderBottom: "1px solid var(--cahtqt-border-subtle)",
                borderLeft: selectedId === g.id ? "3px solid var(--cahtqt-color-primary-alt)" : "3px solid transparent",
                display: "flex",
                alignItems: "center",
                gap: 6,
              }}
              onClick={() => onSelect(g.id)}
              onMouseEnter={(e) => {
                if (selectedId !== g.id) {
                  (e.currentTarget as HTMLDivElement).style.background = "var(--cahtqt-bg-selected)";
                }
              }}
              onMouseLeave={(e) => {
                if (selectedId !== g.id) {
                  (e.currentTarget as HTMLDivElement).style.background = "transparent";
                }
              }}
            >
              {renamingId === g.id ? (
                <input
                  autoFocus
                  value={renameValue}
                  onChange={(e) => setRenameValue(e.target.value)}
                  onBlur={() => submitRename(g.id)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") submitRename(g.id);
                    if (e.key === "Escape") setRenamingId(null);
                  }}
                  onClick={(e) => e.stopPropagation()}
                  style={{ flex: 1, padding: "2px 4px", height: 24 }}
                />
              ) : (
                <>
                  <span style={{ flex: 1, fontSize: "var(--cahtqt-font-size-base)", color: "var(--cahtqt-text-on-light)" }}>
                    {g.name}
                  </span>
                  <span className="badge badge-default">{g.member_count ?? 0}</span>
                  <button
                    className="btn-icon"
                    title="Rename"
                    onClick={(e) => { e.stopPropagation(); startRename(g); }}
                    style={{ width: 20, height: 20, color: "var(--cahtqt-text-on-light-2)" }}
                  >
                    <Pencil size={11} />
                  </button>
                  <button
                    className="btn-icon"
                    title="Delete"
                    onClick={(e) => { e.stopPropagation(); handleDelete(g.id, g.name); }}
                    style={{ width: 20, height: 20, color: "var(--cahtqt-color-danger)" }}
                  >
                    <X size={11} />
                  </button>
                </>
              )}
            </div>
          ))
        )}
      </div>

      {showCreate && (
        <CreateGroupDialog
          onConfirm={handleCreate}
          onCancel={() => setShowCreate(false)}
        />
      )}
    </div>
  );
}
