# DESIGN_RULES.md — Orbit

> Regras obrigatórias para manter consistência visual, reutilização e estabilidade da interface do Orbit.
> Leia este arquivo antes de criar, alterar ou refatorar qualquer página ou componente visual.

## 1. Princípio central

O Orbit deve ser construído como **um único sistema de design**, não como um conjunto de páginas independentes.

Uma nova página deve reutilizar a estrutura existente sempre que possível.

```text
ERRADO
HomeSidebar
CanvasSidebar
NotesSidebar
DocsSidebar

CERTO
AppSidebar
├── Home
├── Canvas
├── Notes
└── Docs
```

Criar uma nova tela **não autoriza redesenhar elementos compartilhados**.

## 2. Preservação visual

Ao receber uma tarefa localizada, altere somente o escopo necessário.

Exemplo:

```text
Tarefa: implementar Notes

Pode alterar:
✓ conteúdo de Notes
✓ componentes exclusivos de Notes
✓ estados necessários de Notes

Não pode alterar sem necessidade:
✗ Sidebar
✗ App Shell
✗ logo
✗ navegação global
✗ tipografia global
✗ tokens
✗ Home
✗ Canvas
```

Uma feature nova não pode causar regressão visual em features existentes.

## 3. Hierarquia obrigatória

A interface deve seguir:

```text
Design Tokens
      ↓
UI Primitives
      ↓
Shared Components
      ↓
App Shell / Layouts
      ↓
Feature Components
      ↓
Pages
```

Uma Page pode usar Shared Components. Um Shared Component não deve depender de uma Page específica.

## 4. Design Tokens

Valores globais devem possuir uma única fonte de verdade:

```text
colors
typography
spacing
radius
borders
shadows
motion
z-index
breakpoints
```

Não espalhar valores arbitrários ou hexadecimais repetidos pelas páginas. Se Tailwind for usado, centralize tokens/configuração equivalente.

Não crie uma nova cor, radius, spacing ou sombra para uma única tela sem verificar o Design System existente.

## 5. UI Primitives

Elementos básicos devem ser reutilizáveis:

```text
Button
IconButton
Input
SearchInput
Select
Tooltip
Popover
Dialog
Dropdown
Tabs
Badge
Card
Separator
ScrollArea
```

Antes de criar um primitive:

1. procure um equivalente;
2. reutilize;
3. adicione variante se necessário;
4. crie outro somente se a semântica realmente for diferente.

## 6. Shared Components

Componentes recorrentes devem existir uma única vez.

Exemplos:

```text
AppSidebar
AppHeader
ProjectSwitcher
UserProfile
SearchBar
CommandPalette
PageHeader
EmptyState
```

### Regra crítica da Sidebar

Existe **uma Sidebar compartilhada**.

As páginas não devem possuir cópias independentes dela.

```tsx
<AppShell>
  <AppSidebar />
  <PageContent />
</AppShell>
```

A página apenas informa estado, por exemplo `activeSection="canvas"`. Ela não recria a Sidebar para marcar Canvas como ativo.

## 7. App Shell

A estrutura global fica fora das páginas.

```text
AppShell
├── AppSidebar
├── MainArea
│   ├── Global/Project Header
│   └── Route Content
└── Global Overlays
```

As rotas trocam principalmente o conteúdo da área de página.

```text
/home
/canvas
/notes
/templates
/components
/simulate
/docs
/profile
```

Trocar de rota não deve recriar desnecessariamente a identidade visual global.

## 8. Layout compartilhado antes da página

Evite colocar Sidebar/Header compartilhados dentro de `CanvasPage`, `NotesPage`, etc.

Prefira:

```tsx
<AppShell>
  <CanvasPage />
</AppShell>
```

A página deve conter principalmente a feature específica.

## 9. Single Source of Truth

Informações compartilhadas devem ter uma fonte de verdade.

Exemplo:

```ts
const navigation = [
  { id: "home", label: "Home" },
  { id: "canvas", label: "Canvas" },
  { id: "notes", label: "Notes" },
  { id: "templates", label: "Templates" },
  { id: "components", label: "Components" },
  { id: "simulate", label: "Simulate" },
  { id: "docs", label: "Docs" },
];
```

Não duplicar navegação, routes, labels, ícones, tokens ou shared actions em cada página.

## 10. Variants em vez de duplicação

Prefira:

```tsx
<Button variant="primary" />
<Button variant="secondary" />
<Button variant="ghost" />
```

Evite criar `CanvasPurpleButton`, `NotesPurpleButton` e `DocsPurpleButton` se são o mesmo componente.

## 11. Feature Components

Componentes específicos permanecem dentro da feature:

```text
features/
├── canvas/
│   ├── CanvasToolbar
│   ├── CanvasMinimap
│   └── PropertiesPanel
├── notes/
│   ├── NoteEditor
│   └── NotesList
└── simulate/
    ├── SimulationTimeline
    └── MetricsPanel
```

Promova para `shared` apenas quando realmente for compartilhado.

## 12. Ordem obrigatória antes de criar uma página

Antes de implementar uma nova página:

1. ler `AGENTS.md`;
2. ler `DESIGN_RULES.md`;
3. inspecionar App Shell;
4. identificar Shared Components;
5. identificar tokens;
6. identificar padrões das telas existentes;
7. separar o que é global do que é específico;
8. somente então implementar.

Não começar copiando uma concept art diretamente para JSX.

## 13. Concept art não autoriza duplicação

Uma concept art pode mostrar Sidebar, Header ou Search Bar já existentes. Isso não significa implementá-los novamente.

```text
Concept Art
     ↓
Separar
     ↓
Global existente | Feature nova
```

Exemplo:

```text
Canvas concept
├── Sidebar          → REUTILIZAR
├── Project Header   → REUTILIZAR se compartilhado
├── Canvas Toolbar   → IMPLEMENTAR
├── Canvas Workspace → IMPLEMENTAR
└── Properties Panel → IMPLEMENTAR
```

A concept art define direção visual. A arquitetura existente define onde cada parte vive.

## 14. Não quebrar Shared Components por causa de uma tela

Uma nova tela não pode alterar um componente compartilhado de forma que outras telas quebrem.

Se precisar de variação, use props, slots, variants ou composição, preservando o contrato existente.

## 15. Contrato visual

Shared Components possuem contratos estáveis.

Exemplo da Sidebar:

```text
width
background
border
logo position
item height
icon size
spacing
active state
hover state
profile area
```

Uma feature não modifica esse contrato silenciosamente.

Mudanças globais precisam ser intencionais e validadas em todas as telas consumidoras.

## 16. CSS

Evite CSS de página que vaze para o restante do aplicativo.

Evite seletores genéricos como `button`, `svg` ou `aside` dentro de CSS específico quando puderem afetar outros componentes.

Prefira escopo claro ou a estratégia de isolamento adotada no projeto.

Não use `!important` como solução padrão para conflitos arquiteturais.

## 17. Ícones

Use a fonte oficial de ícones/assets do projeto.

Não misture arbitrariamente SVG próprio, emoji, Lucide, Iconify e CSS drawing para a mesma linguagem visual.

Se um asset oficial já existe, reutilize-o.

## 18. Estados consistentes

Componentes reutilizáveis devem compartilhar estados:

```text
default
hover
active
focus
disabled
loading
error
```

O estado ativo da Sidebar deve ter o mesmo comportamento visual em todas as páginas.

## 19. Responsividade desktop

Orbit é desktop-first, mas deve tolerar diferentes tamanhos de janela.

O layout compartilhado define comportamento de Sidebar, Main Content, Inspector, Toolbars, overflow e minimum widths.

Uma página não deve resolver responsividade alterando globalmente a Sidebar.

## 20. Regra para refatorações

Uma refatoração deve preservar comportamento e aparência existentes, salvo quando uma mudança visual for explicitamente solicitada.

Antes de alterar shared code:

```text
Quem usa isso?
Quais páginas serão afetadas?
Existe referência visual?
A mudança é realmente global?
```

## 21. Classificação de impacto

### Local
Afeta uma feature, como `CanvasToolbar`.

### Shared
Afeta múltiplas features, como `AppSidebar`, `Button`, `AppShell`.

### Global
Afeta o produto inteiro, como tokens, typography, theme ou routing shell.

Uma tarefa local não deve provocar mudanças Shared/Global sem necessidade explícita.

## 22. Não quebrar para depois corrigir

Não adote:

> "Vou quebrar o componente compartilhado agora e depois arrumo as outras páginas."

Preserve consumidores existentes durante a alteração. Mudanças de API devem ser migradas de forma controlada.

## 23. Regra de reutilização

Antes de criar qualquer componente:

```text
Já existe?
    ↓
SIM → reutilizar
    ↓
Quase → variante/composição
    ↓
NÃO → criar
```

Duplicação visual é dívida técnica.

## 24. Composição antes de forks

Prefira:

```tsx
<PageHeader
  title="Canvas"
  actions={<CanvasActions />}
/>
```

em vez de criar headers quase idênticos para cada página.

## 25. Design e domínio

Reutilização visual não significa misturar regras de negócio.

`Button` pode ser compartilhado. O comportamento de `RunSimulationButton` pertence à feature Simulation.

## 26. Checklist antes de alterar UI

```text
[ ] Li AGENTS.md
[ ] Li DESIGN_RULES.md
[ ] Inspecionei componentes existentes
[ ] Identifiquei App Shell
[ ] Identifiquei tokens
[ ] Identifiquei o escopo da mudança
[ ] Sei quais componentes são Shared
[ ] Sei quais são Feature-specific
```

## 27. Checklist depois de alterar UI

```text
[ ] A página solicitada funciona
[ ] Sidebar continua igual nas demais páginas
[ ] App Shell não sofreu regressão
[ ] Header compartilhado continua consistente
[ ] Nenhum componente foi duplicado sem necessidade
[ ] Tokens existentes foram reutilizados
[ ] Hover/focus/active continuam consistentes
[ ] CSS local não vazou globalmente
[ ] Rotas anteriores continuam funcionando
[ ] Build passa
[ ] Testes relevantes passam
```

## 28. Critério de aceite de nova página

Uma página só está pronta quando:

1. usa o App Shell existente;
2. reutiliza Sidebar existente;
3. reutiliza primitives;
4. mantém tokens;
5. adiciona apenas componentes realmente específicos;
6. não modifica outras páginas acidentalmente;
7. preserva navegação;
8. preserva identidade visual;
9. evita infraestrutura visual duplicada;
10. passa validação de regressão.

## 29. Exemplo — Canvas

```text
REUTILIZAR
├── AppShell
├── AppSidebar
├── Project Header, se compartilhado
├── Button
├── Tooltip
├── Dropdown
└── Design Tokens

CRIAR/EVOLUIR EM CANVAS
├── CanvasWorkspace
├── CanvasToolbar
├── ComponentNode
├── SmartConnection
├── CanvasMinimap
└── PropertiesPanel
```

Não criar `CanvasSidebar` se `AppSidebar` já existe.

## 30. Exemplo — Notes

```tsx
<AppShell>
  <AppSidebar />   // mesma Sidebar
  <NotesPage />    // conteúdo específico
</AppShell>
```

Não criar uma segunda Sidebar apenas porque a concept art de Notes também mostra uma.

## 31. Alterações globais deliberadas

Se Sidebar ou outro Shared Component realmente precisar mudar:

1. declarar que é uma alteração global;
2. explicar o motivo;
3. alterar o componente compartilhado;
4. validar todas as páginas consumidoras;
5. atualizar referências de design;
6. documentar a mudança.

Nunca fazer isso silenciosamente dentro de uma tarefa específica.

## 32. Regra para agentes de IA

Quando o pedido for:

> "Implemente esta página seguindo a concept art."

Interprete como:

> "Implemente a parte específica desta página dentro do sistema visual existente, reutilizando App Shell e componentes compartilhados e preservando tudo já aprovado."

Não interprete como:

> "Reconstrua toda a interface mostrada na imagem do zero."

## 33. Ordem de prioridade em conflitos

```text
1. Decisão explícita mais recente do usuário
2. DESIGN_RULES.md
3. AGENTS.md
4. Design System / tokens existentes
5. Shared Components aprovados
6. Concept art atual
7. Implementação temporária/legada
```

Se for necessário quebrar regra superior, documente o conflito antes de agir.

## 34. Regra final

**Uma nova feature deve parecer que sempre fez parte do Orbit.**

```text
Reuse
  ↓
Compose
  ↓
Extend
  ↓
Create only when necessary
```

A consistência do produto é uma responsabilidade arquitetural, não apenas estética.
