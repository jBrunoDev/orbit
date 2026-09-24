import { create } from "zustand";
import { isTauriAvailable } from "../../../shared/tauri";
import type { DocKind, OrbitDocument } from "../domain/types";
import { docsGateway } from "../infrastructure/docsGateway";

type DocsState = {
  documents: OrbitDocument[];
  trash: OrbitDocument[];
  selectedId?: string;
  loading: boolean;
  error?: string;
  load: (projectId: string) => Promise<void>;
  loadTrash: (projectId: string) => Promise<void>;
  create: (projectId: string, kind?: DocKind) => Promise<OrbitDocument | undefined>;
  save: (projectId: string, document: Pick<OrbitDocument, "id" | "title" | "content">) => Promise<void>;
  trashDocument: (projectId: string, documentId: string) => Promise<void>;
  restore: (projectId: string, documentId: string) => Promise<void>;
  select: (id?: string) => void;
};

export const useDocsStore = create<DocsState>((set, get) => ({
  documents: [], trash: [], loading: true,
  select: (selectedId) => set({ selectedId }),
  load: async (projectId) => {
    if (!isTauriAvailable()) { set({ loading: false, error: "Abra o Orbit pelo aplicativo desktop para usar Docs locais." }); return; }
    set({ loading: true, error: undefined });
    try {
      const documents = await docsGateway.list(projectId);
      set({ documents, selectedId: documents.some((item) => item.id === get().selectedId) ? get().selectedId : documents.find((item) => item.kind === "page")?.id, loading: false });
    } catch (error) { set({ loading: false, error: error instanceof Error ? error.message : "Não foi possível carregar os Docs." }); }
  },
  loadTrash: async (projectId) => { try { set({ trash: await docsGateway.list(projectId, true) }); } catch (error) { set({ error: error instanceof Error ? error.message : "Não foi possível abrir a lixeira." }); } },
  create: async (projectId, kind = "page") => {
    const document = await docsGateway.create(projectId, { kind });
    set((state) => ({ documents: [document, ...state.documents], selectedId: document.kind === "page" ? document.id : state.selectedId }));
    return document;
  },
  save: async (projectId, document) => {
    const saved = await docsGateway.update(projectId, document);
    set((state) => ({ documents: state.documents.map((item) => item.id === saved.id ? saved : item) }));
  },
  trashDocument: async (projectId, documentId) => { await docsGateway.trash(projectId, documentId); set((state) => ({ documents: state.documents.filter((item) => item.id !== documentId && item.parentId !== documentId), selectedId: state.selectedId === documentId ? undefined : state.selectedId })); await get().loadTrash(projectId); },
  restore: async (projectId, documentId) => { await docsGateway.restore(projectId, documentId); await get().load(projectId); await get().loadTrash(projectId); },
}));
