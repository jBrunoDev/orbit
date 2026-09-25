import { invoke } from "@tauri-apps/api/core";
import type { Note } from "../domain/types";

export const notesGateway = {
  list: (projectId: string, archived = false) => invoke<Note[]>("list_notes", { projectId, archived }),
  create: (projectId: string) => invoke<Note>("create_note", { input: { projectId } }),
  update: (projectId: string, note: Note) => invoke<Note>("update_note", { input: { projectId, noteId: note.id, title: note.title, content: note.content, tags: note.tags } }),
  archive: (projectId: string, noteId: string, archived: boolean) => invoke("archive_note", { projectId, noteId, archived }),
  trash: (noteId: string) => invoke("trash_note", { noteId }),
  togglePin: (projectId: string, noteId: string) => invoke("toggle_note_pin", { projectId, noteId }),
  createAsset: (file: { name: string; type: string; bytes: number[] }) => invoke<{ id: string }>("create_note_asset", { input: { originalName: file.name, mediaType: file.type, bytes: file.bytes } }),
  readAsset: (assetId: string) => invoke<{ mediaType: string; bytes: number[] }>("read_note_asset", { input: { assetId } }),
};
