import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppShell } from "../../shared/AppShell";
import { isTauriAvailable } from "../../shared/tauri";
import { CatalogIcon } from "../catalog/icons";
import { createCanvasCatalogRepository, defaultCatalogRepository } from "../catalog/repository";
import { awsPreviewItems, awsServiceItems } from "../catalog/awsServices";
import { useComponentLibraryStore } from "../catalog/application/componentLibraryStore";
import type { CatalogItem } from "../catalog/types";
import "./ComponentsPage.css";

const featured = ["api-server", "database", "load-balancer", "cache"];
const categories = ["All", "Clients", "Compute", "Data", "Networking", "Tech/Infra"];
type LocalProject = { id: string };
type PersonalLibraryItem = { sourceKey: string };

function tags(item: CatalogItem) { return item.tags?.length ? item.tags : [item.category, item.type.includes("api") ? "API" : "Infra"]; }

export default function ComponentsPage() {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("All");
  const [selected, setSelected] = useState<CatalogItem>(() => defaultCatalogRepository.getItemByType("api-server")!);
  const [awsCatalogOpen, setAwsCatalogOpen] = useState(false);
  const [libraryKeys, setLibraryKeys] = useState<Set<string>>(() => new Set());
  const [isAdding, setIsAdding] = useState(false);
  const [libraryMessage, setLibraryMessage] = useState("");
  const { libraries, loading: librariesLoading, load: loadLibraries, setAwsEnabled } = useComponentLibraryStore();
  const awsEnabled = libraries.some((library) => library.id === "aws" && library.enabled);
  const activeCatalog = useMemo(() => createCanvasCatalogRepository(awsEnabled), [awsEnabled]);
  const items = useMemo(() => activeCatalog.search(query).filter((item) => category === "All" || item.category === category), [activeCatalog, query, category]);
  const featuredItems = featured.map((type) => defaultCatalogRepository.getItemByType(type)).filter((item): item is CatalogItem => Boolean(item));
  const choose = (item: CatalogItem) => setSelected(item);

  useEffect(() => {
    if (!isTauriAvailable()) return;
    void loadLibraries();
    void invoke("initialize_local_profile")
      .then(() => invoke<PersonalLibraryItem[]>("list_personal_library"))
      .then((personalLibrary) => setLibraryKeys(new Set(personalLibrary.map((item) => item.sourceKey))))
      .catch(() => setLibraryMessage("Não foi possível carregar sua biblioteca local."));
  }, [loadLibraries]);

  const toggleAws = async () => {
    try {
      await setAwsEnabled(!awsEnabled);
      setLibraryMessage(awsEnabled ? "A coleção AWS foi ocultada da Library." : "A coleção AWS foi habilitada na Library.");
    } catch (reason) {
      setLibraryMessage(reason instanceof Error ? reason.message : "Não foi possível salvar a coleção AWS.");
    }
  };

  const addToLibrary = async (item: CatalogItem) => {
    if (libraryKeys.has(item.type)) {
      setLibraryMessage(`${item.label} já está na sua biblioteca.`);
      return;
    }
    if (!isTauriAvailable()) {
      setLibraryMessage("Abra o Orbit pelo aplicativo desktop para alterar sua biblioteca.");
      return;
    }
    setIsAdding(true);
    setLibraryMessage("");
    try {
      await invoke<PersonalLibraryItem>("add_library_component", { sourceKey: item.type });
      setLibraryKeys((current) => new Set([...current, item.type]));
      setLibraryMessage(`${item.label} foi adicionado à sua biblioteca.`);
    } catch (reason) {
      setLibraryMessage(reason instanceof Error ? reason.message : "Não foi possível adicionar o Component à biblioteca.");
    } finally {
      setIsAdding(false);
    }
  };

  const openAwsCatalog = () => setAwsCatalogOpen(true);
  const navigate = async (section: string) => {
    if (section === "home") {
      window.location.hash = "";
      return;
    }
    if (section === "canvas" || section === "calendar" || section === "docs") {
      if (!isTauriAvailable()) {
        window.location.hash = "";
        return;
      }
      try {
        const projects = await invoke<LocalProject[]>("list_local_projects");
        window.location.hash = projects[0] ? `${section}/${encodeURIComponent(projects[0].id)}` : "";
      } catch {
        window.location.hash = "";
      }
      return;
    }
    window.location.hash = section;
  };

  return <AppShell activeSection="components" onNavigate={(section) => { void navigate(section); }}>
    <main className="components-page">
      <section className="components-content" aria-labelledby="components-title">
        {awsCatalogOpen ? <AwsCatalog items={awsServiceItems} onBack={() => setAwsCatalogOpen(false)} onSelect={choose} onAdd={addToLibrary} libraryKeys={libraryKeys} selected={selected} /> : <>
        <header className="components-topbar"><label className="components-search"><span aria-hidden="true">⌕</span><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search components..." aria-label="Buscar Components" /><kbd>⌘ K</kbd></label><button className="new-component">＋ New Component</button></header>
        <h1 id="components-title">Components</h1><p className="components-subtitle">Blocos prontos para construir seus diagramas, arquiteturas e fluxos.</p>
        <nav className="component-filters" aria-label="Categorias de Components">{categories.map((item) => <button key={item} className={category === item ? "active" : ""} onClick={() => setCategory(item)}>{item}</button>)}</nav>
        {!query && <section className="component-section"><div className="section-title"><h2>Featured</h2><button>See all →</button></div><div className="featured-grid">{featuredItems.map((item) => <ComponentCard key={item.type} item={item} selected={selected.type === item.type} onSelect={choose} onAdd={addToLibrary} inLibrary={libraryKeys.has(item.type)} featured />)}</div></section>}
        {!query && <section className="aws-callout"><div><span className="aws-label">AWS ARCHITECTURE ICONS</span><h2>Serviços AWS locais</h2><p>{awsEnabled ? "A coleção está disponível na sua biblioteca global." : "Habilite a coleção quando quiser usar serviços AWS nos seus diagramas."}</p></div><button className={`aws-toggle ${awsEnabled ? "enabled" : ""}`} type="button" role="switch" aria-checked={awsEnabled} aria-label="Habilitar biblioteca AWS" disabled={librariesLoading} onClick={() => void toggleAws()}><span aria-hidden="true" /></button></section>}
        {!query && awsEnabled && <section className="component-section aws-preview"><div className="section-title"><h2>AWS Architecture Icons</h2><button onClick={openAwsCatalog}>Ver todos os componentes AWS →</button></div><div className="component-grid">{awsPreviewItems.map((item) => <ComponentCard key={item.type} item={item} selected={selected.type === item.type} onSelect={choose} onAdd={addToLibrary} inLibrary={libraryKeys.has(item.type)} />)}</div></section>}
        <section className="component-section"><div className="section-title"><h2>{query ? "Resultados" : "All Components"}</h2><span>{items.length} Components</span></div><div className="component-grid">{items.map((item) => <ComponentCard key={item.type} item={item} selected={selected.type === item.type} onSelect={choose} onAdd={addToLibrary} inLibrary={libraryKeys.has(item.type)} />)}</div>{items.length === 0 && <p className="components-empty">Nenhum Component encontrado.</p>}</section>
        </>}
      </section>
      <aside className="component-inspector" aria-label="Detalhes do Component"><header><span>Component</span><button aria-label="Favoritar Component">☆</button></header><div className="inspector-hero"><ComponentVisual item={selected} /><h2>{selected.label}</h2><p>{selected.description}</p><div className="component-tags">{tags(selected).map((tag) => <span key={tag}>{tag}</span>)}</div><button className="add-library" onClick={() => void addToLibrary(selected)} disabled={isAdding || libraryKeys.has(selected.type)}>{libraryKeys.has(selected.type) ? "✓ Na sua biblioteca" : isAdding ? "Adicionando..." : "＋ Adicionar à biblioteca"}</button>{libraryMessage && <p className="library-message" role="status">{libraryMessage}</p>}</div><section className="component-preview"><h3>Preview</h3><div><i>Request</i><b>●</b><ComponentVisual item={selected} /><strong>{selected.label}</strong></div></section><section className="component-properties"><div className="inspector-tabs"><button className="active">Properties</button><button>Documentation</button><button>Examples</button></div><label>Name<input value={selected.label} readOnly /></label><label>Description<textarea value={selected.description} readOnly /></label><label>Color<span className={`color-value tone-${selected.color}`}>■ {selected.color}</span></label></section></aside>
    </main>
  </AppShell>;
}

function ComponentVisual({ item }: { item: CatalogItem }) {
  return item.assetPath ? <img className="component-asset-icon" src={item.assetPath} alt="" /> : <CatalogIcon icon={item.icon} className={`tone-${item.color}`} />;
}

function ComponentCard({ item, selected, onSelect, onAdd, inLibrary, featured = false }: { item: CatalogItem; selected: boolean; onSelect: (item: CatalogItem) => void; onAdd: (item: CatalogItem) => Promise<void>; inLibrary: boolean; featured?: boolean }) {
  return <article className={`component-card ${selected ? "selected" : ""} ${featured ? "featured" : ""}`}><button className="component-card-main" onClick={() => onSelect(item)}><ComponentVisual item={item} /><strong>{item.label}</strong><p>{item.description}</p><div className="component-tags">{tags(item).slice(0, 2).map((tag) => <span key={tag}>{tag}</span>)}</div></button><button className="card-add" aria-label={inLibrary ? `${item.label} já está na biblioteca` : `Adicionar ${item.label} à biblioteca`} onClick={() => void onAdd(item)} disabled={inLibrary}>{inLibrary ? "✓" : "＋"}</button>{featured && <button className="card-favorite" aria-label={`Favoritar ${item.label}`}>☆</button>}</article>;
}

function AwsCatalog({ items, onBack, onSelect, onAdd, libraryKeys, selected }: { items: CatalogItem[]; onBack: () => void; onSelect: (item: CatalogItem) => void; onAdd: (item: CatalogItem) => Promise<void>; libraryKeys: Set<string>; selected: CatalogItem }) {
  return <section className="aws-catalog" aria-labelledby="aws-catalog-title"><header className="aws-catalog-header"><button onClick={onBack}>← Components</button><div><span className="aws-label">AWS ARCHITECTURE ICONS</span><h1 id="aws-catalog-title">Todos os componentes AWS</h1><p>Serviços AWS locais disponíveis para adicionar à sua biblioteca.</p></div></header><div className="section-title"><h2>Serviços AWS</h2><span>{items.length} componentes disponíveis</span></div><div className="component-grid">{items.map((item) => <ComponentCard key={item.type} item={item} selected={selected.type === item.type} onSelect={onSelect} onAdd={onAdd} inLibrary={libraryKeys.has(item.type)} />)}</div></section>;
}
