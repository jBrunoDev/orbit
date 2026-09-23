import { invoke } from "@tauri-apps/api/core";
import type { Note } from "../domain/types";

export const notesGateway = {
  list: (projectId: string, archived = false) => invoke<Note[]>("list_notes", { projectId, archived }),
  create: (projectId: string) => invoke<Note>("create_note", { input: { projectId } }),
  update: (projectId: string, note: Note) => invoke<Note>("update_note", { input: { projectId, noteId: note.id, title: note.title, content: note.content, tags: note.tags } }),
  archive: (projectId: string, noteId: string, archived: boolean) => invoke("archive_note", { projectId, noteId, archived }),
  togglePin: (projectId: string, noteId: string) => invoke("toggle_note_pin", { projectId, noteId }),
  createAsset: (projectId: string, file: { name: string; type: string; bytes: number[] }) => invoke<{ id: string }>("create_asset", { input: { projectId, originalName: file.name, mediaType: file.type, bytes: file.bytes } }),
  readAsset: (projectId: string, assetId: string) => invoke<{ mediaType: string; bytes: number[] }>("read_asset", { input: { projectId, assetId } }),
};
