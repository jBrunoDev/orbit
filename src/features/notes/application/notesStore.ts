import { create } from "zustand";
import { isTauriAvailable } from "../../../shared/tauri";
import type { Note, NoteFilter } from "../domain/types";
import { notesGateway } from "../infrastructure/notesGateway";

type NotesState = { notes: Note[]; selectedId?: string; filter: NoteFilter; query: string; loading: boolean; error?: string; load: (projectId: string) => Promise<void>; create: (projectId: string) => Promise<void>; save: (projectId: string, note: Note) => Promise<void>; archive: (projectId: string, noteId: string, archived: boolean) => Promise<void>; pin: (projectId: string, noteId: string) => Promise<void>; setFilter: (filter: NoteFilter) => void; setQuery: (query: string) => void; select: (id?: string) => void };

export const useNotesStore = create<NotesState>((set, get) => ({
  notes: [], filter: "all", query: "", loading: true,
  select: (selectedId) => set({ selectedId }), setFilter: (filter) => set({ filter }), setQuery: (query) => set({ query }),
  load: async (projectId) => { if (!isTauriAvailable()) { set({ loading: false, error: "Abra o Orbit pelo aplicativo desktop para usar Notes locais." }); return; } set({ loading: true, error: undefined }); try { const notes = await notesGateway.list(projectId); set({ notes, selectedId: get().selectedId ?? notes[0]?.id, loading: false }); } catch (error) { set({ loading: false, error: error instanceof Error ? error.message : "Não foi possível carregar as Notes." }); } },
  create: async (projectId) => { const note = await notesGateway.create(projectId); set((state) => ({ notes: [note, ...state.notes], selectedId: note.id, filter: "all" })); },
  save: async (projectId, note) => { const saved = await notesGateway.update(projectId, note); set((state) => ({ notes: state.notes.map((item) => item.id === saved.id ? saved : item) })); },
  archive: async (projectId, noteId, archived) => { await notesGateway.archive(projectId, noteId, archived); await get().load(projectId); },
  pin: async (projectId, noteId) => { await notesGateway.togglePin(projectId, noteId); await get().load(projectId); },
}));
