import { invoke } from "@tauri-apps/api/core";
import type { DocKind, OrbitDocument } from "../domain/types";

export const docsGateway = {
  list: (deleted = false) => invoke<OrbitDocument[]>("list_docs", { deleted }),
  create: (options?: { title?: string; parentId?: string; kind?: DocKind }) => invoke<OrbitDocument>("create_doc", { input: options }),
  get: (documentId: string) => invoke<OrbitDocument>("get_doc", { documentId }),
  update: (document: Pick<OrbitDocument, "id" | "title" | "content">) => invoke<OrbitDocument>("update_doc", { input: { documentId: document.id, title: document.title, content: document.content } }),
  trash: (documentId: string) => invoke<void>("trash_doc", { input: { documentId } }),
  restore: (documentId: string) => invoke<void>("restore_doc", { input: { documentId } }),
  exportMarkdown: (documentId: string, destinationPath: string) => invoke<void>("export_doc_markdown", { input: { documentId, destinationPath } }),
  createAsset: (file: { name: string; type: string; bytes: number[] }) => invoke<{ id: string }>("create_doc_asset", { input: { originalName: file.name, mediaType: file.type, bytes: file.bytes } }),
  readAsset: (assetId: string) => invoke<{ mediaType: string; bytes: number[] }>("read_doc_asset", { input: { assetId } }),
};
