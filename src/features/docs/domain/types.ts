export type DocKind = "page" | "folder";

export type OrbitDocument = {
  id: string;
  projectId: string;
  spaceId?: string;
  parentId?: string;
  kind: DocKind;
  title: string;
  slug: string;
  content: string;
  contentFormat: "markdown";
  sortOrder: number;
  origin: "manual" | "ai" | string;
  createdAt: number;
  updatedAt: number;
};
