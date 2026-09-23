import { ReactNode } from "react";
import { OrbitIcon, OrbitIconName } from "./OrbitIcon";
import "./AppShell.css";

const navigation: { id: string; label: string; icon: OrbitIconName }[] = [
  { id: "home", label: "Home", icon: "home" },
  { id: "canvas", label: "Canvas", icon: "canvas" },
  { id: "notes", label: "Notes", icon: "notes" },
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
  return (
    <aside className="app-sidebar" aria-label="Navegação principal">
      <div className="app-brand">
        <span className="orbit-mark" aria-hidden="true">
          <i />
          <b />
        </span>
        <strong>Orbit</strong>
      </div>
      <nav>
        {navigation.map((item, index) => (
          <span key={item.id}>
            {index === 5 && <i className="app-sidebar-divider" />}
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
        <span>BG</span>
        <strong>Bruno G</strong>
        <button aria-label="Abrir configurações">
          <OrbitIcon name="settings" />
        </button>
      </div>
    </aside>
  );
}
