import { Connection, Edge, Node } from "@xyflow/react";
import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import { isTauriAvailable } from "../../../shared/tauri";
import { ComponentDefinition } from "../domain/componentRegistry";

export type PersistedPort = {
  id: string;
  componentId: string;
  key: string;
  direction: "input" | "output" | "bidirectional";
  protocol: "http" | "sql" | "data" | "event";
  data: Record<string, unknown>;
  order: number;
};
export type PersistedComponent = {
  id: string;
  canvasId: string;
  componentType: string;
  label: string;
  description: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  data: Record<string, unknown>;
  ports: PersistedPort[];
};
export type PersistedVisual = {
  id: string;
  canvasId: string;
  visualType: string;
  x: number;
  y: number;
  width: number;
  height: number;
  data: Record<string, unknown>;
};
type Snapshot = {
  canvas: { id: string; projectId: string; name: string };
  components: PersistedComponent[];
  connections: {
    id: string;
    sourceComponentId: string;
    sourcePortId: string;
    targetComponentId: string;
    targetPortId: string;
    connectionType: string;
    data: Record<string, unknown>;
  }[];
  visuals: PersistedVisual[];
};

const asRecord = (value: unknown): Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};

const normalizeComponents = (components: PersistedComponent[]) =>
  components.map((component) => ({
    ...component,
    data: asRecord(component.data),
    ports: component.ports.map((port) => ({ ...port, data: asRecord(port.data) })),
  }));

const asNodes = (components: PersistedComponent[], projectId: string): Node[] =>
  components.map((component) => ({
    id: component.id,
    type: "orbitNode",
    position: { x: component.x, y: component.y },
    data: { ...component, projectId },
    width: component.width,
    height: component.height,
  }));
const asEdges = (connections: Snapshot["connections"]): Edge[] =>
  connections.map((connection) => ({
    id: connection.id,
    source: connection.sourceComponentId,
    sourceHandle: connection.sourcePortId,
    target: connection.targetComponentId,
    targetHandle: connection.targetPortId,
    type: "smoothstep",
    animated: false,
    data: connection,
  }));
const asVisualNodes = (visuals: Snapshot["visuals"]): Node[] => visuals.map((visual) => ({ id: visual.id, type: "imageNode", className: "canvas-image-node", position: { x: visual.x, y: visual.y }, data: { ...visual.data, visualType: visual.visualType }, style: { width: visual.width, height: visual.height } }));

type CanvasState = {
  canvasId?: string;
  nodes: Node[];
  edges: Edge[];
  selectedId?: string;
  loading: boolean;
  error?: string;
  load: (projectId: string, canvasId?: string) => Promise<void>;
  addComponent: (
    projectId: string,
    definition: ComponentDefinition,
    position?: { x: number; y: number },
  ) => Promise<void>;
  addImage: (projectId: string, assetId: string, position: { x: number; y: number }) => Promise<void>;
  move: (projectId: string, node: Node) => Promise<void>;
  connect: (projectId: string, connection: Connection) => Promise<void>;
  updateComponent: (
    projectId: string,
    component: PersistedComponent,
  ) => Promise<void>;
  duplicateComponent: (
    projectId: string,
    component: PersistedComponent,
    offset?: { x: number; y: number },
  ) => Promise<void>;
  updateVisual: (projectId: string, visual: PersistedVisual) => Promise<void>;
  removeComponent: (projectId: string, componentId: string) => Promise<void>;
  removeVisual: (projectId: string, visualId: string) => Promise<void>;
  select: (id?: string) => void;
};

function applySnapshot(snapshot: Snapshot, projectId: string) {
  const components = normalizeComponents(snapshot.components);
  return {
    canvasId: snapshot.canvas.id,
    nodes: [...asNodes(components, projectId), ...asVisualNodes(snapshot.visuals)],
    edges: asEdges(snapshot.connections),
    error: undefined,
  };
}

export const useCanvasStore = create<CanvasState>((set, get) => ({
  nodes: [],
  edges: [],
  loading: true,
  select: (selectedId) => set({ selectedId }),
  load: async (projectId, canvasId) => {
    set({ loading: true, error: undefined });
    if (!isTauriAvailable()) {
      set({
        loading: false,
        error: "Abra o Orbit pelo aplicativo desktop para usar o Canvas local.",
      });
      return;
    }
    try {
      set({
        ...applySnapshot(await invoke<Snapshot>("load_canvas", { projectId, canvasId }), projectId),
        loading: false,
      });
    } catch (error) {
      set({
        loading: false,
        error:
          error instanceof Error
            ? error.message
            : "Não foi possível carregar o Canvas.",
      });
    }
  },
  addComponent: async (projectId, definition, position) => {
    const { canvasId } = get();
    if (!canvasId) return;
    const offset = position ?? { x: 80 + get().nodes.length * 28, y: 80 + get().nodes.length * 28 };
    const snapshot = await invoke<Snapshot>("create_canvas_component", {
      input: {
        projectId,
        canvasId,
        componentType: definition.type,
        label: definition.label,
        description: definition.description,
        x: offset.x,
        y: offset.y,
        width: definition.defaultSize?.width ?? 180,
        height: definition.defaultSize?.height ?? 86,
        color: definition.color,
        data: definition.defaultData ?? {},
        ports: definition.ports.map((port) => ({ ...port, data: {} })),
      },
    });
    set(applySnapshot(snapshot, projectId));
  },
  addImage: async (projectId, assetId, position) => {
    const { canvasId } = get(); if (!canvasId) return;
    const snapshot = await invoke<Snapshot>("create_canvas_visual", { input: { projectId, canvasId, visualType: "image", x: position.x, y: position.y, width: 320, height: 220, data: { assetId, label: "Imagem" } } });
    set(applySnapshot(snapshot, projectId));
  },
  move: async (projectId, node) => {
    const { canvasId } = get();
    if (!canvasId) return;
    set((state) => ({
      nodes: state.nodes.map((item) =>
        item.id === node.id ? { ...item, position: node.position } : item,
      ),
    }));
    await invoke("move_canvas_element", {
      input: {
        projectId,
        canvasId,
        elementId: node.id,
        x: node.position.x,
        y: node.position.y,
      },
    });
  },
  connect: async (projectId, connection) => {
    const { canvasId } = get();
    if (
      !canvasId ||
      !connection.source ||
      !connection.target ||
      !connection.sourceHandle ||
      !connection.targetHandle
    )
      return;
    const snapshot = await invoke<Snapshot>("create_canvas_connection", {
      input: {
        projectId,
        canvasId,
        sourceComponentId: connection.source,
        sourcePortId: connection.sourceHandle,
        targetComponentId: connection.target,
        targetPortId: connection.targetHandle,
        connectionType: "default",
        data: {},
      },
    });
    set(applySnapshot(snapshot, projectId));
  },
  updateComponent: async (projectId, component) => {
    const { canvasId } = get();
    if (!canvasId) return;
    await invoke("update_canvas_component", {
      input: {
        projectId,
        canvasId,
        componentId: component.id,
        label: component.label,
        description: component.description,
        color: component.color,
        data: component.data,
      },
    });
    set((state) => ({
      nodes: state.nodes.map((node) =>
        node.id === component.id ? { ...node, data: component } : node,
      ),
    }));
  },
  duplicateComponent: async (projectId, component, offset = { x: 32, y: 32 }) => {
    const { canvasId } = get();
    if (!canvasId) return;
    const snapshot = await invoke<Snapshot>("create_canvas_component", {
      input: {
        projectId,
        canvasId,
        componentType: component.componentType,
        label: component.label,
        description: component.description,
        x: component.x + offset.x,
        y: component.y + offset.y,
        width: component.width,
        height: component.height,
        color: component.color,
        data: component.data,
        ports: component.ports.map((port) => ({
          key: port.key,
          direction: port.direction,
          protocol: port.protocol,
          data: port.data,
        })),
      },
    });
    const created = snapshot.components.find(
      (candidate) => candidate.x === component.x + offset.x && candidate.y === component.y + offset.y,
    );
    set({ ...applySnapshot(snapshot, projectId), selectedId: created?.id });
  },
  updateVisual: async (projectId, visual) => {
    const { canvasId } = get();
    if (!canvasId) return;
    await invoke("update_canvas_visual", { input: { projectId, canvasId, visualId: visual.id, data: visual.data } });
    set((state) => ({ nodes: state.nodes.map((node) => node.id === visual.id ? { ...node, data: { ...visual.data, visualType: visual.visualType } } : node) }));
  },
  removeComponent: async (projectId, componentId) => {
    const { canvasId } = get();
    if (!canvasId) return;
    const snapshot = await invoke<Snapshot>("delete_canvas_component", {
      input: { projectId, canvasId, componentId },
    });
    set({ ...applySnapshot(snapshot, projectId), selectedId: undefined });
  },
  removeVisual: async (projectId, visualId) => {
    const { canvasId } = get();
    if (!canvasId) return;
    const snapshot = await invoke<Snapshot>("delete_canvas_visual", { input: { projectId, canvasId, visualId } });
    set({ ...applySnapshot(snapshot, projectId), selectedId: undefined });
  },
}));
