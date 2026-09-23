export type PortDirection = "input" | "output" | "bidirectional";
export type PortProtocol = "http" | "sql" | "data" | "event";
export type ComponentType = "client" | "api-server" | "database" | "cache" | "load-balancer" | "custom";
export type VisualElementType = "text" | "sticky-note" | "shape" | "image";

export type CanvasViewport = { x: number; y: number; zoom: number };
export type Canvas = { id: string; projectId: string; name: string; viewport: CanvasViewport; createdAt: number; updatedAt: number };
export type Port = { id: string; componentId: string; key: string; direction: PortDirection; protocol: PortProtocol; data: Record<string, unknown>; order: number };
export type Component = { id: string; canvasId: string; type: ComponentType; label: string; description: string; x: number; y: number; width: number; height: number; color: string; data: Record<string, unknown>; ports: Port[] };
export type VisualElement = { id: string; canvasId: string; type: VisualElementType; x: number; y: number; width: number; height: number; data: Record<string, unknown> };
export type CanvasConnection = { id: string; canvasId: string; sourceComponentId: string; sourcePortId: string; targetComponentId: string; targetPortId: string; type: string; data: Record<string, unknown> };
