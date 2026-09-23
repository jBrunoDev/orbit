import { OrbitIcon } from "./OrbitIcon";

export function ProjectHeader({ title, onBack }: { title: string; onBack: () => void }) {
  return <header className="canvas-header"><div className="project-title"><button onClick={onBack} aria-label="Voltar para Home">‹</button><button aria-label="Avançar">›</button><strong>{title}</strong><span>⌑ Private</span></div><div className="project-actions"><button><OrbitIcon name="simulate" />Simulate</button><button className="share">↥ Share</button><span className="header-avatar">BG</span><button aria-label="Mais opções">•••</button></div><div className="canvas-tabs" role="tablist"><button className="active" role="tab" aria-selected="true">Canvas</button><button role="tab" aria-selected="false">Notes</button><button role="tab" aria-selected="false">Run</button></div></header>;
}
