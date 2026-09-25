import { invoke } from "@tauri-apps/api/core";
import { FormEvent, useEffect, useId, useRef, useState } from "react";
import { AppShell } from "./shared/AppShell";
import { isTauriAvailable } from "./shared/tauri";
import "./App.css";

type IconName =
  | "home"
  | "canvas"
  | "templates"
  | "components"
  | "simulate"
  | "docs"
  | "search"
  | "bell"
  | "plus"
  | "play"
  | "arrow"
  | "more"
  | "settings"
  | "close"
  | "command"
  | "select"
  | "shape"
  | "text"
  | "image"
  | "connect"
  | "client"
  | "loadBalancer"
  | "api"
  | "cache"
  | "database"
  | "systemDesign"
  | "processFlow"
  | "planning"
  | "wireframes";
type LocalProject = {
  id: string;
  workspaceId: string;
  name: string;
  storageMode: "local" | "cloud";
  createdAt: number;
  updatedAt: number;
};
type LoadState = "loading" | "ready" | "error";

const quickStarts = [
  {
    title: "System Design",
    description: "Arquitetura e infraestrutura",
    icon: "systemDesign" as IconName,
    tone: "purple",
  },
  {
    title: "Anotações",
    description: "Ideias, estudos e reuniões",
    icon: "notes" as IconName,
    tone: "green",
  },
  {
    title: "Fluxo de Processo",
    description: "Fluxos e automações",
    icon: "processFlow" as IconName,
    tone: "blue",
  },
  {
    title: "Planejamento",
    description: "Roadmaps e tarefas",
    icon: "planning" as IconName,
    tone: "yellow",
  },
  {
    title: "Wireframes",
    description: "Esboços e interfaces",
    icon: "wireframes" as IconName,
    tone: "pink",
  },
];

const iconFiles = import.meta.glob<string>("../assets/icons/*.svg", {
  eager: true,
  import: "default",
  query: "?raw",
});
const iconAssets: Record<Exclude<IconName, "close">, string> = {
  home: iconFiles["../assets/icons/nav-home.svg"],
  canvas: iconFiles["../assets/icons/nav-canvas.svg"],
  templates: iconFiles["../assets/icons/nav-templates.svg"],
  components: iconFiles["../assets/icons/nav-components.svg"],
  simulate: iconFiles["../assets/icons/nav-simulate.svg"],
  docs: iconFiles["../assets/icons/nav-docs.svg"],
  settings: iconFiles["../assets/icons/control-settings.svg"],
  search: iconFiles["../assets/icons/control-search.svg"],
  command: iconFiles["../assets/icons/control-command.svg"],
  bell: iconFiles["../assets/icons/control-notifications.svg"],
  plus: iconFiles["../assets/icons/action-add.svg"],
  arrow: iconFiles["../assets/icons/action-arrow-right.svg"],
  more: iconFiles["../assets/icons/action-more.svg"],
  play: iconFiles["../assets/icons/nav-simulate.svg"],
  systemDesign: iconFiles["../assets/icons/template-system-design.svg"],
  processFlow: iconFiles["../assets/icons/template-process-flow.svg"],
  planning: iconFiles["../assets/icons/template-planning.svg"],
  wireframes: iconFiles["../assets/icons/template-wireframes.svg"],
  select: iconFiles["../assets/icons/canvas-select.svg"],
  shape: iconFiles["../assets/icons/canvas-shape.svg"],
  text: iconFiles["../assets/icons/canvas-text.svg"],
  image: iconFiles["../assets/icons/canvas-image.svg"],
  connect: iconFiles["../assets/icons/canvas-connect.svg"],
  client: iconFiles["../assets/icons/component-client.svg"],
  loadBalancer: iconFiles["../assets/icons/component-load-balancer.svg"],
  api: iconFiles["../assets/icons/component-api-server.svg"],
  cache: iconFiles["../assets/icons/component-cache.svg"],
  database: iconFiles["../assets/icons/component-database.svg"],
};

function Icon({ name, className }: { name: IconName; className?: string }) {
  if (name === "close")
    return (
      <svg
        aria-hidden="true"
        className={className}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeWidth="1.7"
      >
        <path d="m6 6 12 12M18 6 6 18" />
      </svg>
    );
  return (
    <span
      aria-hidden="true"
      className={`icon ${className ?? ""}`}
      dangerouslySetInnerHTML={{ __html: iconAssets[name] }}
    />
  );
}

function ProjectPreview({ index }: { index: number }) {
  if (index % 3 === 1)
    return (
      <div className="preview preview-list">
        <span />
        <span />
        <span />
        <span />
      </div>
    );
  if (index % 3 === 2)
    return (
      <div className="preview preview-flow">
        <i />
        <b />
        <em />
      </div>
    );
  return (
    <div className="preview preview-graph">
      <i />
      <b />
      <em />
      <span />
      <strong />
    </div>
  );
}
function relativeTime(value: number) {
  const minutes = Math.max(0, Math.floor((Date.now() - value) / 60000));
  if (minutes < 1) return "agora";
  if (minutes < 60) return `há ${minutes} min`;
  const hours = Math.floor(minutes / 60);
  return hours < 24 ? `há ${hours} h` : `há ${Math.floor(hours / 24)} dias`;
}
function ProjectSkeleton() {
  return (
    <div className="project-card skeleton-card" aria-hidden="true">
      <div />
      <span />
      <span />
    </div>
  );
}

function App() {
  const [projects, setProjects] = useState<LocalProject[]>([]);
  const [loadState, setLoadState] = useState<LoadState>("loading");
  const [error, setError] = useState("");
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [projectToTrash, setProjectToTrash] = useState<LocalProject>();
  const [isTrashing, setIsTrashing] = useState(false);
  const [trashError, setTrashError] = useState("");
  const [openProjectMenuId, setOpenProjectMenuId] = useState<string>();
  const [projectName, setProjectName] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const [createError, setCreateError] = useState("");
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const projectInput = useRef<HTMLInputElement>(null);
  const projectNameId = useId();
  const projectErrorId = useId();
  async function loadProjects() {
    setLoadState("loading");
    setError("");
    if (!isTauriAvailable()) {
      setProjects([]);
      setLoadState("ready");
      return;
    }
    try {
      await invoke("initialize_local_profile");
      const result = await invoke<LocalProject[]>("list_local_projects");
      setProjects(result);
      setLoadState("ready");
    } catch (reason) {
      setError(
        reason instanceof Error
          ? reason.message
          : "Não foi possível carregar seus projetos locais.",
      );
      setLoadState("error");
    }
  }
  useEffect(() => {
    void loadProjects();
  }, []);
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setIsSearchOpen(true);
      }
      if (event.key === "Escape" && !isTrashing) {
        setIsCreateOpen(false);
        setIsSearchOpen(false);
        setProjectToTrash(undefined);
        setOpenProjectMenuId(undefined);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [isTrashing]);
  useEffect(() => {
    if (isCreateOpen) window.setTimeout(() => projectInput.current?.focus(), 0);
  }, [isCreateOpen]);
  function openCreateProject() {
    setProjectName("");
    setCreateError("");
    setIsCreateOpen(true);
  }
  function openCanvas(projectId: string) {
    window.location.hash = `canvas/${encodeURIComponent(projectId)}`;
  }
  function navigateFromSidebar(section: string) {
    if (section === "home") return;
    if (section === "canvas" || section === "calendar" || section === "docs") {
      const latestProject = projects[0];
      if (latestProject) window.location.hash = `${section}/${encodeURIComponent(latestProject.id)}`;
      else openCreateProject();
      return;
    }
    window.location.hash = section;
  }
  async function createProject(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!projectName.trim()) {
      setCreateError("Dê um nome ao seu Project para continuar.");
      return;
    }
    if (!isTauriAvailable()) {
      setCreateError(
        "Abra o Orbit pelo aplicativo desktop para criar um Project local.",
      );
      return;
    }
    setIsCreating(true);
    setCreateError("");
    try {
      const project = await invoke<LocalProject>("create_local_project", {
        name: projectName,
      });
      setProjects((current) => [project, ...current]);
      setIsCreateOpen(false);
    } catch (reason) {
      setCreateError(
        reason instanceof Error
          ? reason.message
          : "Não foi possível criar o Project.",
      );
    } finally {
      setIsCreating(false);
    }
  }
  function openTrashDialog(project: LocalProject) {
    setOpenProjectMenuId(undefined);
    setTrashError("");
    setProjectToTrash(project);
  }
  async function trashProject() {
    if (!projectToTrash || isTrashing) return;
    if (!isTauriAvailable()) {
      setTrashError("Abra o Orbit pelo aplicativo desktop para mover este Project à lixeira.");
      return;
    }
    setIsTrashing(true);
    setTrashError("");
    try {
      await invoke("trash_local_project", { projectId: projectToTrash.id });
      setProjects((current) => current.filter((project) => project.id !== projectToTrash.id));
      setProjectToTrash(undefined);
    } catch (reason) {
      setTrashError(reason instanceof Error ? reason.message : "Não foi possível mover o Project à lixeira.");
    } finally {
      setIsTrashing(false);
    }
  }
  return (
    <AppShell activeSection="home" onNavigate={navigateFromSidebar}>
      <a className="skip-link" href="#main-content">
        Pular para o conteúdo
      </a>
      <main id="main-content" className="home-content">
        <header className="topbar">
          <button
            className="search-trigger"
            onClick={() => setIsSearchOpen(true)}
            aria-label="Pesquisar no Orbit, atalho Control K"
          >
            <Icon name="search" />
            <span>Search canvases, templates...</span>
            <kbd>⌘ K</kbd>
          </button>
          <button
            className="icon-button notification"
            aria-label="Ver notificações"
          >
            <Icon name="bell" />
          </button>
        </header>
        <section className="hero" aria-labelledby="home-title">
          <div className="hero-copy">
            <p className="eyebrow">
              PENSE&nbsp;&nbsp;·&nbsp;&nbsp;PLANEJE&nbsp;&nbsp;·&nbsp;&nbsp;DESENHE&nbsp;&nbsp;·&nbsp;&nbsp;SIMULE
            </p>
            <h1 id="home-title">
              Do caos à <span>clareza.</span>
            </h1>
            <p className="hero-description">
              Um espaço visual para suas ideias, anotações e sistemas.
              <br />
              Organize, conecte e veja suas ideias ganharem forma.
            </p>
            <div className="hero-actions">
              <button
                className="button button-primary"
                onClick={openCreateProject}
              >
                <Icon name="plus" />
                Novo Canvas
              </button>
              <button
                className="button button-secondary"
                onClick={() => setIsSearchOpen(true)}
              >
                <Icon name="play" />
                Fazer um tour
              </button>
            </div>
          </div>
          <div
            className="hero-visual"
            aria-label="Exemplo de um fluxo visual de arquitetura"
          >
            <span className="hero-orbit" aria-hidden="true" />
            <div className="visual-heading">
              THINK&nbsp; · &nbsp;DRAW&nbsp; · &nbsp;RUN
            </div>
            <div className="canvas-demo">
              <div className="demo-toolbar">
                <span className="tool-selected">↖</span>
                <span>□</span>
                <span>T</span>
                <span>▧</span>
                <span>↗</span>
              </div>
              <div className="idea-note">
                Ideia:
                <br />• esquema de feed
                <br />• fila + workers
                <br />• cache
                <br />• simular latência
              </div>
              <div className="demo-node client">
                ▣<small>Client</small>
              </div>
              <div className="demo-node load">
                ◇<small>Load Balancer</small>
              </div>
              <div className="demo-node api first">
                ▤<small>API Server</small>
              </div>
              <div className="demo-node api second">
                ▤<small>API Server</small>
              </div>
              <div className="demo-node cache">
                ◇<small>Cache</small>
              </div>
              <div className="demo-node database">
                ▱<small>Database</small>
              </div>
              <span className="line line-one" />
              <span className="line line-two" />
              <span className="line line-three" />
              <span className="line line-four" />
              <span className="line line-five" />
              <div className="sticky">
                Melhorar:
                <br />• tratar erros
                <br />• métricas
                <br />• retry ?
              </div>
              <div className="handwriting">
                Escalar
                <br />
                horizontalmente?
              </div>
              <div className="demo-caption">
                Ideias de hoje, sistemas de amanhã.
              </div>
            </div>
          </div>
        </section>
        <section className="quick-section" aria-labelledby="quick-start-title">
          <h2 id="quick-start-title">Comece rapidamente</h2>
          <ul className="quick-grid">
            {quickStarts.map((item) => (
              <li key={item.title}>
                <button
                  className={`quick-card ${item.tone}`}
                  onClick={openCreateProject}
                >
                  <Icon name={item.icon} />
                  <strong>{item.title}</strong>
                  <span>{item.description}</span>
                  <Icon name="arrow" className="quick-arrow" />
                </button>
              </li>
            ))}
          </ul>
        </section>
        <section className="recent-section" aria-labelledby="recent-title">
          <div className="section-heading">
            <h2 id="recent-title">Recentes</h2>
            <button className="view-all" onClick={() => setIsSearchOpen(true)}>
              Ver todos <Icon name="arrow" />
            </button>
          </div>
          {loadState === "loading" && (
            <div className="recent-grid" aria-label="Carregando Projects">
              <ProjectSkeleton />
              <ProjectSkeleton />
              <ProjectSkeleton />
            </div>
          )}
          {loadState === "error" && (
            <div className="state-panel state-error" role="alert">
              <strong>Não foi possível carregar os Projects.</strong>
              <span>{error}</span>
              <button
                className="button button-secondary"
                onClick={() => void loadProjects()}
              >
                Tentar novamente
              </button>
            </div>
          )}
          {loadState === "ready" && projects.length === 0 && (
            <div className="state-panel">
              <strong>Seu espaço está pronto para começar.</strong>
              <span>
                Crie um Project local para manter Canvas e Docs no seu
                dispositivo.
              </span>
              <button
                className="button button-primary"
                onClick={openCreateProject}
              >
                <Icon name="plus" />
                Criar Project local
              </button>
            </div>
          )}
          {loadState === "ready" && projects.length > 0 && (
            <ul className="recent-grid">
              {projects.slice(0, 5).map((project, index) => (
                <li key={project.id}>
                  <article
                    className="project-card"
                    role="button"
                    tabIndex={0}
                    onClick={() => openCanvas(project.id)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        openCanvas(project.id);
                      }
                    }}
                  >
                    <ProjectPreview index={index} />
                    <div className="project-details">
                      <div>
                        <h3>{project.name}</h3>
                        <p>
                          Project local · atualizado{" "}
                          {relativeTime(project.updatedAt)}
                        </p>
                      </div>
                      <div className="project-actions-menu">
                        <button
                          className="more-button"
                          aria-label={`Mais ações para ${project.name}`}
                          aria-expanded={openProjectMenuId === project.id}
                          aria-controls={`project-menu-${project.id}`}
                          onClick={(event) => { event.stopPropagation(); setOpenProjectMenuId((current) => current === project.id ? undefined : project.id); }}
                        >
                          <Icon name="more" />
                        </button>
                        {openProjectMenuId === project.id && <div id={`project-menu-${project.id}`} className="project-menu" role="menu"><button type="button" role="menuitem" onClick={(event) => { event.stopPropagation(); openTrashDialog(project); }}>Mover para lixeira</button></div>}
                      </div>
                    </div>
                  </article>
                </li>
              ))}
            </ul>
          )}
        </section>
      </main>
      {isCreateOpen && (
        <div
          className="dialog-backdrop"
          role="presentation"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) setIsCreateOpen(false);
          }}
        >
          <section
            className="dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="create-project-title"
          >
            <button
              className="dialog-close"
              onClick={() => setIsCreateOpen(false)}
              aria-label="Fechar"
            >
              <Icon name="close" />
            </button>
            <p className="eyebrow">LOCAL FIRST</p>
            <h2 id="create-project-title">Comece seu próximo Project.</h2>
            <p>
              Ele será salvo apenas neste dispositivo. Você poderá habilitar
              sync com conta posteriormente.
            </p>
            <form onSubmit={createProject}>
              <label htmlFor={projectNameId}>Nome do Project</label>
              <input
                ref={projectInput}
                id={projectNameId}
                value={projectName}
                onChange={(event) => setProjectName(event.target.value)}
                aria-describedby={createError ? projectErrorId : undefined}
                aria-invalid={Boolean(createError)}
                placeholder="Ex.: Arquitetura do produto"
                maxLength={120}
              />
              <div id={projectErrorId} className="form-error" role="alert">
                {createError}
              </div>
              <div className="dialog-actions">
                <button
                  type="button"
                  className="button button-secondary"
                  onClick={() => setIsCreateOpen(false)}
                >
                  Cancelar
                </button>
                <button
                  type="submit"
                  className="button button-primary"
                  disabled={isCreating}
                >
                  {isCreating ? "Criando..." : "Criar Project"}
                </button>
              </div>
            </form>
          </section>
        </div>
      )}
      {projectToTrash && (
        <div className="dialog-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget && !isTrashing) setProjectToTrash(undefined); }}>
          <section className="dialog trash-project-dialog" role="dialog" aria-modal="true" aria-labelledby="trash-project-title" aria-describedby="trash-project-description">
            <button className="dialog-close" onClick={() => setProjectToTrash(undefined)} disabled={isTrashing} aria-label="Fechar"><Icon name="close" /></button>
            <p className="eyebrow">LIXEIRA LOCAL</p>
            <h2 id="trash-project-title">Mover {projectToTrash.name} para a lixeira?</h2>
            <p id="trash-project-description">O Project sairá da Home, mas seus arquivos serão mantidos localmente na lixeira. Esta ação não apaga os dados de forma permanente.</p>
            {trashError && <p className="form-error" role="alert">{trashError}</p>}
            <div className="dialog-actions">
              <button type="button" className="button button-secondary" onClick={() => setProjectToTrash(undefined)} disabled={isTrashing} autoFocus>Cancelar</button>
              <button type="button" className="button button-danger" onClick={() => void trashProject()} disabled={isTrashing}>{isTrashing ? "Movendo..." : "Mover para lixeira"}</button>
            </div>
          </section>
        </div>
      )}
      {isSearchOpen && (
        <div
          className="dialog-backdrop"
          role="presentation"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) setIsSearchOpen(false);
          }}
        >
          <section
            className="command-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="search-title"
          >
            <div className="command-input">
              <Icon name="search" />
              <input
                autoFocus
                aria-label="Pesquisar no Orbit"
                placeholder="Buscar no Orbit"
              />
              <kbd>Esc</kbd>
            </div>
            <h2 id="search-title">Atalhos</h2>
            <button onClick={openCreateProject}>
              <Icon name="plus" />
              Criar Project local
            </button>
            <button onClick={() => setIsSearchOpen(false)}>
              <Icon name="canvas" />
              Abrir Canvas quando um Project estiver criado
            </button>
          </section>
        </div>
      )}
    </AppShell>
  );
}

export default App;
