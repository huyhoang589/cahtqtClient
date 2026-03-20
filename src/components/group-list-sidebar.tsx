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
        background: "#f8fafc",
        borderRight: "1px solid #334155",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "12px 12px 8px",
          borderBottom: "1px solid #334155",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <span
          className="section-header"
          style={{ marginBottom: 0, color: "#1e293b" }}
        >
          Partner List
        </span>
        <button
          className="btn-icon"
          onClick={() => setShowCreate(true)}
          title="Create partner"
          style={{ width: 24, height: 24, color: "#1e293b" }}
        >
          <Plus size={14} />
        </button>
      </div>

      {/* Partner list */}
      <div style={{ flex: 1, overflowY: "auto" }}>
        {groups.length === 0 ? (
          <div style={{ padding: 16, color: "#64748b", fontSize: "var(--font-size-sm)" }}>
            No partners yet
          </div>
        ) : (
          groups.map((g) => (
            <div
              key={g.id}
              style={{
                padding: "8px 12px",
                cursor: "pointer",
                background: selectedId === g.id ? "#e0f2fe" : "transparent",
                borderBottom: "1px solid #e2e8f0",
                borderLeft: selectedId === g.id ? "3px solid #00b4d8" : "3px solid transparent",
                display: "flex",
                alignItems: "center",
                gap: 6,
              }}
              onClick={() => onSelect(g.id)}
              onMouseEnter={(e) => {
                if (selectedId !== g.id) {
                  (e.currentTarget as HTMLDivElement).style.background = "#e0f2fe";
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
                  <span style={{ flex: 1, fontSize: "var(--font-size-base)", color: "#1e293b" }}>
                    {g.name}
                  </span>
                  <span className="badge badge-default">{g.member_count ?? 0}</span>
                  <button
                    className="btn-icon"
                    title="Rename"
                    onClick={(e) => { e.stopPropagation(); startRename(g); }}
                    style={{ width: 20, height: 20, color: "#475569" }}
                  >
                    <Pencil size={11} />
                  </button>
                  <button
                    className="btn-icon"
                    title="Delete"
                    onClick={(e) => { e.stopPropagation(); handleDelete(g.id, g.name); }}
                    style={{ width: 20, height: 20, color: "var(--color-accent-danger)" }}
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
