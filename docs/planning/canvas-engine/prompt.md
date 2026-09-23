# Prompt de planejamento: Canvas Engine local first

## Estado

Este prompt substitui a abordagem de Canvas somente visual. Ele foi elaborado após a auditoria registrada em `docs/logs/2026-09-21-canvas-engine-audit.md` e deve ser aprovado antes da implementação.

## Decisão proposta

Implementar um primeiro Canvas Engine real, local first e vinculado a um Project local. React Flow renderiza e recebe interação, mas não é a fonte de verdade. O domínio Orbit, os commands da aplicação e SQLite definem o estado persistido.

Fluxo:

```text
React Flow UI
  ↓
Canvas Store
  ↓
Canvas commands e domínio
  ↓
Repository Tauri
  ↓
SQLite em project.db
```

## Dependências propostas

1. `@xyflow/react`, necessária porque `AGENTS.md` define React Flow como a base aprovada do graph editor e ela fornece pan, zoom, drag, handles, MiniMap e Controls.
2. `zustand`, para uma store fora de `CanvasPage`, com estado, seleção, viewport, modo e comandos. Não há solução equivalente instalada. A store será a adaptação da aplicação, não o domínio nem o banco.

Não adicionar Supabase, Clerk, servidor local ou outra biblioteca de banco.

## Modelo de domínio

### Canvas

`id`, `projectId`, `name`, `viewport`, `createdAt`, `updatedAt` e `deletedAt`.

### CanvasElement

Entidade base com `id`, `canvasId`, `kind`, `type`, `position`, `size`, `parentId`, `data`, datas de criação, atualização e exclusão.

### VisualElement

Especialização livre de CanvasElement. Tipos iniciais: text, sticky note, shape e image. Não possui ports e não entra automaticamente na simulação.

### Component

Especialização estruturada de CanvasElement. Tipos iniciais: client, api server, database, cache, load balancer e custom component. Possui label, descrição, cor, ícone, propriedades extensíveis, configuração futura de simulação e ports tipados.

### Port

Pertence a um Component. Possui `id`, `key`, `direction`, `protocol`, `data` e ordem. Protocolos iniciais: HTTP IN, HTTP OUT, SQL, DATA IN, DATA OUT, EVENT IN e EVENT OUT.

### Connection

Possui `id`, `canvasId`, `sourceComponentId`, `sourcePortId`, `targetComponentId`, `targetPortId`, `type`, `data`, `createdAt`, `updatedAt` e `deletedAt`. O objeto já reserva propriedades futuras de protocolo, latência, throughput, failure rate, timeout e retry dentro de `data`.

## Biblioteca de Components

Criar definições reutilizáveis fora do JSX. Cada definição declara label, categoria, ícone, propriedades padrão, ports padrão e defaults futuros de simulação. Criar Component instância uma definição e recebe IDs UUIDv7 no backend.

## Schema SQLite proposto

Manter `project.db`. Adicionar uma migration versionada para as tabelas abaixo.

1. `canvases`, com o Project, nome, viewport serializado, datas e soft delete.
2. `canvas_elements`, como entidade base com kind, type, geometria, parent e dados extensíveis.
3. `components`, vinculada um para um a `canvas_elements`, para os dados próprios de Component.
4. `component_ports`, vinculada a `components`.
5. `connections`, vinculada a Canvas, Components e Ports.

Esta adaptação evita tratar VisualElement e Component como o mesmo conceito, preserva um elemento visual livre e mantém dados extensíveis em JSON somente onde necessário. A migration também registra uma revision para cada command concluído. Exclusões usam `deleted_at` em vez de remoção física.

## Regras de persistência

1. Criar, remover, conectar, desconectar e editar propriedades executam command e transaction SQLite.
2. Durante drag, a posição muda apenas na store.
3. Em drag stop, uma única operação Move Component persiste a posição final e registra uma revision.
4. Ao abrir o Canvas, o repository carrega Canvas, elementos, Components, Ports, Connections e viewport em uma leitura consistente.
5. Ao reabrir o aplicativo, o estado salvo em `project.db` é restaurado.

## Validação de ports

A conexão valida direção e compatibilidade básica de protocolo. Uma conexão incomum mostra um aviso explicativo, mas continua disponível após confirmação. Nenhuma regra bloqueia silenciosamente o usuário.

## History preparada

Criar uma interface de command e registrar as operações Add Component, Remove Component, Move Component, Update Properties, Create Connection e Remove Connection. Undo e Redo não serão expostos nesta etapa se isso ampliar o risco, mas a modelagem deve permitir agrupamento de drag como uma única operação.

## Entrada no Canvas

Proposta: a rota muda para `#canvas/<projectId>`. A Home abre o Canvas de um Project ao selecionar seu card ou a ação Canvas associada ao Project mais recente. Se não existir Project, a Home mantém o fluxo atual de criação. A tela não inventa um Project fixo de demonstração.

## Entrega inicial

1. Substituir o grafo JSX fixo por React Flow com custom nodes Orbit e edges estilizadas.
2. Criar Components por toolbar ou biblioteca.
3. Mover, selecionar, deletar, conectar e editar propriedades.
4. Usar MiniMap e Controls reais, com aparência Orbit.
5. Persistir e restaurar um Canvas ligado a um Project local.
6. Adicionar testes unitários para a compatibilidade de ports e testes de integração Rust para migration e repository.
7. Manter a Home sem regressão e o visual do Canvas próximo da concept art.

## Fora desta entrega

Supabase sync, Clerk, colaboração, IA, marketplace, Simulation Engine, Presentation Flow, grupos, subcanvas e uma interface final de Undo e Redo.

## Critérios de aceite

1. O usuário abre o Canvas de um Project real.
2. Cria, move, seleciona, edita e remove Components.
3. Cria Connections entre ports tipados e recebe aviso nas conexões incomuns.
4. Pan, zoom, minimapa e controles representam nodes reais.
5. Fechar e reabrir restaura os dados locais.
6. `npm.cmd run build`, testes TypeScript e testes Rust relevantes passam.
7. Cada alteração de implementação recebe seu próprio arquivo em `docs/logs/`.

## Ajustes aprovados

Esta seção prevalece sobre qualquer trecho anterior deste documento.

1. `Port.direction` é somente `input`, `output` ou `bidirectional`.
2. `Port.protocol` é independente e começa com `http`, `sql`, `data` e `event`.
3. Zustand guarda apenas estado e adaptação da UI. Commands da camada Application ficam separados da Store e são os únicos responsáveis por operações persistidas.
4. O Component Registry é código TypeScript com templates oficiais de ports. Ao criar uma instância, o command copia os ports para `component_ports` e gera IDs UUIDv7 próprios. Os ports persistidos da instância são a fonte de verdade do Canvas aberto.
5. A migration de Canvas inclui foreign keys, índices por Canvas, Component, Connection e atualização, além de constraints para kind, direction, geometria não negativa e endpoints distintos.
6. Cada command persistido cria explicitamente uma row em `revisions` na mesma transaction SQLite.
7. Um Project sem Canvas recebe em seu primeiro acesso um Canvas vazio chamado `Untitled Canvas`, com viewport padrão e revision de criação, tudo na mesma transaction.
8. A implementação inclui teste de persistência round trip completo: criar Component, mover, conectar, editar propriedade, reabrir o banco e verificar Canvas, Ports, Connection, viewport e revision.
