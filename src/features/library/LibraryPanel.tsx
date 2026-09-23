import { useMemo, useState } from "react";
import { CatalogIcon } from "../catalog/icons";
import { defaultCatalogRepository } from "../catalog/repository";
import type { CatalogItem } from "../catalog/types";

export function LibraryPanel({ onAdd }: { onAdd: (item: CatalogItem) => void }) {
  const [query, setQuery] = useState("");
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const items = useMemo(() => defaultCatalogRepository.search(query), [query]);
  const categories = defaultCatalogRepository.getCategories();

  return <aside className="library-panel" aria-label="Biblioteca de Components">
    <header className="library-header"><div><strong>Library</strong><span>Components</span></div></header>
    <label className="library-search"><span className="sr-only">Buscar Components</span><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search components" /><kbd>⌘ K</kbd></label>
    <div className="library-list">
      {categories.map((category) => {
        const categoryItems = items.filter((item) => item.category === category);
        if (!categoryItems.length) return null;
        const isCollapsed = collapsed[category] ?? false;
        return <section key={category} className="library-category">
          <button type="button" className="library-category-toggle" aria-expanded={!isCollapsed} onClick={() => setCollapsed((current) => ({ ...current, [category]: !isCollapsed }))}><span>{category}</span><span aria-hidden="true">{isCollapsed ? "＋" : "−"}</span></button>
          {!isCollapsed && <div className="library-items">{categoryItems.map((item) => <button key={item.type} type="button" draggable onDragStart={(event) => { event.dataTransfer.effectAllowed = "copy"; event.dataTransfer.setData("application/orbit-node", item.type); }} onClick={() => onAdd(item)} className="library-item"><CatalogIcon icon={item.icon} className={`tone-${item.color}`} /><span>{item.label}</span></button>)}</div>}
        </section>;
      })}
      {!items.length && <div className="library-empty"><strong>No components found</strong><span>Try another name or category.</span></div>}
    </div>
  </aside>;
}
