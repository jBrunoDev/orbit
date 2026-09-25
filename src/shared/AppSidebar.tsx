import { ReactNode, useEffect } from "react";
import { ProfileAvatar } from "../features/profile/ProfileAvatar";
import { useProfileStore } from "../features/profile/profileStore";
import orbitLogoHorizontal from "../../assets/branding/orbit-logo-horizontal.png";
import orbitSymbol from "../../assets/branding/orbit-symbol.png";
import { OrbitIcon, OrbitIconName } from "./OrbitIcon";
import "./AppShell.css";
import "./OrbitBranding.css";
import "./ProfileSidebar.css";

const navigation: { id: string; label: string; icon: OrbitIconName }[] = [
  { id: "home", label: "Home", icon: "home" },
  { id: "canvas", label: "Canvas", icon: "canvas" },
  { id: "calendar", label: "Calendar", icon: "calendar" },
  { id: "templates", label: "Templates", icon: "templates" },
  { id: "components", label: "Components", icon: "components" },
  { id: "simulate", label: "Simulate", icon: "simulate" },
  { id: "docs", label: "Docs", icon: "docs" },
];

export function AppSidebar({
  activeSection,
  children,
  onNavigate,
}: {
  activeSection: string;
  children?: ReactNode;
  onNavigate?: (section: string) => void;
}) {
  const loadProfile = useProfileStore((state) => state.load);
  useEffect(() => { void loadProfile(); }, [loadProfile]);

  return (
    <aside className="app-sidebar" aria-label="Navegação principal">
      <div className="app-brand">
        <img className="app-brand-logo" src={orbitLogoHorizontal} alt="Orbit" />
        <img className="app-brand-symbol" src={orbitSymbol} alt="Orbit" />
      </div>
      <nav>
        {navigation.map((item, index) => (
          <span key={item.id}>
            {index === 6 && <i className="app-sidebar-divider" />}
            <button
              className={activeSection === item.id ? "active" : ""}
              aria-current={activeSection === item.id ? "page" : undefined}
              onClick={() => {
                if (onNavigate) {
                  onNavigate(item.id);
                  return;
                }
                window.location.hash = item.id === "home" ? "" : item.id;
              }}
            >
              <OrbitIcon name={item.icon} />
              {item.label}
            </button>
          </span>
        ))}
      </nav>
      {children && <div className="app-sidebar-slot">{children}</div>}
      <div className="app-profile">
        <button className="app-profile-open" aria-label="Abrir Profile" aria-current={activeSection === "profile" ? "page" : undefined} onClick={() => { if (onNavigate) { onNavigate("profile"); return; } window.location.hash = "profile"; }}>
          <span><ProfileAvatar alt="" fallback="BG" /></span>
          <strong>Bruno G</strong>
        </button>
        <button aria-label="Abrir configurações" onClick={() => { window.location.hash = "settings/ai"; }}>
          <OrbitIcon name="settings" />
        </button>
      </div>
    </aside>
  );
}
