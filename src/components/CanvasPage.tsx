import { useEffect, useMemo, useRef, useState, type ClipboardEvent, type DragEvent } from "react";
import { Background, Controls, MiniMap, ReactFlow, ReactFlowProvider, useReactFlow } from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { AppShell } from "../shared/AppShell";
import { ProjectHeader } from "../shared/ProjectHeader";
import { componentRegistry } from "../features/canvas/domain/componentRegistry";
import { OrbitNode } from "../features/canvas/OrbitNode";
import { LibraryPanel } from "../features/library/LibraryPanel";
import { defaultCatalogRepository } from "../features/catalog/repository";
import { CatalogIcon } from "../features/catalog/icons";
import type { CatalogItem } from "../features/catalog/types";
import { type PersistedComponent, type PersistedVisual, useCanvasStore } from "../features/canvas/application/canvasStore";
import { invoke } from "@tauri-apps/api/core";
import "./CanvasPage.css";
import "./CanvasLayout.css";

function ImageNode({ data }: { data: { assetId?: string } }) { const [src, setSrc] = useState<string>(); useEffect(() => { if (!data.assetId) return; void invoke<{ mediaType: string; bytes: number[] }>("read_asset", { input: { projectId: window.location.hash.split("/")[1], assetId: data.assetId } }).then((asset) => { let binary = ""; asset.bytes.forEach((byte) => { binary += String.fromCharCode(byte); }); setSrc(`data:${asset.mediaType};base64,${btoa(binary)}`); }); }, [data.assetId]); return src ? <img src={src} alt="Imagem do Canvas" style={{ width: "100%", height: "100%", objectFit: "contain" }} /> : <div style={{ padding: 12 }}>Carregando imagem...</div>; }
const nodeTypes = { orbitNode: OrbitNode, orbit: OrbitNode, imageNode: ImageNode };

function isPersistedComponent(data: unknown): data is PersistedComponent {
  if (!data || typeof data !== "object") return false;
  const candidate = data as Partial<PersistedComponent>;
  return (
    typeof candidate.componentType === "string" &&
    typeof candidate.label === "string" &&
    Array.isArray(candidate.ports)
  );
}

function isPersistedImage(data: unknown): data is Record<string, unknown> & { assetId: string } {
  return Boolean(data) && typeof data === "object" && typeof (data as { assetId?: unknown }).assetId === "string";
}

function CanvasEngine({ projectId, canvasId }: { projectId: string; canvasId?: string }) {
  const { screenToFlowPosition } = useReactFlow();
  const { nodes, edges, selectedId, loading, error, load, addComponent, addImage, move, connect, updateComponent, updateVisual, duplicateComponent, removeComponent, removeVisual, select } = useCanvasStore();
  const componentClipboard = useRef<PersistedComponent | undefined>(undefined);
  const pasteCount = useRef(0);
  useEffect(() => { void load(projectId, canvasId); }, [canvasId, load, projectId]);
  const selected = useMemo(() => {
    const data = nodes.find((node) => node.id === selectedId)?.data;
    return isPersistedComponent(data) ? data : undefined;
  }, [nodes, selectedId]);
  const selectedImage = useMemo<PersistedVisual | undefined>(() => {
    const node = nodes.find((item) => item.id === selectedId);
    if (!node || !isPersistedImage(node.data)) return undefined;
    return { id: node.id, canvasId: "", visualType: "image", x: node.position.x, y: node.position.y, width: Number(node.style?.width) || 320, height: Number(node.style?.height) || 220, data: node.data };
  }, [nodes, selectedId]);
  useEffect(() => {
    const handleCanvasShortcut = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (target?.matches("input, textarea, select, [contenteditable='true']")) return;
      const isCopyShortcut = (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "c";
      const isPasteShortcut = (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v";
      if (isCopyShortcut && selected) {
        event.preventDefault();
        componentClipboard.current = {
          ...selected,
          data: { ...selected.data },
          ports: selected.ports.map((port) => ({ ...port, data: { ...port.data } })),
        };
        pasteCount.current = 0;
        return;
      }
      if (isPasteShortcut && componentClipboard.current) {
        event.preventDefault();
        pasteCount.current += 1;
        const offset = 32 * pasteCount.current;
        void duplicateComponent(projectId, componentClipboard.current, { x: offset, y: offset });
        return;
      }
      if (!selectedId || (event.key !== "Delete" && event.key !== "Backspace")) return;
      event.preventDefault();
      if (selectedImage) void removeVisual(projectId, selectedId);
      else void removeComponent(projectId, selectedId);
    };
    window.addEventListener("keydown", handleCanvasShortcut);
    return () => window.removeEventListener("keydown", handleCanvasShortcut);
  }, [duplicateComponent, projectId, removeComponent, removeVisual, selected, selectedId, selectedImage]);
  const addCatalogItem = (item: CatalogItem, position?: { x: number; y: number }) => void addComponent(projectId, item, position);
  const addAtCenter = (item: CatalogItem) => { const workspace = document.querySelector<HTMLElement>(".canvas-workspace"); if (!workspace) return addCatalogItem(item); const bounds = workspace.getBoundingClientRect(); addCatalogItem(item, screenToFlowPosition({ x: bounds.left + bounds.width / 2, y: bounds.top + bounds.height / 2 })); };
  const handleDrop = (event: DragEvent) => { event.preventDefault(); const image = Array.from(event.dataTransfer.files).find((file) => file.type.startsWith("image/")); if (image) { void image.arrayBuffer().then((buffer) => invoke<{ id: string }>("create_asset", { input: { projectId, originalName: image.name || "dropped-image.png", mediaType: image.type, bytes: Array.from(new Uint8Array(buffer)) } })).then((asset) => addImage(projectId, asset.id, screenToFlowPosition({ x: event.clientX, y: event.clientY }))); return; } const type = event.dataTransfer.getData("application/orbit-node"); const item = defaultCatalogRepository.getItemByType(type); if (item) addCatalogItem(item, screenToFlowPosition({ x: event.clientX, y: event.clientY })); };
  const handlePaste = (event: ClipboardEvent) => { const image = Array.from(event.clipboardData.files).find((file) => file.type.startsWith("image/")); if (!image) return; event.preventDefault(); void image.arrayBuffer().then((buffer) => invoke<{ id: string }>("create_asset", { input: { projectId, originalName: image.name || "pasted-image.png", mediaType: image.type, bytes: Array.from(new Uint8Array(buffer)) } })).then((asset) => addImage(projectId, asset.id, screenToFlowPosition({ x: 240, y: 180 }))); };

  return <AppShell activeSection="canvas" onNavigate={(section) => { window.location.hash = section === "home" ? "" : section === "canvas" ? `canvas/${encodeURIComponent(projectId)}` : section === "notes" ? `notes/${encodeURIComponent(projectId)}` : section === "calendar" ? `calendar/${encodeURIComponent(projectId)}` : section === "docs" ? `docs/${encodeURIComponent(projectId)}` : section; }}>
    <div className="canvas-layout">
      <LibraryPanel onAdd={addAtCenter} />
      <main className="canvas-main">
        <ProjectHeader title="Canvas" onBack={() => { window.location.hash = ""; }} />
        <section className="canvas-workspace" aria-label="Canvas do Project" onPaste={handlePaste}>
          <div className="canvas-toolbar" aria-label="Ferramentas do Canvas">{Object.values(componentRegistry).slice(0, 5).map((item) => <button key={item.type} className="canvas-tool" aria-label={`Adicionar ${item.label}`} onClick={() => void addComponent(projectId, item)}><CatalogIcon icon={item.icon} className={`tone-${item.color}`} /></button>)}</div>
          {loading && <div className="canvas-engine-state">Carregando Canvas...</div>}
          {error && <div className="canvas-engine-state error" role="alert">{error}</div>}
          {!loading && !error && <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} fitView onNodeClick={(_, node) => select(node.id)} onPaneClick={() => select(undefined)} onNodeDragStop={(_, node) => void move(projectId, node)} onConnect={(connection) => void connect(projectId, connection)} onDragOver={(event) => { event.preventDefault(); event.dataTransfer.dropEffect = "copy"; }} onDrop={handleDrop} deleteKeyCode={null}><Background gap={18} size={1} /><Controls showInteractive={false} /><MiniMap pannable zoomable /></ReactFlow>}
        </section>
      </main>
      <aside className="properties-panel"><header><button className="active">Properties</button><button disabled>AI Assist</button></header>{selected ? <ComponentInspector projectId={projectId} component={selected} onSave={updateComponent} onDuplicate={duplicateComponent} onRemove={removeComponent} /> : selectedImage ? <ImageInspector projectId={projectId} image={selectedImage} onSave={updateVisual} onRemove={removeVisual} /> : <div className="canvas-empty-inspector"><strong>Selecione um elemento</strong><span>Crie um Component ou adicione uma imagem para começar.</span></div>}</aside>
    </div>
  </AppShell>;
}

function ComponentInspector({ projectId, component, onSave, onDuplicate, onRemove }: { projectId: string; component: PersistedComponent; onSave: (projectId: string, component: PersistedComponent) => Promise<void>; onDuplicate: (projectId: string, component: PersistedComponent) => Promise<void>; onRemove: (projectId: string, componentId: string) => Promise<void> }) {
  const update = (field: "label" | "description", value: string) => void onSave(projectId, { ...component, [field]: value });
  const icon = defaultCatalogRepository.getItemByType(component.componentType)?.icon ?? "shape";
  return <><section className="selected-node"><CatalogIcon icon={icon} className={`tone-${component.color}`} /><div><strong>{component.label}</strong><span>{component.componentType}</span></div></section><section className="property-group"><h2>General</h2><label>Label<input defaultValue={component.label} onBlur={(event) => update("label", event.target.value)} /></label><label>Description<input defaultValue={component.description} onBlur={(event) => update("description", event.target.value)} /></label><p className="component-port-summary">{component.ports.length} Ports tipados</p></section><section className="property-group"><h2>Position</h2><div className="dimensions"><label>X<input value={Math.round(component.x)} readOnly /></label><label>Y<input value={Math.round(component.y)} readOnly /></label></div></section><button type="button" className="duplicate-component-button" onClick={() => void onDuplicate(projectId, component)}>Duplicar Component <kbd>Ctrl+C / Ctrl+V</kbd></button><button type="button" className="delete-component-button" onClick={() => void onRemove(projectId, component.id)}>Apagar Component</button></>;
}

function ImageInspector({ projectId, image, onSave, onRemove }: { projectId: string; image: PersistedVisual; onSave: (projectId: string, image: PersistedVisual) => Promise<void>; onRemove: (projectId: string, visualId: string) => Promise<void> }) {
  const label = typeof image.data.label === "string" ? image.data.label : "Imagem";
  return <><section className="selected-node image-selected-node"><strong>Imagem</strong><span>Elemento visual</span></section><section className="property-group"><h2>General</h2><label>Nome<input defaultValue={label} onBlur={(event) => void onSave(projectId, { ...image, data: { ...image.data, label: event.target.value } })} /></label><p className="component-port-summary">Imagens não possuem Ports nem entram na simulação.</p></section><button type="button" className="delete-component-button" onClick={() => void onRemove(projectId, image.id)}>Apagar imagem</button></>;
}

export default function CanvasPage({ projectId, canvasId }: { projectId: string; canvasId?: string }) { return <ReactFlowProvider><CanvasEngine projectId={projectId} canvasId={canvasId} /></ReactFlowProvider>; }
