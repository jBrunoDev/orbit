import { useEffect, useRef, useState } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { CatalogIcon } from "../catalog/icons";
import { defaultCatalogRepository } from "../catalog/repository";
import type { PersistedComponent, PersistedPort } from "./application/canvasStore";
import { useCanvasStore } from "./application/canvasStore";

type OrbitNodeData = PersistedComponent & { projectId?: string; catalogType?: string };

function EditableText({ value, className, onSave }: { value: string; className: string; onSave: (value: string) => void }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => { if (editing) inputRef.current?.focus(); }, [editing]);

  if (!editing) return <span className={className} onDoubleClick={() => { setDraft(value); setEditing(true); }}>{value}</span>;
  return <input ref={inputRef} className={`orbit-node-inline-input ${className}`} value={draft} onChange={(event) => setDraft(event.target.value)} onBlur={() => { if (draft.trim()) onSave(draft.trim()); setEditing(false); }} onKeyDown={(event) => { if (event.key === "Enter") event.currentTarget.blur(); if (event.key === "Escape") { setDraft(value); setEditing(false); } }} />;
}

function NodeHandle({ port }: { port: PersistedPort }) {
  const isInput = port.direction === "input";
  return <Handle id={port.id} type={isInput ? "target" : "source"} position={isInput ? Position.Left : Position.Right} className={`orbit-handle ${port.direction}`} title={`${port.key} (${port.protocol})`} />;
}

export function OrbitNode({ data, selected }: NodeProps) {
  const component = data as unknown as OrbitNodeData;
  const catalogItem = defaultCatalogRepository.getItemByType(component.catalogType ?? component.componentType);
  const updateComponent = useCanvasStore((state) => state.updateComponent);
  const save = (field: "label" | "description", value: string) => {
    if (component.projectId) void updateComponent(component.projectId, { ...component, [field]: value });
  };
  return <article className={`orbit-flow-node ${selected ? "selected" : ""} tone-${component.color}`}>
    <CatalogIcon icon={catalogItem?.icon} className={`tone-${component.color}`} />
    <div className="orbit-node-copy">
      <EditableText value={component.label} className="orbit-node-title" onSave={(value) => save("label", value)} />
      <EditableText value={component.description} className="orbit-node-subtitle" onSave={(value) => save("description", value)} />
    </div>
    {component.ports.map((port) => <NodeHandle key={port.id} port={port} />)}
  </article>;
}
