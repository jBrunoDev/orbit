import { useEffect, useState } from "react";
import App from "./App";
import CanvasPage from "./components/CanvasPage";
import NotesPage from "./features/notes/components/NotesPage";
import ComponentsPage from "./features/components/ComponentsPage";
import TemplatesPage from "./features/templates/TemplatesPage";
import CalendarPage from "./features/calendar/CalendarPage";
import SimulationPage from "./features/simulate/SimulationPage";
import DocsPage from "./features/docs/components/DocsPage";
import AiSettingsPage from "./features/ai-architecture/AiSettingsPage";
import ProfilePage from "./features/profile/ProfilePage";

export default function AppRouter() {
  const currentRoute = (hash: string) => {
    const match = hash.match(/^#(canvas|notes|calendar|docs)\/([^/]+)(?:\/([^/]+))?$/);
    return match ? { section: match[1], projectId: decodeURIComponent(match[2]), resourceId: match[3] ? decodeURIComponent(match[3]) : undefined } : undefined;
  };
  const [hash, setHash] = useState(() => window.location.hash);
  useEffect(() => {
    const syncPage = () => setHash(window.location.hash);
    window.addEventListener("hashchange", syncPage);
    return () => window.removeEventListener("hashchange", syncPage);
  }, []);
  const route = currentRoute(hash);
  if (route?.section === "canvas") return <CanvasPage projectId={route.projectId} canvasId={route.resourceId} />;
  if (route?.section === "notes") return <NotesPage projectId={route.projectId} />;
  if (route?.section === "calendar") return <CalendarPage projectId={route.projectId} />;
  if (route?.section === "docs") return <DocsPage projectId={route.projectId} documentId={route.resourceId} />;
  if (hash === "#components") return <ComponentsPage />;
  if (hash === "#templates") return <TemplatesPage />;
  if (hash === "#simulate") return <SimulationPage />;
  if (hash === "#settings/ai") return <AiSettingsPage />;
  if (hash === "#profile") return <ProfilePage />;
  return <App />;
}
