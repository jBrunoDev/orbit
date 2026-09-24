import { ReactNode } from "react";
import { AppSidebar } from "./AppSidebar";
import "./AppShellScroll.css";

export function AppShell({
  activeSection,
  sidebarContent,
  children,
  onNavigate,
}: {
  activeSection: string;
  sidebarContent?: ReactNode;
  children: ReactNode;
  onNavigate?: (section: string) => void;
}) {
  return (
    <div className="app-shell">
      <AppSidebar activeSection={activeSection} onNavigate={onNavigate}>
        {sidebarContent}
      </AppSidebar>
      <div className="app-shell-content">{children}</div>
    </div>
  );
}
