import { invoke } from "@tauri-apps/api/core";
import { FormEvent, useEffect, useMemo, useState } from "react";
import { AppShell } from "../../shared/AppShell";
import { OrbitIcon } from "../../shared/OrbitIcon";
import { isTauriAvailable } from "../../shared/tauri";
import { officialTemplates, workflowPreviewFor, type TemplateDefinition, type TemplateSection } from "./templateManifest";
import "./TemplatesPage.css";
import "./TemplatesScroll.css";

type TemplateCatalogState = { favoriteTemplateIds: string[] };
type LocalProject = { id: string; name: string };
type TemplateApplicationResult = { projectId: string; canvasId: string };
type TemplateDialogMode = "create" | "apply";
const filters = ["All", "System Design", "Architecture", "Development", "Product", "Infra", "Diagrams", "Personal", "Study"];
const sections: TemplateSection[] = ["Popular", "Development", "Personal & Study"];

function previewClass(template: TemplateDefinition) { return `template-preview preview-${template.preview}`; }

export default function TemplatesPage() {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("All");
  const [selectedId, setSelectedId] = useState(officialTemplates[0].id);
  const [favoriteIds, setFavoriteIds] = useState<Set<string>>(() => new Set());
  const [categoryView, setCategoryView] = useState<TemplateSection | null>(null);
  const [message, setMessage] = useState("");
  const [projects, setProjects] = useState<LocalProject[]>([]);
  const [projectName, setProjectName] = useState("");
  const [targetProjectId, setTargetProjectId] = useState("");
  const [dialogMode, setDialogMode] = useState<TemplateDialogMode | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const selected = officialTemplates.find((template) => template.id === selectedId) ?? officialTemplates[0];
  const matching = useMemo(() => officialTemplates.filter((template) => {
    const value = `${template.name} ${template.description} ${template.categories.join(" ")} ${template.tags.join(" ")}`.toLowerCase();
    return value.includes(query.trim().toLowerCase()) && (filter === "All" || template.categories.includes(filter));
  }), [filter, query]);
  const favorites = matching.filter((template) => favoriteIds.has(template.id));

  useEffect(() => {
    if (!isTauriAvailable()) return;
    void Promise.all([
      invoke<TemplateCatalogState>("list_templates"),
      invoke<LocalProject[]>("list_local_projects"),
    ]).then(([state, localProjects]) => {
      setFavoriteIds(new Set(state.favoriteTemplateIds));
      setProjects(localProjects);
      setTargetProjectId((current) => current || localProjects[0]?.id || "");
    }).catch(() => setMessage("Não foi possível carregar seus Templates locais."));
  }, []);

  const navigate = async (section: string) => {
    if (section === "home") { window.location.hash = ""; return; }
    if (section === "canvas" || section === "calendar" || section === "docs") {
      let latestProject: LocalProject | undefined = projects[0];
      if (!latestProject && isTauriAvailable()) {
        try {
          const localProjects = await invoke<LocalProject[]>("list_local_projects");
          setProjects(localProjects);
          latestProject = localProjects[0];
        } catch {
          latestProject = undefined;
        }
      }
      window.location.hash = latestProject ? `${section}/${encodeURIComponent(latestProject.id)}` : "";
      return;
    }
    window.location.hash = section;
  };
  const toggleFavorite = async (template: TemplateDefinition) => {
    const nextFavorite = !favoriteIds.has(template.id);
    if (!isTauriAvailable()) { setMessage("Abra o Orbit pelo aplicativo desktop para salvar favoritos."); return; }
    try {
      const state = await invoke<TemplateCatalogState>("set_template_favorite", { templateId: template.id, favorite: nextFavorite });
      setFavoriteIds(new Set(state.favoriteTemplateIds));
    } catch (reason) { setMessage(reason instanceof Error ? reason.message : "Não foi possível atualizar o favorito."); }
  };
  const openCreate = () => {
    setProjectName("");
    setMessage("");
    setDialogMode("create");
  };
  const openApply = () => {
    if (!projects.length) {
      setMessage("Crie um Project na Home antes de aplicar um Template.");
      return;
    }
    setMessage("");
    setDialogMode("apply");
  };
  const closeDialog = () => {
    if (!isCreating) setDialogMode(null);
  };
  const openTemplateTarget = (result: TemplateApplicationResult) => {
    window.location.hash = `canvas/${encodeURIComponent(result.projectId)}/${encodeURIComponent(result.canvasId)}`;
  };
  const createProjectFromTemplate = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!isTauriAvailable()) { setMessage("Abra o Orbit pelo aplicativo desktop para criar um Project pelo Template."); return; }
    if (!projectName.trim()) { setMessage("Dê um nome ao novo Project."); return; }
    setIsCreating(true); setMessage("");
    try {
      const result = await invoke<TemplateApplicationResult>("create_project_from_template", { input: { templateId: selected.id, projectName, seed: selected.seed } });
      setDialogMode(null);
      openTemplateTarget(result);
    } catch (reason) { setMessage(reason instanceof Error ? reason.message : "Não foi possível criar o Project pelo Template."); }
    finally { setIsCreating(false); }
  };
  const applyTemplate = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!isTauriAvailable()) { setMessage("Abra o Orbit pelo aplicativo desktop para aplicar um Template."); return; }
    if (!targetProjectId) { setMessage("Escolha o Project que receberá o Template."); return; }
      setIsCreating(true); setMessage("");
    try {
      const result = await invoke<TemplateApplicationResult>("apply_template_to_project", { input: { templateId: selected.id, projectId: targetProjectId, seed: selected.seed } });
      setDialogMode(null);
      openTemplateTarget(result);
    } catch (reason) { setMessage(reason instanceof Error ? reason.message : "Não foi possível aplicar o Template ao Project."); }
    finally { setIsCreating(false); }
  };
  const visibleSections = categoryView ? [categoryView] : sections;

  return <AppShell activeSection="templates" onNavigate={(section) => { void navigate(section); }}>
    <main className="templates-page">
      <section className="templates-content" aria-labelledby="templates-title">
        <header className="templates-topbar">
          <label className="templates-search"><OrbitIcon name="search" /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search templates..." aria-label="Buscar Templates" /><kbd>⌘ K</kbd></label>
          <button className="new-template" type="button" disabled title="Templates personalizados serão disponibilizados em uma etapa futura">＋ New Template</button>
        </header>
        {categoryView && <button className="templates-back" onClick={() => setCategoryView(null)}>← Templates</button>}
        <h1 id="templates-title">{categoryView ? categoryView : "Templates"}</h1>
        <p className="templates-subtitle">Comece mais rápido com templates prontos para suas ideias.</p>
        <nav className="template-filters" aria-label="Filtros de Templates">{filters.map((item) => <button key={item} className={filter === item ? "active" : ""} onClick={() => { setFilter(item); setCategoryView(null); }}>{item}</button>)}<button type="button" aria-label="Mais filtros">•••</button></nav>
        {favorites.length > 0 && !categoryView && <TemplateSection title="Favoritos" templates={favorites} selectedId={selected.id} onSelect={setSelectedId} onFavorite={toggleFavorite} favoriteIds={favoriteIds} />}
        {visibleSections.map((section) => <TemplateSection key={section} title={section} templates={matching.filter((template) => template.section === section)} selectedId={selected.id} onSelect={setSelectedId} onFavorite={toggleFavorite} favoriteIds={favoriteIds} onSeeAll={() => setCategoryView(section)} />)}
        {matching.length === 0 && <div className="templates-empty"><strong>Nenhum template encontrado.</strong><span>Ajuste sua busca ou escolha outro filtro.</span></div>}
      </section>
      <TemplateInspector template={selected} favorite={favoriteIds.has(selected.id)} onFavorite={() => void toggleFavorite(selected)} onUse={openCreate} message={message} />
    </main>
    {dialogMode && <div className="dialog-backdrop template-dialog-backdrop" role="presentation"><section className="dialog template-dialog" role="dialog" aria-modal="true" aria-labelledby="template-dialog-title"><button className="dialog-close template-dialog-close" onClick={closeDialog} aria-label="Fechar">×</button><h2 id="template-dialog-title">{dialogMode === "create" ? `Usar ${selected.name}` : `Adicionar ${selected.name}`}</h2><p>{dialogMode === "create" ? "Crie um novo Project local com este Template como base." : "Escolha um Project existente para receber este Template."}</p>{dialogMode === "create" ? <form onSubmit={(event) => void createProjectFromTemplate(event)}><label>Nome do Project<input value={projectName} onChange={(event) => setProjectName(event.target.value)} placeholder="Ex.: Arquitetura da minha aplicação" autoFocus /></label>{message && <p className="template-error" role="alert">{message}</p>}<div className="dialog-actions template-dialog-actions"><button type="button" onClick={closeDialog}>Cancelar</button><button className="use-template" disabled={isCreating || !projectName.trim()}>{isCreating ? "Criando..." : "Criar Project"}</button></div><button className="template-secondary-action" type="button" onClick={openApply} disabled={isCreating}>Adicionar a um Project existente</button></form> : <form onSubmit={(event) => void applyTemplate(event)}><label>Project<select value={targetProjectId} onChange={(event) => setTargetProjectId(event.target.value)} autoFocus>{projects.map((project) => <option key={project.id} value={project.id}>{project.name}</option>)}</select></label>{message && <p className="template-error" role="alert">{message}</p>}<div className="dialog-actions template-dialog-actions"><button type="button" onClick={closeDialog}>Cancelar</button><button className="use-template" disabled={isCreating || !targetProjectId}>{isCreating ? "Adicionando..." : "Adicionar Template"}</button></div><button className="template-secondary-action" type="button" onClick={openCreate} disabled={isCreating}>Criar novo Project com este Template</button></form>}</section></div>}
  </AppShell>;
}

function TemplateSection({ title, templates, selectedId, onSelect, onFavorite, favoriteIds, onSeeAll }: { title: string; templates: TemplateDefinition[]; selectedId: string; onSelect: (id: string) => void; onFavorite: (template: TemplateDefinition) => Promise<void>; favoriteIds: Set<string>; onSeeAll?: () => void }) {
  if (!templates.length) return null;
  return <section className="template-section"><div className="template-section-title"><h2>{title}</h2>{onSeeAll && <button onClick={onSeeAll}>See all →</button>}</div><div className="template-grid">{templates.map((template) => <article className={`template-card ${selectedId === template.id ? "selected" : ""}`} key={template.id}><button className="template-card-main" onClick={() => onSelect(template.id)}><TemplatePreview template={template} /><strong>{template.name}</strong><p>{template.description}</p><div>{template.tags.map((tag) => <span key={tag}>{tag}</span>)}</div></button><button className="template-favorite" onClick={() => void onFavorite(template)} aria-label={favoriteIds.has(template.id) ? `Remover ${template.name} dos favoritos` : `Favoritar ${template.name}`}>{favoriteIds.has(template.id) ? "★" : "☆"}</button></article>)}</div></section>;
}

function TemplatePreview({ template }: { template: TemplateDefinition }) {
  const source = workflowPreviewFor(template);
  if (source) return <div className={previewClass(template)}><img src={source} alt={`Preview de ${template.name}`} /></div>;
  return <div className={previewClass(template)} aria-hidden="true"><i /><b /><em /><span /><strong /></div>;
}

function TemplateInspector({ template, favorite, onFavorite, onUse, message }: { template: TemplateDefinition; favorite: boolean; onFavorite: () => void; onUse: () => void; message: string }) {
  return <aside className="template-inspector" aria-label="Detalhes do Template"><div className="template-inspector-preview"><TemplatePreview template={template} /></div><h2>{template.name}</h2><p>{template.description}</p><div className="template-tags">{template.tags.map((tag) => <span key={tag}>{tag}</span>)}</div><div className="template-actions"><button className="use-template" onClick={onUse}>↗ Use Template</button><button className="template-inspector-favorite" onClick={onFavorite} aria-label="Favoritar Template">{favorite ? "★" : "☆"}</button></div>{message && <p className="template-status" role="status">{message}</p>}<section><h3>O que inclui</h3><ul><li>Estrutura completa</li><li>Componentes editáveis</li><li>Boas práticas anotadas</li><li>Pronto para simulação</li></ul></section><section><h3>Tags</h3><div className="template-tags">{template.categories.concat(template.tags).map((tag) => <span key={tag}>{tag}</span>)}</div></section><section><h3>Templates relacionados</h3>{officialTemplates.filter((item) => item.id !== template.id && item.section === template.section).slice(0, 3).map((item) => <p className="related-template" key={item.id}>{item.name}<span>›</span></p>)}</section></aside>;
}
