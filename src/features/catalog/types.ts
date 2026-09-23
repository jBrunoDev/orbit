export type CatalogPortDirection = "input" | "output" | "bidirectional";
export type CatalogPortProtocol = "http" | "sql" | "data" | "event";

export type CatalogPort = {
  key: string;
  direction: CatalogPortDirection;
  protocol: CatalogPortProtocol;
};

export type CatalogItem = {
  type: string;
  libraryId?: string;
  label: string;
  category: string;
  icon: string;
  color: string;
  description: string;
  tags?: string[];
  assetPath?: string;
  iconLoader?: () => Promise<string>;
  ports: CatalogPort[];
  simulationDefaults: Record<string, unknown>;
};
