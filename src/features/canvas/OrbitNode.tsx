import { useEffect, useRef, useState } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { CatalogIcon } from "../catalog/icons";
import { defaultCatalogRepository } from "../catalog/repository";
import type { PersistedComponent, PersistedPort } from "./application/canvasStore";
import { useCanvasStore } from "./application/canvasStore";

type OrbitNodeData = PersistedComponent & { projectId?: string; catalogType?: string };
type NoteTemplate = "daily-planner" | "study-notes" | "meeting-notes";
type KanbanStatus = "todo" | "in-progress" | "done";
type KanbanTask = { id: string; text: string; status: KanbanStatus };

const kanbanColumns: { status: KanbanStatus; label: string }[] = [
  { status: "todo", label: "To Do" },
  { status: "in-progress", label: "In Progress" },
  { status: "done", label: "Done" },
];

const defaultPlannerTasks: KanbanTask[] = [
  { id: "daily-study", text: "Estudar", status: "todo" },
  { id: "daily-implement", text: "Implementar", status: "in-progress" },
  { id: "daily-review", text: "Revisar", status: "done" },
];

function plannerTasks(value: unknown): KanbanTask[] {
  if (!Array.isArray(value)) return defaultPlannerTasks;
  return value.flatMap((task, index) => {
    if (!task || typeof task !== "object") return [];
    const candidate = task as Partial<KanbanTask>;
    if (typeof candidate.text !== "string") return [];
    const status = candidate.status === "in-progress" || candidate.status === "done" ? candidate.status : "todo";
    return [{ id: typeof candidate.id === "string" ? candidate.id : `task-${index}`, text: candidate.text, status }];
  });
}

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
  const requestedPosition = typeof port.data?.handlePosition === "string" ? port.data.handlePosition : undefined;
  const position = requestedPosition === "top" ? Position.Top : requestedPosition === "bottom" ? Position.Bottom : requestedPosition === "right" ? Position.Right : requestedPosition === "left" ? Position.Left : isInput ? Position.Left : Position.Right;
  return <Handle id={port.id} type={isInput ? "target" : "source"} position={position} className={`orbit-handle ${port.direction}`} title={`${port.key} (${port.protocol})`} />;
}

function NoteTemplateFrame({ template }: { template: NoteTemplate }) {
  if (template === "study-notes") {
    return <div className="orbit-note-template-rail study-notes-rail" aria-label="Estrutura do Study Notes"><span>1&nbsp;&nbsp;Tópicos</span><span>2&nbsp;&nbsp;Exemplos</span><span>3&nbsp;&nbsp;Resumo</span><span>4&nbsp;&nbsp;Exercícios</span></div>;
  }
  return <div className="orbit-note-template-rail meeting-notes-rail" aria-label="Campos do Meeting Notes"><span>Data</span><span>Participantes</span><span>Pontos de ação</span></div>;
}

function DailyPlannerBoard({ component, onSave }: { component: OrbitNodeData; onSave: (tasks: KanbanTask[]) => void }) {
  const sourceTasks = plannerTasks(component.data.tasks);
  const [tasks, setTasks] = useState(sourceTasks);
  useEffect(() => setTasks(sourceTasks), [component.id, component.data.tasks]);
  const persist = (next: KanbanTask[]) => {
    setTasks(next);
    onSave(next);
  };
  const updateTask = (taskId: string, update: Partial<KanbanTask>) => persist(tasks.map((task) => task.id === taskId ? { ...task, ...update } : task));
  const addTask = () => {
    const id = typeof crypto?.randomUUID === "function" ? crypto.randomUUID() : `task-${Date.now()}`;
    persist([...tasks, { id, text: "Nova tarefa", status: "todo" }]);
  };

  return <section className="daily-planner-board nodrag nowheel" aria-label="Kanban do Daily Planner" onKeyDown={(event) => event.stopPropagation()}>
    <div className="daily-planner-columns">
      {kanbanColumns.map((column) => <section key={column.status} className={`daily-planner-column status-${column.status}`}>
        <h3>{column.label}</h3>
        <div className="daily-planner-tasks">
          {tasks.filter((task) => task.status === column.status).map((task) => <article key={task.id} className="daily-planner-task">
            <input aria-label={`Tarefa em ${column.label}`} value={task.text} onChange={(event) => setTasks((current) => current.map((item) => item.id === task.id ? { ...item, text: event.target.value } : item))} onBlur={() => updateTask(task.id, { text: task.text.trim() || "Nova tarefa" })} />
            <select aria-label={`Mover ${task.text} para outra coluna`} value={task.status} onChange={(event) => updateTask(task.id, { status: event.target.value as KanbanStatus })}>
              {kanbanColumns.map((option) => <option key={option.status} value={option.status}>{option.label}</option>)}
            </select>
          </article>)}
        </div>
      </section>)}
    </div>
    <button type="button" className="daily-planner-add-task" onClick={addTask}>+ Adicionar tarefa</button>
  </section>;
}

function NoteNode({ component, selected }: { component: OrbitNodeData; selected?: boolean }) {
  const updateComponent = useCanvasStore((state) => state.updateComponent);
  const template = typeof component.data.noteTemplate === "string" ? component.data.noteTemplate as NoteTemplate : "meeting-notes";
  const initialContent = typeof component.data.content === "string" ? component.data.content : "";
  const [draft, setDraft] = useState(initialContent);
  useEffect(() => setDraft(initialContent), [component.id, initialContent]);
  const saveContent = () => {
    if (component.projectId && draft !== initialContent) {
      void updateComponent(component.projectId, { ...component, data: { ...component.data, content: draft } });
    }
  };
  const save = (field: "label" | "description", value: string) => {
    if (component.projectId) void updateComponent(component.projectId, { ...component, [field]: value });
  };
  const saveTasks = (tasks: KanbanTask[]) => {
    if (component.projectId) void updateComponent(component.projectId, { ...component, data: { ...component.data, tasks } });
  };
  return <article className={`orbit-flow-node orbit-note-node ${template === "daily-planner" ? "daily-planner-note" : ""} ${selected ? "selected" : ""} tone-${component.color}`}>
    <header className="orbit-note-header">
      <CatalogIcon icon="note" className={`tone-${component.color}`} />
      <div className="orbit-node-copy">
        <EditableText value={component.label} className="orbit-node-title" onSave={(value) => save("label", value)} />
        <EditableText value={component.description} className="orbit-node-subtitle" onSave={(value) => save("description", value)} />
      </div>
    </header>
    {template === "daily-planner" ? <DailyPlannerBoard component={component} onSave={saveTasks} /> : <NoteTemplateFrame template={template} />}
    <label className="orbit-note-writing-field">
      <span>Notas</span>
      <textarea className="nodrag nowheel" value={draft} aria-label={`Notas de ${component.label}`} onChange={(event) => setDraft(event.target.value)} onBlur={saveContent} onKeyDown={(event) => event.stopPropagation()} />
    </label>
    {component.ports.map((port) => <NodeHandle key={port.id} port={port} />)}
  </article>;
}

export function OrbitNode({ data, selected }: NodeProps) {
  const component = data as unknown as OrbitNodeData;
  if (component.data?.canvasKind === "note") return <NoteNode component={component} selected={selected} />;
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
