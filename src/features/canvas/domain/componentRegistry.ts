import { ComponentType, PortDirection, PortProtocol } from "./types";

export type PortTemplate = { key: string; direction: PortDirection; protocol: PortProtocol };
export type ComponentDefinition = { type: string; label: string; category: string; icon: string; color: string; description: string; ports: PortTemplate[]; simulationDefaults: Record<string, unknown> };

export const componentRegistry: Record<ComponentType, ComponentDefinition> = {
  client: { type: "client", label: "Client", category: "Entry", icon: "client", color: "purple", description: "Web / Mobile", ports: [{ key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  "load-balancer": { type: "load-balancer", label: "Load Balancer", category: "Network", icon: "loadBalancer", color: "blue", description: "Routes traffic", ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }], simulationDefaults: {} },
  "api-server": { type: "api-server", label: "API Server", category: "Service", icon: "api", color: "green", description: "Service", ports: [{ key: "http-in", direction: "input", protocol: "http" }, { key: "http-out", direction: "output", protocol: "http" }, { key: "data-out", direction: "output", protocol: "data" }], simulationDefaults: {} },
  database: { type: "database", label: "Database", category: "Data", icon: "database", color: "blue", description: "PostgreSQL", ports: [{ key: "sql-in", direction: "input", protocol: "sql" }], simulationDefaults: {} },
  cache: { type: "cache", label: "Cache", category: "Data", icon: "cache", color: "red", description: "Redis", ports: [{ key: "data-in", direction: "input", protocol: "data" }, { key: "data-out", direction: "output", protocol: "data" }], simulationDefaults: {} },
  custom: { type: "custom", label: "Custom Component", category: "Custom", icon: "shape", color: "purple", description: "Custom", ports: [], simulationDefaults: {} },
};
