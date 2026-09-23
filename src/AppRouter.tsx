import { useEffect, useState } from "react";
import App from "./App";
import CanvasPage from "./components/CanvasPage";
import NotesPage from "./features/notes/components/NotesPage";
import ComponentsPage from "./features/components/ComponentsPage";

export default function AppRouter() {
  const currentRoute = (hash: string) => {
    const match = hash.match(/^#(canvas|notes)\/([^/]+)$/);
    return match ? { section: match[1], projectId: decodeURIComponent(match[2]) } : undefined;
  };
  const [hash, setHash] = useState(() => window.location.hash);
  useEffect(() => {
    const syncPage = () => setHash(window.location.hash);
    window.addEventListener("hashchange", syncPage);
    return () => window.removeEventListener("hashchange", syncPage);
  }, []);
  const route = currentRoute(hash);
  if (route?.section === "canvas") return <CanvasPage projectId={route.projectId} />;
  if (route?.section === "notes") return <NotesPage projectId={route.projectId} />;
  if (hash === "#components") return <ComponentsPage />;
  return <App />;
}
