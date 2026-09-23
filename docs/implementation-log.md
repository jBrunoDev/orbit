# Implementation log

## 2026-09-21

1. O projeto foi iniciado com Tauri 2, React, TypeScript e Tailwind.
2. A camada Rust usa SQLite local direto por `rusqlite`, UUIDv7 e plugin oficial de instância única do Tauri.
3. Foram adicionados commands para iniciar Local Profile e Personal Workspace, criar Local Project e listar Projects locais.
4. O Project usa `orbit.db` como catálogo e `project.db` por Project, com migrations iniciais e revision de criação.
5. O Tauri não usa servidor local no aplicativo final. Vite existe apenas durante desenvolvimento.
6. Rust MSVC e Microsoft C++ Build Tools foram instalados para compilação local.
7. `cargo fmt --check`, `cargo check` e `npm run build` passaram em 2026-09-21.
8. A Home foi implementada a partir de `Design/orbitHomePage.png`, com App Shell, tokens visuais em `src/App.css`, sidebar, hero, atalhos e lista de Projects locais.
9. A Home chama `initialize_local_profile` e `list_local_projects` do Tauri. Criar Project pela Home usa `create_local_project`, sem internet ou servidor local.
10. Foram adicionados estados de carregamento, vazio e erro para a lista de Projects, além de foco visível, navegação por teclado, atalhos Ctrl ou Cmd K e redução de movimento.
11. A Home foi revisada visualmente em uma prévia Vite. A prévia web não possui o bridge do Tauri, portanto exibe intencionalmente o estado de erro ao tentar ler SQLite. A integração real é destinada ao executável Tauri.
12. A primeira entrega visual da página Canvas foi implementada a partir de `Design/orbitCanvasPage.png`. Ela inclui a navegação local Home e Canvas, app shell próprio, toolbar, área de diagrama, minimapa e painel Properties.
13. O Canvas usa dados demonstrativos e SVGs locais. Não foram adicionados servidor local, autenticação, sync ou editor funcional nesta etapa visual.
14. `npm.cmd run build` passou após a implementação da página Canvas em 2026-09-21.
15. A estrutura do Canvas foi corrigida para reutilizar `AppShell`, `AppSidebar`, `ProjectHeader` e `OrbitIcon` compartilhados, conforme `DESIGN_RULES.md`. Sidebar e header duplicados foram removidos de `CanvasPage`.
16. O logo da `AppSidebar` foi corrigido para reutilizar o mesmo markup e estilos `orbit-mark` já usados pela Home.
17. Foram adicionados `@xyflow/react` e `zustand` para iniciar o Canvas Engine funcional, conforme o planejamento aprovado.
18. Foi adicionada a fundação TypeScript do domínio Canvas, incluindo Component Registry e validação inicial de ports.
19. A migration do Canvas foi limitada ao `project.db`, e projetos existentes na versão 1 agora recebem a versão 2 ao abrir. Foram adicionados testes Rust para os dois cenários.
20. A entrada do Canvas passou a usar `#canvas/<projectId>`. A Home abre o Canvas a partir de um Project recente ou do item mais recente da Sidebar.
21. A Home passou a usar o mesmo `AppShell` e `AppSidebar` do Canvas. A navegação duplicada foi removida.
22. O primeiro Canvas Engine local foi conectado ao `project.db`: cria um Canvas vazio no primeiro acesso, carrega Components, Ports e Connections, e registra revisions para criação, movimento, edição e conexão.
23. O Canvas passou a renderizar dados reais com React Flow. A store Zustand adapta somente o estado da UI e chama commands Tauri para persistir mudanças.

## Limites atuais

1. Importação de assets, backups de migration, login e sync ainda não foram implementados.
2. O Canvas Engine inicial cobre Components e Connections. Visual Elements livres, exclusão, resize, viewport persistido, undo e redo permanecem para a próxima etapa.
3. As especificações detalhadas anteriores não foram restauradas após a substituição da pasta pelo scaffold. Este log não tenta reconstruí las.
