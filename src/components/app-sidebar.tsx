import { useState } from "react";
import { NavLink } from "react-router-dom";
import { Lock, Unlock, Settings, ChevronLeft, ChevronRight } from "lucide-react";

const NAV_ITEMS = [
  { to: "/encrypt",  label: "Encrypt",  Icon: Lock },
  { to: "/decrypt",  label: "Decrypt",  Icon: Unlock },
  { to: "/settings", label: "Settings", Icon: Settings },
];

export default function AppSidebar() {
  const [collapsed, setCollapsed] = useState<boolean>(() =>
    localStorage.getItem("sidebar-collapsed") === "true"
  );
  const [toggleHover, setToggleHover] = useState(false);

  const toggle = () => {
    setCollapsed((prev) => {
      localStorage.setItem("sidebar-collapsed", String(!prev));
      return !prev;
    });
  };

  const w = collapsed ? 56 : 200;

  return (
    <nav style={{
      width: w,
      minWidth: w,
      background: "#12161f",
      borderRight: "1px solid rgba(255,255,255,0.05)",
      display: "flex",
      flexDirection: "column",
      transition: "width var(--transition-base)",
      overflow: "hidden",
    }}>
      {/* App name */}
      <div style={{
        padding: collapsed ? "20px 0 16px" : "20px 16px 16px",
        borderBottom: "1px solid rgba(255,255,255,0.06)",
        marginBottom: 8,
        textAlign: collapsed ? "center" : "left",
        flexShrink: 0,
      }}>
        {collapsed ? null : (
          <>
            <div style={{
              fontSize: "var(--font-size-lg)",
              fontWeight: "var(--font-weight-bold)",
              color: "var(--color-accent-primary)",
              whiteSpace: "nowrap",
            }}>
              CAHTQT
            </div>
            <div style={{
              fontSize: "var(--font-size-xs)",
              color: "rgba(255,255,255,0.45)",
              marginTop: 2,
              whiteSpace: "nowrap",
            }}>
              PKI Encryption
            </div>
          </>
        )}
      </div>

      {/* Navigation links */}
      <div style={{ flex: 1 }}>
        {NAV_ITEMS.map(({ to, label, Icon }) => (
          <NavLink
            key={to}
            to={to}
            style={({ isActive }) => ({
              display: "flex",
              alignItems: "center",
              justifyContent: collapsed ? "center" : "flex-start",
              gap: collapsed ? 0 : 12,
              height: 40,
              padding: collapsed ? 0 : "0 16px",
              color: isActive ? "#ffffff" : "rgba(255,255,255,0.45)",
              background: isActive
                ? "linear-gradient(90deg, var(--color-accent-primary), #0098b5)"
                : "transparent",
              boxShadow: isActive ? "0 2px 10px rgba(0,198,224,0.3)" : "none",
              textDecoration: "none",
              fontSize: "var(--font-size-base)",
              fontWeight: "var(--font-weight-medium)",
              borderRadius: "var(--radius-md)",
              margin: "2px 8px",
              transition: "background var(--transition-base), color var(--transition-base)",
            })}
            title={collapsed ? label : undefined}
          >
            {({ isActive }) => (
              <>
                <Icon size={18} color={isActive ? "#ffffff" : "rgba(255,255,255,0.45)"} />
                {!collapsed && <span style={{ whiteSpace: "nowrap" }}>{label}</span>}
              </>
            )}
          </NavLink>
        ))}
      </div>

      {/* Collapse toggle */}
      <button
        onClick={toggle}
        onMouseEnter={() => setToggleHover(true)}
        onMouseLeave={() => setToggleHover(false)}
        style={{
          height: 32,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          background: "transparent",
          border: "none",
          borderTop: "1px solid rgba(255,255,255,0.06)",
          color: toggleHover ? "rgba(255,255,255,0.6)" : "rgba(255,255,255,0.25)",
          cursor: "pointer",
          padding: 0,
          flexShrink: 0,
          transition: "color var(--transition-base)",
        }}
        title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
      >
        {collapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
      </button>
    </nav>
  );
}
