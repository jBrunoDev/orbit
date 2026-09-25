import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import ReactMarkdown, { defaultUrlTransform } from "react-markdown";
import remarkGfm from "remark-gfm";
import { open, save } from "@tauri-apps/plugin-dialog";
import { AppShell } from "../../../shared/AppShell";
import { ModuleAssetImage } from "../../../shared/ModuleAssetImage";
import { OrbitIcon } from "../../../shared/OrbitIcon";
import type { OrbitDocument } from "../domain/types";
import { docsGateway } from "../infrastructure/docsGateway";
import { useDocsStore } from "../application/docsStore";
import { aiGateway, type AiSettings } from "../../ai-architecture/aiGateway";
import "./DocsPage.css";
const relative = (v: number) => {
  const m = Math.max(0, Math.round((Date.now() - v) / 60000));
  return m < 1 ? "agora" : `há ${m} min`;
};
const Icon = ({ children }: { children: string }) => (
  <span className="docs-ai-icon" aria-hidden="true">
    {children}
  </span>
);
function Popover({
  projectId,
  anchor,
  close,
}: {
  projectId: string;
  anchor: React.RefObject<HTMLButtonElement | null>;
  close: () => void;
}) {
  const [s, setS] = useState<AiSettings>();
  const [url, setUrl] = useState("");
  const [github, setGithub] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    void aiGateway.settings().then(setS);
    const k = (e: KeyboardEvent) => e.key === "Escape" && close();
    const o = (e: MouseEvent) =>
      !ref.current?.contains(e.target as Node) &&
      !anchor.current?.contains(e.target as Node) &&
      close();
    window.addEventListener("keydown", k);
    window.addEventListener("mousedown", o);
    return () => {
      window.removeEventListener("keydown", k);
      window.removeEventListener("mousedown", o);
    };
  }, [anchor, close]);
  const ready = !!s?.enabled && !!s?.hasOpenaiApiKey;
  const run = async (kind: "repository" | "document" | "folder") => {
    if (kind === "repository") {
      setGithub(true);
      return;
    }
    const p = await open(
      kind === "folder"
        ? { directory: true, multiple: false }
        : {
            multiple: false,
            filters: [
              { name: "Documentos", extensions: ["md", "txt", "pdf", "docx"] },
            ],
          },
    );
    if (p && !Array.isArray(p)) {
      await aiGateway.analyze({
        projectId,
        [kind === "folder" ? "folderPath" : "documentPath"]: p,
        consentToSendSources: true,
      });
      close();
    }
  };
  return (
    <div ref={ref} className="docs-ai-popover" role="menu">
      <strong>Analisar projeto com IA</strong>
      {!ready ? (
        <>
          <p>Configure a IA para usar este recurso</p>
          <a href="#settings/ai" onClick={close}>
            Configurar IA
          </a>
        </>
      ) : github ? (
        <div className="docs-inline-url">
          <input
            autoFocus
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            onKeyDown={(e) =>
              e.key === "Enter" &&
              void aiGateway
                .analyze({
                  projectId,
                  repositoryUrl: url,
                  consentToSendSources: true,
                })
                .then(close)
            }
            placeholder="https://github.com/owner/repo"
          />
          <button
            onClick={() =>
              void aiGateway
                .analyze({
                  projectId,
                  repositoryUrl: url,
                  consentToSendSources: true,
                })
                .then(close)
            }
          >
            Analisar
          </button>
        </div>
      ) : (
        <>
          <button onClick={() => void run("repository")}>
            <Icon>◖</Icon>GitHub
          </button>
          <button onClick={() => void run("document")}>
            <Icon>▱</Icon>Arquivo
          </button>
          <button onClick={() => void run("folder")}>
            <Icon>□</Icon>Pasta
          </button>
          <i />
          <a href="#settings/ai" onClick={close}>
            <Icon>⚙</Icon>Configurar IA
          </a>
        </>
      )}
    </div>
  );
}
function DocumentActionsMenu({
  anchor,
  restoring,
  onClose,
  onAction,
}: {
  anchor: HTMLButtonElement;
  restoring: boolean;
  onClose: () => void;
  onAction: () => void;
}) {
  const menu = useRef<HTMLDivElement>(null);
  const rect = anchor.getBoundingClientRect();
  useEffect(() => {
    menu.current?.querySelector<HTMLButtonElement>("[role='menuitem']")?.focus();
    const closeOutside = (event: PointerEvent) => {
      if (!menu.current?.contains(event.target as Node) && !anchor.contains(event.target as Node)) onClose();
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        anchor.focus();
      }
    };
    window.addEventListener("pointerdown", closeOutside);
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("pointerdown", closeOutside);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [anchor, onClose]);
  return createPortal(
    <div
      ref={menu}
      className="docs-document-menu"
      role="menu"
      aria-label="Opções do documento"
      style={{ top: Math.min(rect.bottom + 6, window.innerHeight - 44), right: window.innerWidth - rect.right }}
    >
      <button
        type="button"
        role="menuitem"
        className={restoring ? "is-restore" : "is-destructive"}
        onClick={(event) => {
          event.stopPropagation();
          onClose();
          onAction();
        }}
      >
        <OrbitIcon name={restoring ? "docs" : "delete"} />
        {restoring ? "Restaurar" : "Excluir"}
      </button>
    </div>,
    document.body,
  );
}
function Editor({ doc, projectId }: { doc: OrbitDocument; projectId: string }) {
  const saveDoc = useDocsStore((x) => x.save);
  const [d, setD] = useState({
    id: doc.id,
    title: doc.title,
    content: doc.content,
  });
  const [ai, setAi] = useState(false);
  const [assetError, setAssetError] = useState("");
  const [exportError, setExportError] = useState("");
  const [exporting, setExporting] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const contentInput = useRef<HTMLTextAreaElement>(null);
  useEffect(
    () => setD({ id: doc.id, title: doc.title, content: doc.content }),
    [doc],
  );
  useEffect(() => {
    if (d.title === doc.title && d.content === doc.content) return;
    const t = setTimeout(() => void saveDoc(d), 650);
    return () => clearTimeout(t);
  }, [d, doc, projectId, saveDoc]);
  const heads = useMemo(
    () =>
      d.content.split("\n").flatMap((l) => /^#{1,3}\s+(.+)/.exec(l)?.[1] ?? []),
    [d.content],
  );
  const markdownComponents = useMemo(() => ({
    img: ({ src, alt = "Imagem do documento" }: { src?: string; alt?: string }) => src?.startsWith("asset://") ? <ModuleAssetImage module="docs" assetId={src.slice("asset://".length)} alt={alt} /> : <img src={src} alt={alt} />,
  }), []);
  const insertImage = (file?: File) => {
    if (!file || !file.type.startsWith("image/")) return;
    setAssetError("");
    void file.arrayBuffer().then((buffer) => docsGateway.createAsset({
      name: file.name || "imagem-colada.png",
      type: file.type,
      bytes: Array.from(new Uint8Array(buffer)),
    })).then((asset) => {
      const markdown = `![${file.name || "Imagem"}](asset://${asset.id})`;
      setD((current) => {
        const input = contentInput.current;
        const start = input?.selectionStart ?? current.content.length;
        const end = input?.selectionEnd ?? start;
        return { ...current, content: `${current.content.slice(0, start)}${markdown}${current.content.slice(end)}` };
      });
    }).catch(() => setAssetError("Não foi possível salvar a imagem localmente."));
  };
  return (
    <>
      <main className="docs-main">
        <header className="docs-editor-header">
          <div>
            <span className="docs-breadcrumb">
              Docs <span>›</span> {doc.title}
            </span>
            <p>Última edição {relative(doc.updatedAt)}</p>
          </div>
          <div className="docs-header-actions">
            <span className="docs-save-state">Salvo localmente</span>
            <span className="docs-ai-anchor">
              <button
                ref={trigger}
                className={`docs-ai-trigger ${ai ? "is-open" : ""}`}
                aria-label="Analisar projeto com IA"
                title="Analisar projeto com IA"
                onClick={() => setAi(!ai)}
              >
                <Icon>✧</Icon>
              </button>
              {ai && <Popover projectId={projectId} anchor={trigger} close={() => setAi(false)} />}
            </span>
            <button
              disabled={exporting}
              onClick={() => void (async () => {
                setExportError("");
                setExporting(true);
                try {
                  await saveDoc(d);
                  const p = await save({
                    defaultPath: `${d.title || doc.slug || "document"}.md`,
                    filters: [{ name: "Markdown", extensions: ["md"] }],
                  });
                  if (p) await docsGateway.exportMarkdown(d.id, p);
                } catch (error) {
                  setExportError(error instanceof Error ? error.message : "Não foi possível exportar o Markdown.");
                } finally {
                  setExporting(false);
                }
              })()}
            >
              {exporting ? "Exportando..." : "Exportar .md"}
            </button>
            {exportError && <span className="docs-save-state is-error" role="alert">{exportError}</span>}
          </div>
        </header>
        <div className="docs-workspace">
          <section className="docs-source-pane">
            <input
              className="docs-title-input"
              value={d.title}
              onChange={(e) => setD({ ...d, title: e.target.value })}
            />
            <textarea
              ref={contentInput}
              value={d.content}
              onChange={(e) => setD({ ...d, content: e.target.value })}
              onPaste={(event) => {
                const image = Array.from(event.clipboardData.files).find((file) => file.type.startsWith("image/")) ?? Array.from(event.clipboardData.items).map((item) => item.type.startsWith("image/") ? item.getAsFile() : null).find((file): file is File => Boolean(file));
                if (!image) return;
                event.preventDefault();
                insertImage(image);
              }}
              placeholder={
                "# Visão geral\n\nDocumente a arquitetura, decisões e fluxos do seu projeto."
              }
              aria-label="Conteúdo em Markdown. Cole imagens aqui para anexá-las ao documento."
            />
            {assetError && <p className="docs-asset-error" role="alert">{assetError}</p>}
          </section>
          <section className="docs-preview-pane">
            {d.content.trim() ? (
              <article className="docs-markdown">
                <ReactMarkdown remarkPlugins={[remarkGfm]} urlTransform={(url) => url.startsWith("asset://") ? url : defaultUrlTransform(url)} components={markdownComponents}>
                  {d.content}
                </ReactMarkdown>
              </article>
            ) : (
              <div className="docs-preview-empty">
                <OrbitIcon name="docs" />
                <p>O preview do seu Markdown aparece aqui.</p>
                <button onClick={() => setAi(true)}>
                  ou gerar a partir de um repositório com IA
                </button>
              </div>
            )}
          </section>
        </div>
      </main>
      <aside className="docs-toc">
        <p className="docs-panel-title">Índice</p>
        {heads.length ? (
          heads.map((h) => <button key={h}>{h}</button>)
        ) : (
          <span>Adicione títulos para criar o índice.</span>
        )}
      </aside>
    </>
  );
}
export default function DocsPage({ documentId }: { documentId?: string }) {
  const projectId = "docs";
  const s = useDocsStore();
  const [trash, setTrash] = useState(false);
  const [q, setQ] = useState("");
  const [menu, setMenu] = useState<{ document: OrbitDocument; anchor: HTMLButtonElement }>();
  useEffect(() => {
    void s.load();
    void s.loadTrash();
  }, [projectId]);
  useEffect(() => {
    if (documentId) s.select(documentId);
  }, [documentId]);
  const docs = (trash ? s.trash : s.documents).filter((x) =>
    x.title.toLowerCase().includes(q.toLowerCase()),
  );
  const selected = s.documents.find(
    (x) => x.id === s.selectedId && x.kind === "page",
  );
  return (
    <AppShell
      activeSection="docs"
      onNavigate={(x) =>
        (window.location.hash = [
          "canvas",
          "notes",
          "calendar",
          "docs",
        ].includes(x)
          ? `#${x}/${projectId}`
          : `#${x}`)
      }
    >
      <div className="docs-page">
        <aside className="docs-library">
          <div className="docs-library-heading">
            <div>
              <h1>Docs</h1>
              <p>Organize seu conhecimento.</p>
            </div>
            <button
              className="docs-new-button"
              onClick={() => void s.create()}
            >
              <OrbitIcon name="plus" />
              Novo documento
            </button>
          </div>
          <input
            className="docs-search"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Pesquisar docs..."
          />
          <div className="docs-library-tabs">
            <button
              className={!trash ? "is-active" : ""}
              onClick={() => setTrash(false)}
            >
              Todos
            </button>
            <button
              className={trash ? "is-active" : ""}
              onClick={() => setTrash(true)}
            >
              Lixeira
            </button>
          </div>
          <div className="docs-tree">
            <p className="docs-tree-label">{trash ? "Lixeira" : "Docs"}</p>
            {docs.map((d) => (
              <div className="docs-tree-item" key={d.id}>
                <button onClick={() => s.select(d.id)}>
                  <OrbitIcon name="docs" />
                  <span>
                    <strong title={d.title}>{d.title}</strong>
                    <small>Editado {relative(d.updatedAt)}</small>
                  </span>
                </button>
                <button
                  className="docs-item-action"
                  type="button"
                  aria-label="Mais opções"
                  aria-haspopup="menu"
                  aria-expanded={menu?.document.id === d.id}
                  onClick={(event) => {
                    event.stopPropagation();
                    setMenu(menu?.document.id === d.id ? undefined : { document: d, anchor: event.currentTarget });
                  }}
                >
                  <OrbitIcon name="more" />
                </button>
              </div>
            ))}
          </div>
          {!trash && (
            <button
              className="docs-new-folder"
              onClick={() => void s.create("folder")}
            >
              + Nova pasta
            </button>
          )}
        </aside>
        {s.loading ? (
          <main className="docs-status">Carregando Docs locais…</main>
        ) : selected ? (
          <Editor doc={selected} projectId={projectId} />
        ) : (
          <main className="docs-status">Crie um documento para começar.</main>
        )}
      </div>
      {menu && <DocumentActionsMenu anchor={menu.anchor} restoring={trash} onClose={() => setMenu(undefined)} onAction={() => void (trash ? s.restore(menu.document.id) : s.trashDocument(menu.document.id))} />}
    </AppShell>
  );
}
