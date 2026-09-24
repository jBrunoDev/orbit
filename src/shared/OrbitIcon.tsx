export type OrbitIconName = string;

const iconFiles = import.meta.glob<string>("../../assets/icons/*.svg", { eager: true, import: "default", query: "?raw" });
const assets: Record<OrbitIconName, string> = {
  home: iconFiles["../../assets/icons/nav-home.svg"], canvas: iconFiles["../../assets/icons/nav-canvas.svg"], notes: iconFiles["../../assets/icons/nav-notes.svg"], calendar: iconFiles["../../assets/icons/template-planning.svg"], templates: iconFiles["../../assets/icons/nav-templates.svg"], components: iconFiles["../../assets/icons/nav-components.svg"], simulate: iconFiles["../../assets/icons/nav-simulate.svg"], docs: iconFiles["../../assets/icons/nav-docs.svg"], settings: iconFiles["../../assets/icons/control-settings.svg"], plus: iconFiles["../../assets/icons/action-add.svg"], select: iconFiles["../../assets/icons/canvas-select.svg"], shape: iconFiles["../../assets/icons/canvas-shape.svg"], text: iconFiles["../../assets/icons/canvas-text.svg"], image: iconFiles["../../assets/icons/canvas-image.svg"], connect: iconFiles["../../assets/icons/canvas-connect.svg"], client: iconFiles["../../assets/icons/component-client.svg"], loadBalancer: iconFiles["../../assets/icons/component-load-balancer.svg"], api: iconFiles["../../assets/icons/component-api-server.svg"], cache: iconFiles["../../assets/icons/component-cache.svg"], database: iconFiles["../../assets/icons/component-database.svg"],
};

const fallbackIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="4" y="4" width="16" height="16" rx="3"/><path d="M8 12h8M12 8v8"/></svg>`;

export function OrbitIcon({ name, className }: { name: OrbitIconName; className?: string }) { return <span aria-hidden="true" className={`orbit-icon ${className ?? ""}`} dangerouslySetInnerHTML={{ __html: assets[name as keyof typeof assets] ?? fallbackIcon }} />; }
