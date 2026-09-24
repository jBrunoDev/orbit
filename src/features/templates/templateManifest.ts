export type TemplateSection = "Popular" | "Development" | "Personal & Study";

export type TemplatePort = {
  key: string;
  direction: "input" | "output" | "bidirectional";
  protocol: "http" | "sql" | "data" | "event";
  data?: { handlePosition?: "top" | "right" | "bottom" | "left" };
};
export type TemplateComponent = { componentType: string; label: string; description: string; x: number; y: number; width: number; height: number; color: string; ports: TemplatePort[]; data?: Record<string, unknown> };
export type TemplateSeed = {
  version: string;
  canvasName: string;
  components: TemplateComponent[];
  connections: { sourceIndex: number; sourcePortKey: string; targetIndex: number; targetPortKey: string; connectionType: string }[];
  notes: { title: string; content: string; tags: string[] }[];
  docs: { title: string; content: string }[];
};

export type TemplateDefinition = {
  id: string;
  version: string;
  name: string;
  description: string;
  categories: string[];
  tags: string[];
  section: TemplateSection;
  preview: "web" | "services" | "aws" | "next" | "api" | "database" | "planner" | "study" | "meeting";
  requiresAws: boolean;
  openTarget: "canvas" | "notes";
  seed: TemplateSeed;
};

const workflowPreviews = import.meta.glob<string>("../../../assets/workflows/*.png", { eager: true, import: "default" });

const workflowPreviewFiles: Record<string, string> = {
  "web-application-architecture": "WebApplication.png",
  "microservices-architecture": "Microservices Architecture.png",
  "aws-infrastructure": "AWS Infrastructure.png",
  "nextjs-application": "Next.js Application.png",
  "api-architecture": "API Architecture.png",
  "database-schema": "Database Schema.png",
  "daily-planner": "Daily Planner.png",
  "study-notes": "Study Notes.png",
  "meeting-notes": "Meeting Notes.png",
};

export function workflowPreviewFor(template: TemplateDefinition) {
  const filename = workflowPreviewFiles[template.id];
  return filename
    ? Object.entries(workflowPreviews).find(([path]) => path.endsWith(`/${filename}`))?.[1]
    : undefined;
}

const httpOut: TemplatePort = { key: "http-out", direction: "output", protocol: "http" };
const httpIn: TemplatePort = { key: "http-in", direction: "input", protocol: "http" };
const sqlIn: TemplatePort = { key: "sql-in", direction: "input", protocol: "sql" };
const dataIn: TemplatePort = { key: "data-in", direction: "input", protocol: "data" };
const dataOut: TemplatePort = { key: "data-out", direction: "output", protocol: "data" };

function component(componentType: string, label: string, description: string, x: number, y: number, color: string, ports: TemplatePort[]) : TemplateComponent {
  return { componentType, label, description, x, y, width: 180, height: 92, color, ports };
}

function noteComponent(componentType: string, label: string, description: string, color: string, content: string, tasks?: { id: string; text: string; status: "todo" | "in-progress" | "done" }[]): TemplateComponent {
  return { componentType, label, description, x: 280, y: 150, width: componentType === "note-daily-planner" ? 560 : 480, height: componentType === "note-daily-planner" ? 460 : 340, color, ports: [], data: { canvasKind: "note", noteTemplate: componentType.replace("note-", ""), content, ...(tasks ? { tasks } : {}) } };
}

function systemSeed(canvasName: string, title: string, components: TemplateComponent[], connections: TemplateSeed["connections"], noteTags: string[]): TemplateSeed {
  return {
    version: "1.0.0",
    canvasName,
    components,
    connections,
    notes: [{ title: `${title} decisions`, content: `# ${title}\n\nUse esta Note para registrar decisões, riscos e próximos passos.`, tags: noteTags }],
    docs: [{ title: `${title} overview`, content: `# ${title}\n\nEste documento foi criado pelo template e pode ser editado livremente.` }],
  };
}

const webSeed = systemSeed("Web application", "Web Application Architecture", [
  component("client", "Client", "Web / Mobile", 60, 220, "purple", [httpOut]),
  component("load-balancer", "Load Balancer", "Routes incoming requests", 310, 220, "blue", [httpIn, httpOut]),
  component("api-server", "API Service", "REST API", 580, 100, "green", [httpIn, httpOut, dataOut]),
  component("database", "Database", "PostgreSQL", 840, 175, "blue", [sqlIn]),
  component("cache", "Cache", "Redis", 840, 330, "red", [dataIn, dataOut]),
], [
  { sourceIndex: 0, sourcePortKey: "http-out", targetIndex: 1, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 1, sourcePortKey: "http-out", targetIndex: 2, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 2, sourcePortKey: "data-out", targetIndex: 3, targetPortKey: "sql-in", connectionType: "data" },
  { sourceIndex: 2, sourcePortKey: "data-out", targetIndex: 4, targetPortKey: "data-in", connectionType: "data" },
], ["system-design", "web"]);

const microservicesSeed = systemSeed("Microservices", "Microservices Architecture", [
  component("client", "Client", "Web client", 60, 230, "purple", [httpOut]),
  component("api-server", "API Gateway", "Public API entry", 300, 230, "blue", [httpIn, httpOut]),
  component("service", "User Service", "User domain", 570, 100, "green", [httpIn, dataOut]),
  component("service", "Order Service", "Order domain", 570, 250, "yellow", [httpIn, dataOut]),
  component("database", "Database", "Service data", 840, 250, "blue", [sqlIn]),
], [
  { sourceIndex: 0, sourcePortKey: "http-out", targetIndex: 1, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 1, sourcePortKey: "http-out", targetIndex: 2, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 1, sourcePortKey: "http-out", targetIndex: 3, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 2, sourcePortKey: "data-out", targetIndex: 4, targetPortKey: "sql-in", connectionType: "data" },
  { sourceIndex: 3, sourcePortKey: "data-out", targetIndex: 4, targetPortKey: "sql-in", connectionType: "data" },
], ["system-design", "microservices"]);

const awsSeed = systemSeed("AWS Infrastructure", "AWS Infrastructure", [
  // The reference workflow is a top-down 1 → 3 → 3 architecture, not a horizontal flow.
  component("cdn", "Route 53", "DNS routing", 460, 40, "purple", [
    { key: "route-to-cloudfront", direction: "output", protocol: "http", data: { handlePosition: "bottom" } },
    { key: "route-to-s3", direction: "output", protocol: "http", data: { handlePosition: "bottom" } },
    { key: "route-to-gateway", direction: "output", protocol: "http", data: { handlePosition: "bottom" } },
  ]),
  component("cdn", "CloudFront", "Content delivery", 100, 250, "purple", [{ key: "cloudfront-in", direction: "input", protocol: "http", data: { handlePosition: "top" } }]),
  component("storage", "S3 (Static)", "Static website hosting", 460, 250, "green", [
    { key: "s3-in", direction: "input", protocol: "http", data: { handlePosition: "top" } },
    { key: "s3-to-lambda", direction: "output", protocol: "data", data: { handlePosition: "bottom" } },
    { key: "s3-to-ec2", direction: "output", protocol: "data", data: { handlePosition: "bottom" } },
    { key: "s3-to-dynamodb", direction: "output", protocol: "data", data: { handlePosition: "bottom" } },
  ]),
  component("api-server", "API Gateway", "Managed API gateway", 820, 250, "purple", [{ key: "gateway-in", direction: "input", protocol: "http", data: { handlePosition: "top" } }]),
  component("worker", "Lambda", "Serverless compute", 100, 500, "yellow", [{ key: "lambda-in", direction: "input", protocol: "data", data: { handlePosition: "top" } }]),
  component("service", "EC2", "Virtual compute", 460, 500, "blue", [{ key: "ec2-in", direction: "input", protocol: "data", data: { handlePosition: "top" } }]),
  component("database", "DynamoDB", "Managed database", 820, 500, "blue", [{ key: "dynamodb-in", direction: "input", protocol: "data", data: { handlePosition: "top" } }]),
], [
  { sourceIndex: 0, sourcePortKey: "route-to-cloudfront", targetIndex: 1, targetPortKey: "cloudfront-in", connectionType: "http" },
  { sourceIndex: 0, sourcePortKey: "route-to-s3", targetIndex: 2, targetPortKey: "s3-in", connectionType: "http" },
  { sourceIndex: 0, sourcePortKey: "route-to-gateway", targetIndex: 3, targetPortKey: "gateway-in", connectionType: "http" },
  { sourceIndex: 2, sourcePortKey: "s3-to-lambda", targetIndex: 4, targetPortKey: "lambda-in", connectionType: "data" },
  { sourceIndex: 2, sourcePortKey: "s3-to-ec2", targetIndex: 5, targetPortKey: "ec2-in", connectionType: "data" },
  { sourceIndex: 2, sourcePortKey: "s3-to-dynamodb", targetIndex: 6, targetPortKey: "dynamodb-in", connectionType: "data" },
], ["infra", "aws"]);

const nextSeed = systemSeed("Next.js application", "Next.js Application", [
  component("client", "Browser", "Web browser", 60, 220, "blue", [httpOut]),
  component("service", "Next.js", "Frontend and server rendering", 300, 220, "purple", [httpIn, httpOut]),
  component("api-server", "API Routes", "Application API", 560, 220, "blue", [httpIn, dataOut]),
  component("database", "Database", "Application data", 820, 220, "yellow", [sqlIn]),
], [
  { sourceIndex: 0, sourcePortKey: "http-out", targetIndex: 1, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 1, sourcePortKey: "http-out", targetIndex: 2, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 2, sourcePortKey: "data-out", targetIndex: 3, targetPortKey: "sql-in", connectionType: "data" },
], ["development", "nextjs"]);

const apiSeed = systemSeed("API architecture", "API Architecture", [
  component("client", "Client", "API consumer", 60, 220, "purple", [httpOut]),
  component("api-server", "API", "REST or GraphQL", 300, 220, "green", [httpIn, httpOut, dataOut]),
  component("service", "Auth", "Authentication", 570, 110, "green", [httpIn]),
  component("service", "Rate Limiting", "Traffic policy", 570, 220, "yellow", [httpIn]),
  component("database", "Database", "Persistent data", 570, 330, "green", [sqlIn]),
], [
  { sourceIndex: 0, sourcePortKey: "http-out", targetIndex: 1, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 1, sourcePortKey: "http-out", targetIndex: 2, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 1, sourcePortKey: "http-out", targetIndex: 3, targetPortKey: "http-in", connectionType: "http" },
  { sourceIndex: 1, sourcePortKey: "data-out", targetIndex: 4, targetPortKey: "sql-in", connectionType: "data" },
], ["development", "api"]);

const databaseSeed = systemSeed("Database schema", "Database Schema", [
  component("database", "users", "id, name, email", 100, 160, "blue", [dataOut]),
  component("database", "posts", "id, user_id, title", 400, 160, "blue", [dataIn, dataOut]),
  component("database", "comments", "id, post_id, content", 700, 160, "blue", [dataIn]),
], [
  { sourceIndex: 0, sourcePortKey: "data-out", targetIndex: 1, targetPortKey: "data-in", connectionType: "data" },
  { sourceIndex: 1, sourcePortKey: "data-out", targetIndex: 2, targetPortKey: "data-in", connectionType: "data" },
], ["database", "schema"]);

const plannerSeed = systemSeed("Daily planner", "Daily Planner", [
  noteComponent("note-daily-planner", "Daily Planner", "Organize suas tarefas do dia a dia.", "purple", "Adicione tarefas, prioridades e próximos passos aqui.", [
    { id: "daily-study", text: "Estudar", status: "todo" },
    { id: "daily-implement", text: "Implementar", status: "in-progress" },
    { id: "daily-review", text: "Revisar", status: "done" },
  ]),
], [], ["personal", "productivity"]);

const studySeed = systemSeed("Study notes", "Study Notes", [
  noteComponent("note-study-notes", "Study Notes", "Estrutura para resumos e estudos.", "blue", "Registre conceitos, exemplos e um resumo com suas próprias palavras."),
], [], ["study", "notes"]);

const meetingSeed = systemSeed("Meeting notes", "Meeting Notes", [
  noteComponent("note-meeting-notes", "Meeting Notes", "Template para reuniões.", "green", "Registre decisões, responsáveis e próximos passos da reunião."),
], [], ["personal", "work"]);

export const officialTemplates: TemplateDefinition[] = [
  { id: "web-application-architecture", version: "1.1.0", name: "Web Application Architecture", description: "Arquitetura completa de uma aplicação web moderna.", categories: ["System Design", "Architecture"], tags: ["System Design", "Web", "Popular"], section: "Popular", preview: "web", requiresAws: false, openTarget: "canvas", seed: { ...webSeed, version: "1.1.0" } },
  { id: "microservices-architecture", version: "1.1.0", name: "Microservices Architecture", description: "Estrutura base para sistemas em microserviços.", categories: ["System Design", "Architecture"], tags: ["System Design", "Microservices", "Backend"], section: "Popular", preview: "services", requiresAws: false, openTarget: "canvas", seed: { ...microservicesSeed, version: "1.1.0" } },
  { id: "aws-infrastructure", version: "1.1.0", name: "AWS Infrastructure", description: "Arquitetura AWS com os principais serviços.", categories: ["Infra", "Architecture"], tags: ["Infra", "AWS", "Cloud"], section: "Popular", preview: "aws", requiresAws: true, openTarget: "canvas", seed: { ...awsSeed, version: "1.1.0" } },
  { id: "nextjs-application", version: "1.1.0", name: "Next.js Application", description: "Estrutura de um app Next.js com autenticação.", categories: ["Development"], tags: ["Development", "Next.js", "Fullstack"], section: "Development", preview: "next", requiresAws: false, openTarget: "canvas", seed: { ...nextSeed, version: "1.1.0" } },
  { id: "api-architecture", version: "1.1.0", name: "API Architecture", description: "Estrutura de API REST/GraphQL.", categories: ["Development", "Architecture"], tags: ["Development", "API", "Backend"], section: "Development", preview: "api", requiresAws: false, openTarget: "canvas", seed: { ...apiSeed, version: "1.1.0" } },
  { id: "database-schema", version: "1.1.0", name: "Database Schema", description: "Modelo de dados relacional.", categories: ["Development", "Diagrams"], tags: ["Database", "Schema", "SQL"], section: "Development", preview: "database", requiresAws: false, openTarget: "canvas", seed: { ...databaseSeed, version: "1.1.0" } },
  { id: "daily-planner", version: "1.1.0", name: "Daily Planner", description: "Organize suas tarefas do dia a dia.", categories: ["Personal", "Product"], tags: ["Personal", "Productivity"], section: "Personal & Study", preview: "planner", requiresAws: false, openTarget: "canvas", seed: { ...plannerSeed, version: "1.1.0" } },
  { id: "study-notes", version: "1.1.0", name: "Study Notes", description: "Estrutura para resumos e estudos.", categories: ["Study"], tags: ["Study", "Notes"], section: "Personal & Study", preview: "study", requiresAws: false, openTarget: "canvas", seed: { ...studySeed, version: "1.1.0" } },
  { id: "meeting-notes", version: "1.1.0", name: "Meeting Notes", description: "Template para reuniões.", categories: ["Personal", "Product"], tags: ["Personal", "Work"], section: "Personal & Study", preview: "meeting", requiresAws: false, openTarget: "canvas", seed: { ...meetingSeed, version: "1.1.0" } },
];
