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
  load: () => Promise<void>; loadTrash: () => Promise<void>; create: (kind?: DocKind) => Promise<OrbitDocument | undefined>; save: (document: Pick<OrbitDocument, "id" | "title" | "content">) => Promise<void>; trashDocument: (documentId: string) => Promise<void>; restore: (documentId: string) => Promise<void>;
  select: (id?: string) => void;
};

export const useDocsStore = create<DocsState>((set, get) => ({
  documents: [], trash: [], loading: true,
  select: (selectedId) => set({ selectedId }),
  load: async () => {
    if (!isTauriAvailable()) { set({ loading: false, error: "Abra o Orbit pelo aplicativo desktop para usar Docs locais." }); return; }
    set({ loading: true, error: undefined });
    try {
      const documents = await docsGateway.list();
      set({ documents, selectedId: documents.some((item) => item.id === get().selectedId) ? get().selectedId : documents.find((item) => item.kind === "page")?.id, loading: false });
    } catch (error) { set({ loading: false, error: error instanceof Error ? error.message : "Não foi possível carregar os Docs." }); }
  },
  loadTrash: async () => { try { set({ trash: await docsGateway.list(true) }); } catch (error) { set({ error: error instanceof Error ? error.message : "Não foi possível abrir a lixeira." }); } },
  create: async (kind = "page") => {
    const document = await docsGateway.create({ kind });
    set((state) => ({ documents: [document, ...state.documents], selectedId: document.kind === "page" ? document.id : state.selectedId }));
    return document;
  },
  save: async (document) => {
    const saved = await docsGateway.update(document);
    set((state) => ({ documents: state.documents.map((item) => item.id === saved.id ? saved : item) }));
  },
  trashDocument: async (documentId) => { await docsGateway.trash(documentId); set((state) => ({ documents: state.documents.filter((item) => item.id !== documentId && item.parentId !== documentId), selectedId: state.selectedId === documentId ? undefined : state.selectedId })); await get().loadTrash(); },
  restore: async (documentId) => { await docsGateway.restore(documentId); await get().load(); await get().loadTrash(); },
}));
