import { invoke } from "@tauri-apps/api/core";
import type { DocKind, OrbitDocument } from "../domain/types";

export const docsGateway = {
  list: (projectId: string, deleted = false) => invoke<OrbitDocument[]>("list_docs", { projectId, deleted }),
  create: (projectId: string, options?: { title?: string; parentId?: string; kind?: DocKind }) => invoke<OrbitDocument>("create_doc", { input: { projectId, ...options } }),
  get: (projectId: string, documentId: string) => invoke<OrbitDocument>("get_doc", { projectId, documentId }),
  update: (projectId: string, document: Pick<OrbitDocument, "id" | "title" | "content">) => invoke<OrbitDocument>("update_doc", { input: { projectId, documentId: document.id, title: document.title, content: document.content } }),
  trash: (projectId: string, documentId: string) => invoke<void>("trash_doc", { input: { projectId, documentId } }),
  restore: (projectId: string, documentId: string) => invoke<void>("restore_doc", { input: { projectId, documentId } }),
  exportMarkdown: (projectId: string, documentId: string, destinationPath: string) => invoke<void>("export_doc_markdown", { input: { projectId, documentId, destinationPath } }),
};
