import type { CatalogItem } from "./types";
import { officialTemplates } from "../templates/templateManifest";

const noteContent: Record<string, string> = {
  "daily-planner": "Adicione tarefas, prioridades e próximos passos aqui.",
  "study-notes": "Registre conceitos, exemplos e um resumo com suas próprias palavras.",
  "meeting-notes": "Registre decisões, responsáveis e próximos passos da reunião.",
};

const noteTemplateItems: CatalogItem[] = officialTemplates
  .filter((template) => Object.prototype.hasOwnProperty.call(noteContent, template.id))
  .map((template) => ({
    type: `note-${template.id}`,
    label: template.name,
    category: "Notes",
    icon: "note",
    color: template.id === "daily-planner" ? "purple" : template.id === "study-notes" ? "blue" : "green",
    description: template.description,
    ports: [],
    simulationDefaults: {},
    defaultSize: { width: template.id === "daily-planner" ? 560 : 480, height: template.id === "daily-planner" ? 460 : 340 },
    defaultData: template.seed.components[0]?.data ?? { canvasKind: "note", noteTemplate: template.id, content: noteContent[template.id] ?? "" },
  }));

export const orbitCoreItems: CatalogItem[] = [
  { type: "client", label: "Client", category: "Clients", icon: "client", color: "purple", description: "Web / Mobile", ports: [{ key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "web-client", label: "Web Client", category: "Clients", icon: "client", color: "purple", description: "Browser application", ports: [{ key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "mobile-client", label: "Mobile Client", category: "Clients", icon: "client", color: "purple", description: "Mobile application", ports: [{ key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "api-server", label: "API Server", category: "Compute", icon: "api", color: "green", description: "Service", ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }, { key: "data-out", direction: "output", protocol: "data" }], simulationDefaults: {} },
  { type: "worker", label: "Worker", category: "Compute", icon: "worker", color: "yellow", description: "Background processing", ports: [{ key: "event-in", direction: "input", protocol: "event" }, { key: "data-out", direction: "output", protocol: "data" }], simulationDefaults: {} },
  { type: "load-balancer", label: "Load Balancer", category: "Networking", icon: "loadBalancer", color: "blue", description: "Routes traffic", ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "cdn", label: "CDN", category: "Networking", icon: "cdn", color: "blue", description: "Edge cache", ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "database", label: "Database", category: "Data", icon: "database", color: "blue", description: "PostgreSQL", ports: [{ key: "sql-in", direction: "input", protocol: "sql" }], simulationDefaults: {} },
  { type: "cache", label: "Cache", category: "Data", icon: "cache", color: "red", description: "Redis", ports: [{ key: "data-in", direction: "input", protocol: "data" }, { key: "data-out", direction: "output", protocol: "data" }], simulationDefaults: {} },
  { type: "queue", label: "Queue", category: "Data", icon: "queue", color: "yellow", description: "Async messages", ports: [{ key: "event-in", direction: "input", protocol: "event" }, { key: "event-out", direction: "output", protocol: "event" }], simulationDefaults: {} },
  { type: "python", label: "Python", category: "Languages", icon: "python", color: "yellow", description: "Python service", ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "typescript", label: "TypeScript", category: "Languages", icon: "typescript", color: "blue", description: "TypeScript service", ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "storage", label: "Object Storage", category: "Tech/Infra", icon: "storage", color: "purple", description: "Files and blobs", ports: [{ key: "data-in", direction: "input", protocol: "data" }, { key: "data-out", direction: "output", protocol: "data" }], simulationDefaults: {} },
  { type: "service", label: "Service", category: "Tech/Infra", icon: "service", color: "green", description: "Custom service", ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "postgresql-database", label: "PostgreSQL Database", category: "Database", icon: "database", color: "blue", description: "Relational database", tags: ["PostgreSQL", "SQL"], ports: [{ key: "sql-in", direction: "input", protocol: "sql" }], simulationDefaults: {} },
  { type: "redis-cache", label: "Redis Cache", category: "Database", icon: "cache", color: "red", description: "In memory cache", tags: ["Redis", "Cache"], ports: [{ key: "data-in", direction: "input", protocol: "data" }, { key: "data-out", direction: "output", protocol: "data" }], simulationDefaults: {} },
  { type: "docker-container", label: "Docker Container", category: "Tech/Infra", icon: "service", color: "blue", description: "Packaged application runtime", tags: ["Docker", "Container"], ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  { type: "kubernetes-cluster", label: "Kubernetes Cluster", category: "Tech/Infra", icon: "loadBalancer", color: "blue", description: "Container orchestration", tags: ["Kubernetes", "DevOps"], ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
];

export const catalogItems = [...orbitCoreItems.slice(0, 3), ...noteTemplateItems, ...orbitCoreItems.slice(3)];
export default catalogItems;
