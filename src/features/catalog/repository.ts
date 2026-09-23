import catalogItems from "./catalog";
import type { CatalogItem } from "./types";

export interface CatalogRepository {
  getCategories(): string[];
  getItems(category?: string): CatalogItem[];
  getItemByType(type: string): CatalogItem | undefined;
  search(query: string): CatalogItem[];
}

function normalize(value: string) {
  return value
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLocaleLowerCase();
}

export class InMemoryCatalogRepository implements CatalogRepository {
  private readonly items: CatalogItem[];

  constructor(items: CatalogItem[] = catalogItems) {
    this.items = items;
  }

  getCategories() {
    return [...new Set(this.items.map((item) => item.category))];
  }

  getItems(category?: string) {
    if (!category) return [...this.items];
    const target = normalize(category);
    return this.items.filter((item) => normalize(item.category) === target);
  }

  getItemByType(type: string) {
    return this.items.find((item) => item.type === type);
  }

  search(query: string) {
    const target = normalize(query.trim());
    if (!target) return [...this.items];
    return this.items.filter((item) =>
      [item.label, item.category, item.type, ...(item.tags ?? [])].some((value) => normalize(value).includes(target)),
    );
  }
}

export const defaultCatalogRepository = new InMemoryCatalogRepository();
