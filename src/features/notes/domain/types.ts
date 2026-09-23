export type Note = {
  id: string;
  projectId: string;
  title: string;
  content: string;
  contentFormat: "markdown";
  isPinned: boolean;
  isArchived: boolean;
  tags: string[];
  createdAt: number;
  updatedAt: number;
};

export type NoteFilter = "all" | "pinned" | "archived";
