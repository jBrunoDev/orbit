# AGENTS.md — Orbit

> Guia oficial de produto, arquitetura, UX e engenharia para agentes de IA e pessoas que trabalham no Orbit.
>
> Este arquivo é a fonte de verdade para decisões já aprovadas. Antes de implementar, refatorar ou sugerir uma mudança estrutural, preserve os princípios e limites definidos aqui.

---

## 1. Visão do produto

**Orbit** é um aplicativo desktop local-first para pensamento visual, arquitetura de sistemas, documentação, anotações, planejamento e simulação.

A proposta do produto pode ser resumida por:

> **Think · Draw · Run**

Ou, em português:

> **Pense · Planeje · Desenhe · Simule**

Orbit não deve ser tratado apenas como um editor de diagramas de system design. O Canvas também deve servir para raciocínio visual, planejamento, anotações, fluxos, mapas, esquemas, estudos e apresentações.

O fluxo principal do produto é:

```text
Notes → Canvas → Simulate → Docs
  ↑        ↓         ↓        ↑
  └──── Knowledge Graph ──────┘
```

Interpretação:

- **Notes**: pensar, registrar ideias, decisões, pesquisas e tarefas.
- **Canvas**: estruturar visualmente sistemas, fluxos, conceitos e relações.
- **Simulate**: testar cenários, comportamentos, falhas e capacidade.
- **Docs**: consolidar o conhecimento em documentação organizada e apresentável.
- **Knowledge Graph**: conectar todos os objetos importantes do projeto.

---

# 2. Princípios inegociáveis

Qualquer implementação deve respeitar estes princípios.

## 2.1 Local-first de verdade

Orbit deve funcionar **sem conta, sem internet e sem Supabase** para suas funções principais.

O usuário precisa conseguir:

- criar projetos;
- abrir projetos;
- editar Canvas;
- escrever Notes;
- criar Docs;
- usar Components;
- rodar simulações locais;
- pesquisar;
- exportar/importar projetos;
- acessar histórico local.

A nuvem é uma capacidade adicional, não uma dependência do produto.

### Regra

```text
Core Orbit !== Cloud dependency
```

Nunca transformar autenticação ou sincronização em requisito para abrir o aplicativo ou trabalhar localmente.

---

## 2.2 Conta é opcional

O usuário pode usar Orbit completamente sem login.

**Clerk** é utilizado quando o usuário deseja recursos de conta e sincronização em nuvem.

No primeiro login:

1. detectar projetos locais;
2. mostrar quais podem ser enviados para a nuvem;
3. permitir que o usuário escolha;
4. nunca fazer upload automático sem consentimento.

Cada projeto deve possuir explicitamente um modo de armazenamento:

```text
Local
Cloud
```

Um projeto Local pode ser convertido em Cloud posteriormente.

---

## 2.3 Dados do usuário são prioridade

Nenhuma alteração de arquitetura, sync, migration ou otimização pode introduzir risco desnecessário de perda silenciosa de dados.

Em caso de dúvida entre:

- descartar informação;
- preservar informação duplicada;

prefira **preservar**.

Conflitos devem ser explicitados, não escondidos.

---

## 2.4 Simulação útil, não decorativa

O modo Simulate não deve ser apenas uma animação bonita.

As animações do Canvas devem refletir eventos produzidos pelo motor da simulação.

Exemplo:

```text
Request #142
API received request
Cache MISS
Database query
Database response
```

Métricas e comportamento precisam ter relação com as configurações do cenário, conexões e Components.

---

## 2.5 O Canvas deve continuar livre

Orbit deve oferecer inteligência, validação e sugestões sem transformar o Canvas em um editor rígido.

Quando uma arquitetura for incomum ou potencialmente inválida:

- alertar;
- explicar;
- permitir continuar.

Evite bloquear ações desnecessariamente.

---

## 2.6 Ferramentas avançadas não podem prejudicar simplicidade

Uma Note simples deve continuar podendo ser apenas:

```text
Título
Texto
```

Um Canvas simples deve poder ser apenas:

```text
Texto + Shapes + Arrows
```

Properties, simulation metadata, Graph, templates avançados e outros recursos devem ser progressivos.

---

# 3. Escopo do aplicativo

As áreas principais do Orbit são:

```text
Home
Canvas
Notes
Templates
Components
Simulate
Docs
Profile
```

A sidebar principal deve refletir essa estrutura.

Também deve existir uma estrutura clara de:

```text
Workspace
└── Project
    ├── Canvas
    ├── Notes
    ├── Docs
    ├── Simulations
    ├── Components
    ├── Assets
    └── Settings
```

---

# 4. Stack principal

## Desktop

```text
Tauri
React
TypeScript
```

Tauri é o shell desktop oficial do Orbit.

Evite introduzir Electron sem decisão explícita de arquitetura.

---

## UI

```text
React
Tailwind CSS
```

Tailwind é a camada principal de estilização.

Evite:

- CSS global sem necessidade;
- valores mágicos duplicados;
- componentes visualmente inconsistentes;
- definir cores diretamente em dezenas de componentes.

Prefira tokens e componentes reutilizáveis.

---

## Canvas

```text
React Flow
```

React Flow é a base do graph editor.

Orbit pode construir abstrações acima dele, mas não deve vazar detalhes da biblioteca para o domínio do produto.

Exemplo:

```text
ReactFlow Node
      ↓
Orbit Canvas Element
      ↓
Visual Element | Component
```

---

## Persistência local

```text
SQLite
Filesystem
```

### SQLite

Responsável por:

- workspaces;
- projects;
- canvases;
- notes;
- docs;
- components;
- connections;
- tasks;
- graph relations;
- simulation configs;
- simulation runs;
- revisions;
- snapshots;
- sync state;
- metadata.

### Filesystem

Responsável por:

- imagens;
- anexos;
- assets;
- arquivos importados;
- previews;
- exports;
- dados binários.

Não armazenar grandes blobs no SQLite sem justificativa.

---

## Cloud

```text
Clerk
Supabase
```

### Clerk

Responsável por:

- identidade;
- login;
- sessão;
- provedores de autenticação.

### Supabase

Responsável por:

- PostgreSQL;
- Storage;
- Realtime;
- RLS;
- dados sincronizados do Orbit.

### Regra crítica

**Não adicionar Supabase Auth como segundo sistema de login.**

Clerk é o provedor de identidade do Orbit.

Supabase deve confiar na identidade proveniente do Clerk através da integração de autenticação de terceiros apropriada.

---

# 5. Arquitetura geral

```text
┌───────────────────────────────────────────────┐
│                 Orbit Desktop                 │
│                                               │
│  React + Tailwind + React Flow                │
│                                               │
│  UI / Features                                │
│          │                                    │
│          ▼                                    │
│  Application / Domain                         │
│          │                                    │
│          ├──────────────┐                     │
│          ▼              ▼                     │
│     SQLite          Filesystem                │
│          │              │                     │
│          └──────┬───────┘                     │
│                 ▼                             │
│             Sync Engine                       │
└─────────────────┬─────────────────────────────┘
                  │
           quando habilitado
                  │
          ┌───────▼─────────┐
          │     Supabase     │
          │ Postgres/Storage │
          │ Realtime/RLS     │
          └───────┬─────────┘
                  │
                  ▼
                Clerk
             Identity/Auth
```

O frontend não deve escrever diretamente na nuvem como primeira etapa de uma alteração.

O fluxo correto de edição é:

```text
User action
    ↓
Domain command
    ↓
SQLite
    ↓
Revision/Event
    ↓
Sync Outbox
    ↓
Supabase
```

---

# 6. Organização de domínio

Evite centralizar toda a lógica em componentes React.

Separe:

```text
UI
Application
Domain
Infrastructure
```

Exemplo conceitual:

```text
src/
├── app/
├── features/
│   ├── canvas/
│   ├── notes/
│   ├── docs/
│   ├── simulate/
│   ├── templates/
│   ├── components/
│   └── search/
├── domain/
│   ├── project/
│   ├── canvas/
│   ├── knowledge/
│   ├── simulation/
│   ├── revision/
│   └── sync/
├── infrastructure/
│   ├── sqlite/
│   ├── filesystem/
│   ├── supabase/
│   ├── clerk/
│   └── secure-storage/
└── shared/
```

Esta estrutura é uma direção arquitetural, não uma obrigação literal de nomes de pastas.

O objetivo é impedir que UI, storage, sync e regras de negócio fiquem acoplados.

---

# 7. Workspaces e Projects

O modelo principal é:

```text
Workspace
└── Project
```

Todo usuário começa conceitualmente com:

```text
Personal Workspace
```

O schema deve suportar múltiplos workspaces.

V1 não precisa implementar colaboração completa em equipes, porém o modelo deve estar preparado.

---

## Roles

Preparar autorização para:

```text
owner
admin
editor
viewer
```

No V1, a grande maioria dos projetos utilizará apenas `owner`.

Não simplificar o schema de maneira que torne impossível adicionar os demais papéis futuramente.

---

## Workspace membership

Modelo conceitual:

```text
Clerk User
   ↓
Workspace Membership
   ↓
Workspace
   ↓
Project
```

Projetos podem futuramente possuir regras adicionais ou overrides.

---

# 8. Profile

Orbit deve funcionar sem conta.

Portanto existem dois conceitos:

```text
Local Profile
Cloud Profile
```

Quando o usuário faz login:

```text
Local Profile
      ↓
Link / Merge
      ↓
Cloud Profile
```

Clerk guarda identidade.

Supabase guarda dados específicos do Orbit.

SQLite mantém representação local.

A interface pode exibir estados como:

```text
Local profile
Synced with account
```

---

# 9. Login desktop

A estratégia escolhida é autenticação via navegador externo.

Fluxo conceitual:

```text
Orbit
  ↓
Open default browser
  ↓
Clerk login
  ↓
Authentication complete
  ↓
orbit://auth/callback
  ↓
Orbit desktop
```

Antes da implementação, validar o mecanismo atual recomendado por Clerk e Tauri para:

- deep links;
- callback;
- sessão segura;
- PKCE/OAuth quando aplicável.

Não inventar um fluxo de autenticação próprio.

---

# 10. Modelo Local / Cloud

Cada Project deve possuir um storage mode.

Exemplo:

```ts
type ProjectStorageMode = "local" | "cloud";
```

### Local

Dados permanecem somente no dispositivo, salvo export/backups definidos pelo usuário.

### Cloud

Dados continuam sendo gravados localmente e são sincronizados com Supabase.

Cloud nunca deve significar:

```text
"não existe localmente"
```

O projeto deve permanecer usável offline.

---

# 11. Sync Engine

O sync deve ser uma camada explícita do sistema.

Não misturar sync diretamente com componentes React.

Arquitetura:

```text
Local Change
    ↓
SQLite Transaction
    ↓
Revision
    ↓
Outbox
    ↓
Sync Worker
    ↓
Supabase
```

Para mudanças remotas:

```text
Supabase
    ↓
Realtime notification
    ↓
Sync Worker
    ↓
Inbox
    ↓
Conflict/version analysis
    ↓
SQLite
    ↓
UI update
```

---

## 11.1 Outbox

A Outbox guarda operações locais ainda não confirmadas pela nuvem.

Campos conceituais:

```text
id
project_id
entity_id
entity_type
operation
revision
payload
created_at
synced_at
retry_count
last_error
```

---

## 11.2 Inbox

Mudanças remotas devem passar por uma Inbox ou mecanismo equivalente antes de serem aplicadas.

Isso permite:

- deduplicação;
- ordering;
- version checking;
- conflict handling;
- retries.

---

## 11.3 Realtime

No V1, Supabase Realtime deve atuar principalmente como:

> "Existe uma mudança na nuvem."

Não como o engine principal de edição colaborativa.

Fluxo:

```text
Realtime signal
      ↓
Sync Engine
      ↓
Fetch change
      ↓
Revision check
      ↓
SQLite
```

Isso preserva o princípio local-first.

---

# 12. IDs

Objetos importantes precisam receber IDs estáveis no momento em que são criados localmente.

Use uma estratégia adequada como:

```text
UUID
ou
ULID
```

Não dependa de IDs autoincrementais da nuvem para identidade global.

IDs persistentes são essenciais para:

- backlinks;
- sync;
- graph;
- embeds;
- references;
- offline creation;
- import/export;
- conflict resolution.

---

# 13. Versionamento e histórico

Orbit terá:

```text
Undo/Redo
Revision History
Checkpoints
Project Snapshots
```

Esses recursos devem compartilhar uma arquitetura coerente.

Evite construir quatro sistemas totalmente independentes.

---

## 13.1 Revisions

Canvas, Notes, Docs e Simulations devem possuir histórico.

O usuário pode:

- visualizar versões;
- comparar;
- restaurar.

---

## 13.2 Checkpoints

Usuário pode criar checkpoints nomeados:

```text
Initial Architecture
Before Redis Migration
MVP v1
```

---

## 13.3 Project Snapshots

Snapshot pode representar o estado coordenado de:

```text
Canvas
Notes
Docs
Simulations
Settings relevantes
```

Restaurar snapshot precisa preservar integridade entre referências.

---

## 13.4 Conflitos

Estratégia:

```text
versioning
+
safe merge
+
explicit conflict resolution
```

Quando merge automático não for seguro:

- preservar ambas as versões;
- informar ao usuário;
- permitir comparar;
- escolher Local;
- escolher Cloud;
- manter ambas.

Nunca sobrescrever silenciosamente um conflito relevante.

---

# 14. Exclusão e Trash

Deleção deve ser soft delete sempre que possível.

Exemplo:

```text
deleted_at
```

Objetos removidos podem ir para:

```text
Trash
```

A exclusão deve sincronizar sem destruir imediatamente os dados.

Definir política de retenção separadamente.

Snapshots e revisões precisam continuar coerentes com itens deletados.

---

# 15. Assets

Assets são local-first.

Estrutura conceitual:

```text
orbit.db

projects/
└── project-id/
    └── assets/
        ├── architecture.png
        ├── diagram.svg
        └── requirements.pdf
```

Para projetos Cloud:

```text
Filesystem
    ↓
Sync Engine
    ↓
Supabase Storage
```

A cópia local continua sendo parte importante do projeto.

---

# 16. Formato portátil `.orbit`

Orbit terá um formato de projeto portátil.

Extensão:

```text
.orbit
```

Estrutura conceitual:

```text
manifest.json
project.db
assets/
previews/
metadata/
```

Objetivos:

- exportar projeto completo;
- importar projeto completo;
- funcionar sem conta;
- transferir entre computadores;
- servir como unidade de backup;
- preservar IDs e relações.

O manifest deve possuir uma versão de schema.

Exemplo:

```json
{
  "format": "orbit-project",
  "schemaVersion": 1
}
```

A arquitetura deve permitir futuramente:

- encryption;
- password protection;
- signed packages.

Esses recursos avançados não são obrigatórios no V1.

---

# 17. Backups

Orbit deve suportar backups locais automáticos.

Projetos Cloud possuem também sync com Supabase, mas:

```text
sync != backup
```

O usuário pode configurar uma pasta externa.

Exemplos:

```text
OneDrive folder
Google Drive folder
Dropbox folder
External SSD
NAS folder
```

Orbit não precisa integrar diretamente com esses provedores para essa funcionalidade: salvar em uma pasta escolhida já permite que o software do provedor faça a sincronização.

Backups podem utilizar pacotes `.orbit`.

---

# 18. Segurança

## 18.1 Secrets

Nunca guardar em SQLite, texto puro ou projeto:

- API keys;
- access tokens;
- refresh tokens;
- passwords;
- credentials sensíveis.

V1 deve usar secure storage nativo do sistema operacional sempre que possível.

Exemplos:

```text
Windows Credential Manager
macOS Keychain
Linux secure secret storage apropriado
```

SQLite guarda apenas referência quando necessário.

---

## 18.2 Orbit Vault

Arquitetura pode ser preparada para um futuro:

```text
Orbit Vault
```

Que permitiria secrets compartilhados, portáveis ou relacionados ao projeto.

Não precisa ser implementado no primeiro MVP.

---

## 18.3 Encryption

V1:

- segurança padrão de transporte;
- storage seguro;
- proteção específica de credentials;
- RLS;
- boas práticas de sessão.

Arquitetura não deve impedir um futuro E2EE.

Full E2EE está fora do V1.

---

# 19. Supabase RLS

Cloud data deve ser protegida por Row Level Security.

Modelo conceitual:

```text
User
  ↓
Workspace Membership
  ↓
Workspace
  ↓
Project
  ↓
Resources
```

Recursos incluem:

```text
Canvas
Notes
Docs
Simulation Runs
Components
Assets metadata
```

Não confiar apenas em autorização do frontend.

---

# 20. Canvas

O Canvas é híbrido:

```text
Whiteboard
+
Structured Architecture Editor
```

Modos principais:

```text
Design
Present
Simulate
```

---

# 21. Canvas — Visual Elements

Visual Elements são livres e leves.

Tipos esperados:

```text
Text
Sticky Note
Shape
Image
Drawing
Free Arrow
```

Eles não precisam participar automaticamente da simulação.

---

# 22. Canvas — Components

Components são objetos inteligentes.

Exemplos:

```text
API Server
Database
Cache
Queue
Load Balancer
Service
Custom Component
```

Possuem:

- ID;
- tipo;
- properties;
- ports;
- metadata;
- simulation config;
- references;
- history.

---

# 23. Converter Shape em Component

Um elemento visual pode evoluir.

Exemplo:

```text
Shape
  ↓
Convert to Component
  ↓
Ports
Properties
Simulation behavior
Metadata
```

Isso permite desenhar primeiro e estruturar depois.

---

# 24. Typed Ports

Components possuem portas tipadas.

Exemplos:

```text
HTTP IN
HTTP OUT
EVENT IN
EVENT OUT
DATA IN
DATA OUT
SQL
QUEUE
WEBSOCKET
```

Uma conexão deve conhecer:

```text
source_component
source_port
target_component
target_port
connection_type
```

---

# 25. Smart Connections

Conexões também são objetos inteligentes.

Exemplo:

```text
API Server → Database

Protocol        PostgreSQL
Direction       Request
Latency         12ms
Payload         UserQuery
Throughput      450 req/s
Failure Rate    0.2%
```

Nem todos os campos precisam ser obrigatórios.

Presets podem preencher defaults.

---

# 26. Architecture Issues Engine

Orbit terá análise baseada inicialmente em regras determinísticas.

Exemplos:

```text
⚠ Incompatible ports
Database.SQL → API.HTTP
```

```text
⚠ Missing connection
Payment Service has no failure path
```

```text
ⓘ Potential bottleneck
Load Balancer → API Server
2,000 req/s → capacity 1,200 req/s
```

### Comportamento

Preferir:

```text
Warn + Explain + Allow
```

em vez de:

```text
Hard block
```

IA futura pode complementar o engine, mas V1 não deve depender dela.

---

# 27. Groups e Subcanvas

Orbit suporta:

```text
Visual Group
Collapsible Group
Subcanvas / Subflow
```

Um subsystem pode aparecer no Canvas pai como:

```text
┌──────────────┐
│   Backend    │
│  8 nodes  ↗  │
└──────────────┘
```

Ao abrir:

```text
Backend Subcanvas
├── API
├── Auth
├── Cache
└── Database
```

O grupo/subcanvas pode expor ports para o Canvas pai.

Isso permite arquiteturas grandes sem poluição visual.

---

# 28. Canvas History

Canvas terá:

```text
Persistent Undo/Redo
Visual Timeline
Named Checkpoints
```

Exemplo:

```text
00:42 Connected API → Database
00:41 Added Redis
00:39 Moved Backend
00:36 Changed API properties
```

Checkpoints aparecem na timeline.

---

# 29. Present Mode

Present Mode não é apenas fullscreen.

Orbit terá um **Presentation Flow**.

Exemplo:

```text
01 System Overview
02 Request enters API
03 Authentication
04 Backend Services
05 Database
06 Failure Scenario
```

Cada step pode guardar:

- câmera;
- zoom;
- posição;
- elementos destacados;
- elementos escondidos;
- annotations;
- transição;
- duração opcional.

Modos:

```text
Present
Auto Play
```

O Presentation Flow não deve modificar a estrutura original do Canvas.

---

# 30. Notes

Notes é um híbrido conceitual de:

```text
Notion
+
Obsidian
```

Objetivos:

- escrita rápida;
- pensamento;
- pesquisa;
- decisões;
- tarefas;
- conhecimento conectado.

---

# 31. Modelo de conteúdo das Notes

Internamente, Notes usam blocos estruturados.

Exemplos:

```text
Heading
Paragraph
Code Block
Quote
Callout
Checklist
Image
Canvas Embed
Component Reference
Simulation Result
```

Ao mesmo tempo, Notes devem continuar portáveis para Markdown.

Portabilidade não precisa significar que 100% das funcionalidades avançadas sejam representáveis em Markdown padrão sem extensão.

---

# 32. Links `[[...]]`

Orbit suporta referências internas.

Exemplos conceituais:

```text
[[Authentication Architecture]]
[[PostgreSQL]]
```

Referências podem apontar para:

```text
Note
Doc
Canvas
Component
Simulation Run
Task
```

Internamente, links devem utilizar IDs.

O nome é apenas apresentação.

Assim:

```text
Rename object
≠
Break references
```

---

# 33. Backlinks

Objetos relevantes podem mostrar onde são utilizados.

Exemplo:

```text
PostgreSQL

Referenced by

Notes
- Database Decisions

Docs
- System Architecture

Simulations
- Traffic Spike #42
```

---

# 34. Live Embeds e Snapshot Embeds

Objetos podem ser incorporados em Notes e Docs.

## Live Embed

Reflete o estado atual do objeto original.

## Snapshot Embed

Congela uma versão específica.

Objetos embeddable:

```text
Canvas
Canvas View
Component
Note
Doc
Simulation Run
```

---

# 35. Canvas Views

Um embed de Canvas pode guardar apenas uma View.

Exemplo:

```text
Canvas: Backend Architecture
View: Authentication Flow
```

View guarda:

- camera;
- zoom;
- posição;
- seleção/visibilidade;
- framing.

Não duplica necessariamente o Canvas inteiro.

---

# 36. Note Properties

Notes podem possuir properties opcionais.

Exemplos:

```text
Status
Type
Area
Priority
Tags
Created
Owner
Reviewed
Due date
```

Usuário pode criar properties customizadas.

Uma Note simples não deve exigir nenhum desses campos.

---

# 37. Note Views

Notes podem ser vistas por filtros e configurações.

Exemplo:

```text
All Notes
Decisions
Ideas
To Review
```

Properties devem ser utilizáveis em:

- filters;
- search;
- templates;
- future automation;
- future AI.

---

# 38. Tasks

Tasks são objetos estruturados, mesmo quando surgem de um checklist.

Exemplo:

```text
☐ Optimize database queries

Due       Sep 25
Priority  High

Linked to
- PostgreSQL
- Traffic Spike #42
- Performance Doc
```

Uma Task pode se relacionar com:

```text
Note
Doc
Canvas
Component
Simulation Run
```

Concluir uma Task não deve alterar automaticamente a arquitetura.

---

# 39. Knowledge Graph

Orbit possui um grafo interno real, não apenas visual.

Relações podem possuir tipo.

Exemplos:

```text
references
contains
depends_on
tested_by
documents
task_for
```

Evite reduzir relações a simples strings sem semântica quando houver necessidade estrutural.

---

# 40. Graph View

Existem dois modos.

## Project Graph

Mostra o grafo de conhecimento do projeto.

## Local Graph

Mostra apenas relações próximas ao objeto atual.

Filtros:

```text
Notes
Docs
Canvas
Components
Tasks
Simulations
```

Também pode filtrar por:

- tag;
- type;
- relation;
- property;
- scope.

Clicar em node navega até o objeto.

Se for um Component dentro do Canvas:

```text
Open Canvas
→ Focus Component
```

---

# 41. Docs

Docs não são apenas Notes em outra pasta.

Separação conceitual:

```text
Notes = thinking/workspace
Docs  = structured documentation
```

Fluxo:

```text
Think in Notes
Model in Canvas
Test in Simulate
Consolidate in Docs
```

---

# 42. Note → Doc

Uma Note pode ser promovida/convertida em Doc.

A operação deve preservar quando possível:

- references;
- backlinks;
- history;
- IDs ou relationship mapping.

Evitar copy/paste destrutivo.

---

# 43. Reusable Documentation

Docs podem incorporar conteúdo existente.

Exemplo:

```text
# Authentication
Live Reference → Authentication Decision Note

# Architecture
Canvas View → Backend Overview

# Performance
Simulation Run → Traffic Spike #42
```

Usuário escolhe entre:

```text
Live Reference
Snapshot
```

---

# 44. Documentation Spaces

Um Project pode ter múltiplos espaços.

Exemplo:

```text
Docs
├── Technical Documentation
│   ├── Overview
│   ├── Architecture
│   ├── API
│   └── Deployment
├── Product
│   ├── Requirements
│   ├── User Flows
│   └── Decisions
└── Onboarding
    ├── Getting Started
    └── Development Setup
```

Cada Space possui:

- tree;
- ordering;
- homepage;
- navigation;
- metadata.

Orbit pode gerar:

- sidebar;
- breadcrumbs;
- Table of Contents.

---

# 45. Components Library

Components deve ter:

```text
Official Components
Custom Components
Reusable Presets
Reusable Groups/Stacks
```

Exemplos oficiais:

```text
API Server
Database
Cache
Queue
Load Balancer
Client
Worker
Storage
```

---

# 46. Custom Components

Custom Component deve ser definido por um schema estruturado.

Possíveis campos:

```text
name
category
icon
properties schema
ports
default values
simulation behavior
visual settings
metadata
```

Usuário também pode transformar um grupo em Component reutilizável.

---

# 47. Templates

Templates podem representar:

```text
Full Project
Canvas
Notes structure
Docs structure
Simulation scenario
Component stack
```

Um template pode conter variáveis.

Exemplo:

```text
Project Name
Database Type
Environment
Region
```

Na criação, Orbit resolve essas variáveis.

---

## Marketplace

Arquitetura deve estar preparada para futuro marketplace/community templates.

Marketplace público não pertence ao V1.

Não adicionar upload público, ratings, purchases ou social features sem nova decisão.

---

# 48. Simulate

Simulate usa o mesmo modelo de Components e Connections do Canvas.

O motor recebe:

```text
Scenario
Connections
Components
Seed
```

e produz:

```text
Events
Metrics
Issues
Run Result
```

---

# 49. Simulation configuration hierarchy

Existem três camadas.

```text
Scenario
   ↓
Connections
   ↓
Components
```

---

## Component configuration

Exemplo:

```text
API Server

Latency       20–80 ms
Capacity      1,000 req/s
Failure Rate  0.1%
Instances     3
```

---

## Connection configuration

Exemplo:

```text
API → Database

Protocol       PostgreSQL
Latency        8–15 ms
Bandwidth      1 Gbps
Timeout        3s
Retry          2x
```

---

## Scenario configuration

Exemplo:

```text
Traffic Spike

Duration        5 min
Initial users   1,000
Peak users      25,000
Ramp-up         60s
Requests/user   4/min
```

Fault Injection:

```text
Cache fails at 01:30
DB latency +300% at 02:00
API instance #2 fails at 03:10
```

---

# 50. Simulation Events

Uma animação deve corresponder a eventos.

Exemplo:

```text
00:04.231 API received request
00:04.247 Cache MISS
00:04.251 DB query started
00:04.296 DB response
```

Eventos devem ser registráveis e inspecionáveis.

---

# 51. Observability

Simulation UI pode exibir:

```text
Requests
Success Rate
Error Rate
Average Latency
P95
P99
Throughput
Service Health
Resource Load
```

Não inventar métricas sem base no modelo do simulation engine.

---

# 52. Simulation Scenarios

Exemplos:

```text
Normal Traffic
Traffic Spike
Database Slow
Cache Down
Payment Failure
```

Templates podem fornecer cenários prontos.

---

# 53. Visual Run

Visual Run exibe:

- animações;
- event flow;
- timeline;
- metrics;
- current service state.

Controles:

```text
Reset
Run
Pause
Step
0.5x
1x
2x
5x
10x
```

Step avança evento por evento.

---

# 54. Fast Run

Fast Run:

- não precisa renderizar cada evento;
- calcula o resultado rapidamente;
- prioriza análise.

Uso esperado:

```text
parameter tuning
comparison
batch runs
experimentation
```

---

# 55. Seed

Simulation Run possui seed.

Mesma configuração + mesma seed deve produzir resultado reproduzível, dentro das garantias do engine.

Exemplo:

```text
Seed: 847291
```

Isso é importante para:

- debugging;
- comparação;
- apresentações;
- tests.

---

# 56. Batch / Monte Carlo

Orbit deve estar preparado para executar várias seeds.

Exemplo:

```text
Runs: 100

Average latency
P95
P99
Error rate
Distribution
```

Batch Runs são uma funcionalidade avançada e podem ficar fora do MVP inicial.

A arquitetura do simulation engine não deve impedir sua implementação futura.

---

# 57. Simulation Run

Uma execução pode ser salva como entidade.

Exemplo:

```text
Traffic Spike #42
Sep 21
5 min
Completed
```

Run contém:

- scenario;
- seed;
- config snapshot;
- events;
- metrics;
- timestamps;
- result;
- references.

---

# 58. Simulation comparison

Usuário pode comparar runs.

Exemplo:

```text
Baseline          Redis Added

Latency  241ms    118ms
Errors     2.4%    0.8%
DB Load      82%     41%
```

Evite emitir automaticamente julgamentos absolutos como "arquitetura perfeita".

Fornecer dados e diferenças.

---

# 59. Simulation Run references

Simulation Run pode ser referenciado em:

```text
Notes
Docs
Tasks
Graph
```

Pode ser Live Reference ou Snapshot quando aplicável.

---

# 60. Search

Orbit possui busca universal offline.

Atalho:

```text
Ctrl + K
Cmd + K
```

Command Palette deve pesquisar:

- Projects;
- Canvas;
- Canvas text;
- Components;
- Notes;
- Docs;
- Tasks;
- Simulation Runs.

Também executa ações.

Exemplos:

```text
New Canvas
New Note
Open Project
Run Simulation
Create Snapshot
```

---

# 61. Search indexing

Busca tradicional precisa funcionar offline.

Arquitetura deve estar preparada para busca semântica futura.

Exemplos futuros:

```text
"Where did I write about Redis?"
"Which diagrams use PostgreSQL?"
```

Não tornar busca básica dependente de embeddings ou IA.

---

# 62. AI

AI está preparada na arquitetura, mas não é dependência do V1.

Possibilidades futuras:

```text
Explain Diagram
Generate Diagram
Find Bottlenecks
Generate Docs
Improve Notes
Text to Canvas
Semantic Search
Task Suggestions
```

Regras:

- IA não deve sobrescrever conteúdo silenciosamente;
- mudanças devem ser previewable/reviewable;
- IA deve respeitar local/cloud/privacy settings;
- IA não substitui regras determinísticas quando uma regra simples resolve.

---

# 63. Collaboration

V1 é essencialmente single-user.

Schema deve estar preparado para:

```text
members
roles
permissions
realtime collaboration
```

Mas não adicionar CRDT, presence multi-user complexa ou cursores colaborativos apenas "porque pode ser útil".

Esses itens pertencem a uma fase futura.

---

# 64. Design System

A identidade visual aprovada deve ser transformada em tokens.

Direção:

- dark;
- premium;
- técnico;
- minimalista;
- near-black/deep navy;
- accent purple/blue-purple;
- borders finos;
- sombras discretas;
- glass apenas quando contribuir;
- bastante espaço;
- alta legibilidade.

---

## 64.1 Tokens

Centralizar:

```text
colors
spacing
radius
typography
shadows
z-index
motion
states
```

Exemplo conceitual:

```ts
orbit.colors.background
orbit.colors.surface
orbit.colors.border
orbit.colors.textPrimary
orbit.colors.textMuted
orbit.colors.accent
orbit.colors.success
orbit.colors.warning
orbit.colors.danger
```

Não espalhar `#hex` por componentes.

---

## 64.2 Semantic colors

Cores semânticas podem incluir:

```text
green
blue
yellow
red
```

Usar para:

- status;
- simulation;
- issues;
- success;
- warning;
- errors.

A identidade principal continua roxo/azul-roxo.

---

## 64.3 Typography

Sans-serif moderna e legível.

Handwritten style pode aparecer em:

- sticky notes;
- annotations;
- canvas notes.

Nunca usar handwritten font para UI principal, settings ou leitura longa.

---

# 65. Layout

Direção desktop macOS-like, sem copiar literalmente componentes do macOS.

Padrões:

```text
Left Sidebar
Main Content
Optional Inspector / Right Panel
Top contextual toolbar
```

Canvas pode possuir:

- toolbar;
- inspector;
- minimap;
- simulation controls.

---

# 66. Accessibility

Não depender apenas de cor para comunicar estado.

Garantir:

- contraste adequado;
- focus visible;
- navegação por teclado;
- labels;
- tooltips quando necessário;
- hit areas razoáveis.

Atalhos devem ter equivalente visual.

---

# 67. Motion

Motion deve ajudar compreensão.

Usos válidos:

- canvas transitions;
- Present camera;
- simulation events;
- collapse/expand;
- state changes.

Evitar animação decorativa excessiva.

Respeitar configurações de reduced motion quando possível.

---

# 68. Performance

Orbit precisa continuar responsivo em projetos grandes.

Cuidados:

- virtualização onde fizer sentido;
- não renderizar nodes invisíveis desnecessariamente;
- evitar salvar SQLite a cada pixel de drag sem debounce/coalescing;
- agrupar history operations;
- lazy-load assets;
- download on demand;
- background sync;
- Fast Run sem renderização de eventos.

---

# 69. Novo dispositivo

Após login em outro dispositivo:

1. sincronizar metadata dos projetos Cloud;
2. exibir lista;
3. baixar conteúdo completo quando projeto for aberto.

Usuário pode marcar projetos:

```text
Available Offline
```

Nesse caso, baixar antecipadamente dados/assets necessários.

Não baixar todos os assets de todos os projetos automaticamente.

---

# 70. Database schema philosophy

Use modelo relacional para entidades estáveis.

Exemplos:

```text
workspaces
workspace_members
projects
canvases
canvas_elements
components
connections
notes
docs
tasks
simulation_runs
revisions
snapshots
relations
```

JSON é adequado para campos realmente extensíveis, por exemplo:

```text
custom properties
visual config
component-specific simulation config
```

Não transformar o banco inteiro em blobs JSON sem necessidade.

---

# 71. Migrations

Toda alteração estrutural de banco deve possuir migration.

Isso se aplica a:

```text
SQLite
Supabase/PostgreSQL
```

Não fazer mudanças manuais de produção sem refletir no histórico de migrations.

Migrations devem ser:

- versionadas;
- revisáveis;
- reproduzíveis.

---

# 72. `.orbit` migrations

O formato portátil também deve possuir schema version.

Ao importar projeto antigo:

```text
Read schemaVersion
→ migrate
→ validate
→ open
```

Nunca presumir que todo `.orbit` usa schema atual.

---

# 73. Testes

Prioridades de teste:

## Unit

- domain rules;
- typed ports;
- architecture issue rules;
- simulation calculations;
- version comparison;
- conflict rules.

## Integration

- SQLite repositories;
- migrations;
- filesystem;
- sync;
- outbox/inbox;
- Supabase integration.

## E2E

Fluxos críticos:

```text
Create local project
Edit offline
Close Orbit
Reopen
Data still exists

Login
Convert Local → Cloud
Sync
Disconnect internet
Edit
Reconnect
Sync

Conflict
Preserve both versions

Export .orbit
Delete local copy
Import .orbit
Project preserved

Restore snapshot
References remain valid
```

---

# 74. Regra máxima para testes de persistência

A prioridade é:

> **Não perder dados silenciosamente.**

Qualquer mudança em:

- sync;
- migrations;
- revision;
- import/export;
- filesystem;
- conflict resolver;

deve receber testes específicos.

---

# 75. Updates

Usar o mecanismo apropriado e seguro de atualização do ecossistema Tauri.

Updates devem:

- validar artefato quando aplicável;
- não apagar dados locais;
- executar migrations de maneira segura;
- permitir recuperação em caso de falha.

Não misturar app update com project data migration sem controle de versão.

---

# 76. Telemetria e crash reporting

Se introduzidos:

- devem ser transparentes;
- respeitar privacy settings;
- coletar apenas o necessário;
- nunca capturar conteúdo privado de Notes/Docs/Canvas por padrão;
- nunca capturar secrets.

Não adicionar analytics invasivo.

---

# 77. Templates e defaults para iniciantes

Recursos avançados não devem exigir configuração completa.

Exemplo:

```text
Add API Server
```

pode criar defaults úteis.

Simulation template:

```text
Normal Traffic
```

pode preencher:

- latency;
- capacity;
- retry;
- timeout;
- traffic.

Usuário avançado pode editar depois.

---

# 78. Arquitetura extensível

Preparar extensões futuras sem implementá-las prematuramente.

Future-ready:

```text
AI
Marketplace
Team Workspaces
Realtime Collaboration
E2EE
Semantic Search
Orbit Vault
Real infrastructure execution
Community Components
```

Future-ready significa:

> Não criar uma decisão hoje que torne o recurso impossível amanhã.

Não significa:

> Implementar toda a complexidade agora.

---

# 79. V1 — escopo principal

V1 deve priorizar:

```text
Desktop shell
Projects / Workspaces básicos
Local-first SQLite
Filesystem assets
Canvas
Notes
Docs
Components
Templates
Simulate
Simulation Runs
Search / Command Palette
History / revisions
Snapshots
.orbit import/export
Optional Clerk login
Optional Cloud sync
Supabase
Basic RLS
Backups
Secure credential storage
```

---

# 80. Fora do V1

Não considerar requisito obrigatório inicial:

```text
Full realtime collaboration
Marketplace público
Full E2EE
Advanced AI
Full Orbit Vault
Real infrastructure execution
Public social profiles
Community feed
Payments
Team billing
CRDT editing
Advanced Monte Carlo UI
```

Alguns podem ter arquitetura preparada.

---

# 81. Regras para agentes de IA

Ao trabalhar neste repositório:

## Faça

- leia este arquivo antes de mudanças estruturais;
- preserve local-first;
- reutilize entidades e services existentes;
- mantenha regras de domínio fora da UI;
- escreva migrations;
- adicione testes;
- preserve backward compatibility quando razoável;
- documente novas decisões;
- trate conflitos explicitamente;
- use IDs persistentes;
- mantenha account/cloud opcionais;
- mantenha secrets fora do banco comum.

## Não faça

- não torne login obrigatório;
- não torne internet obrigatória;
- não substitua SQLite por chamadas diretas à nuvem;
- não use Supabase Auth;
- não faça upload automático de projetos Local;
- não armazene secrets em plaintext;
- não apague conflito silenciosamente;
- não crie features future-scope sem aprovação;
- não introduza dependência grande apenas por conveniência;
- não misture regras de simulation com UI animation;
- não duplique dados quando Live Reference resolve;
- não quebre IDs ao renomear objetos;
- não altere design language sem justificativa.

---

# 82. Regra para novas dependências

Antes de adicionar package:

1. confirmar necessidade real;
2. verificar se stack atual resolve;
3. avaliar tamanho/complexidade;
4. verificar manutenção;
5. avaliar impacto no Tauri;
6. evitar duplicação de responsabilidades.

Toda dependência core deve ter justificativa clara.

---

# 83. Regra para mudanças de produto

Quando uma solicitação conflitar com este documento:

1. não esconder o conflito;
2. explicar qual decisão existente seria afetada;
3. propor alternativa compatível;
4. se a mudança for aprovada, atualizar este arquivo.

`AGENTS.md` deve evoluir junto com decisões oficiais.

---

# 84. Critérios de aceite — Local-first

Uma implementação local-first só está completa se:

- app abre offline;
- projeto local abre offline;
- edição funciona offline;
- fechamento/reabertura preserva dados;
- operações pendentes de sync sobrevivem restart;
- falta de nuvem não quebra UI;
- login não é necessário para core features.

---

# 85. Critérios de aceite — Cloud sync

Sync básico está correto quando:

1. alteração local salva em SQLite;
2. alteração entra em fila;
3. offline não perde operação;
4. online envia operação;
5. confirmação remove/fecha pendência;
6. mudança remota chega através do Sync Engine;
7. revision é verificada;
8. conflito não é sobrescrito silenciosamente.

---

# 86. Critérios de aceite — Canvas

Canvas deve suportar:

- Visual Elements;
- Components;
- typed ports;
- smart connections;
- warnings;
- groups;
- collapse;
- subcanvas;
- undo/redo;
- history;
- checkpoints;
- Present;
- Simulate.

Não é necessário que todas as features cheguem no primeiro build, mas o modelo de domínio deve ser compatível.

---

# 87. Critérios de aceite — Notes

Notes deve suportar progressivamente:

- rich blocks;
- Markdown portability;
- references;
- backlinks;
- embeds;
- properties;
- views;
- tasks;
- graph relationships.

Notas simples precisam continuar simples.

---

# 88. Critérios de aceite — Docs

Docs devem:

- possuir estrutura própria;
- suportar spaces;
- permitir hierarquia;
- ter navegação;
- reutilizar Notes/Canvas/Simulation;
- permitir Live e Snapshot references.

Docs não devem ser uma cópia desconectada de Notes.

---

# 89. Critérios de aceite — Simulation

Simulation Engine deve:

- produzir eventos;
- respeitar config;
- suportar seed;
- gerar métricas;
- suportar Visual Run;
- suportar Fast Run;
- persistir Runs;
- permitir comparação;
- aceitar fault injection.

UI animation é consequência do engine.

---

# 90. Design acceptance

Antes de considerar uma tela final:

- está coerente com dark premium Orbit?
- usa tokens?
- evita excesso de bordas?
- tem hierarquia clara?
- mantém boa legibilidade?
- possui estados hover/focus/disabled/loading?
- funciona em janela menor?
- não depende apenas de cor?
- mantém linguagem consistente com outras telas?

---

# 91. Performance acceptance

Em features que trabalham com muitos objetos:

- evitar loops de render desnecessários;
- medir performance antes de grandes otimizações;
- evitar query N+1;
- carregar assets sob demanda;
- usar transactions SQLite;
- realizar operações pesadas fora do frame crítico de UI quando possível.

---

# 92. Terminologia oficial

Preferir:

```text
Workspace
Project
Canvas
Visual Element
Component
Connection
Port
Group
Subcanvas
Note
Doc
Documentation Space
Task
Template
Scenario
Simulation Run
Revision
Checkpoint
Snapshot
Live Reference
Snapshot Reference
Project Graph
Local Graph
Local Project
Cloud Project
Sync Engine
```

Evite criar sinônimos diferentes para a mesma entidade sem necessidade.

---

# 93. Relações principais do domínio

Visão conceitual:

```text
Workspace
└── Project
    ├── Canvas
    │   ├── Visual Elements
    │   ├── Components
    │   ├── Connections
    │   ├── Groups
    │   └── Subcanvas
    │
    ├── Notes
    │   ├── Blocks
    │   ├── Tasks
    │   └── Properties
    │
    ├── Docs
    │   └── Documentation Spaces
    │
    ├── Simulations
    │   ├── Scenarios
    │   └── Simulation Runs
    │
    ├── Templates
    ├── Assets
    ├── Revisions
    ├── Snapshots
    └── Knowledge Graph
```

---

# 94. Modelo mental final

O Orbit não é apenas um editor.

Ele é um ambiente de pensamento conectado.

```text
IDEAS
  ↓
Notes
  ↓
Canvas
  ↓
Simulation
  ↓
Documentation
```

Enquanto isso:

```text
Components
References
Tasks
History
Search
Graph
```

conectam todas as etapas.

---

# 95. Prioridade de implementação sugerida

Ordem recomendada:

```text
1. App shell + Design System
2. SQLite + Project model
3. Workspace / Home
4. Canvas básico
5. Components + Connections
6. Notes
7. References / Knowledge Graph
8. Docs
9. Simulation Engine básico
10. Simulation UI
11. Revisions / Snapshots
12. .orbit import/export
13. Search / Command Palette
14. Clerk
15. Supabase sync
16. Advanced simulation
17. Presentation Flow
18. Advanced Graph / Templates
```

A sequência pode ser ajustada, mas evite implementar cloud antes de existir persistência local sólida.

---

# 96. Definition of Done para features

Uma feature relevante não está pronta apenas porque funciona visualmente.

Verificar:

```text
[ ] Domain model definido
[ ] Persistência quando aplicável
[ ] Offline behavior considerado
[ ] Sync behavior considerado
[ ] History/versioning considerado
[ ] Error state
[ ] Loading state
[ ] Empty state
[ ] Keyboard/focus
[ ] Design tokens
[ ] Tests adequados
[ ] Migrations se necessário
[ ] No secret leakage
[ ] Documentation atualizada
```

Nem todos os itens se aplicam a toda feature.

---

# 97. Regra de simplicidade

Não criar arquitetura complexa para um problema que ainda não existe.

Mas também não violar deliberadamente decisões fundamentais para ganhar velocidade de curto prazo.

Boa solução:

```text
simple now
+
clean boundary
+
future-compatible
```

Má solução:

```text
temporary shortcut
+
business logic in UI
+
cloud coupling
+
rewrite later
```

---

# 98. Decisões resumidas

Decisões oficiais consolidadas:

1. Tauri + React.
2. Tailwind CSS.
3. React Flow.
4. Notes híbrido Notion + Obsidian.
5. Simulation Engine evolutivo.
6. Components oficiais + custom + presets.
7. Workspace → Project → resources.
8. Local-first completo.
9. Single-user V1, colaboração futura.
10. AI future-ready.
11. Design System oficial Orbit.
12. Templates modulares e full project.
13. Living Documentation.
14. Clerk + Supabase.
15. SQLite + filesystem.
16. Bidirectional sync via Sync Engine.
17. Versioning + merge + conflict UI.
18. Revisions + checkpoints + snapshots.
19. `.orbit` portable projects.
20. Secure OS storage para secrets.
21. Typed ports + smart connections.
22. Knowledge Graph e referências persistentes.
23. Simulation Runs + análise comparativa.

---

# 99. Fonte de verdade

Quando existir conflito entre:

```text
mockup antigo
comentário antigo
experimento
implementação temporária
AGENTS.md
```

a decisão mais recente oficialmente aprovada deve prevalecer.

Se `AGENTS.md` estiver desatualizado, atualizar este arquivo na mesma mudança em que a nova decisão for implementada.

## Registro obrigatório de mudanças

Toda alteração concluída na codebase do Orbit deve criar um novo arquivo em `docs/logs/`, inclusive mudanças de frontend, backend, banco de dados, assets, testes, configuração, documentação, correções e refatorações.

O registro deve informar:

- objetivo e escopo da mudança;
- comportamento entregue ou corrigido;
- arquivos relevantes alterados;
- validações executadas e o que ainda exige teste manual.
- documentação ou especificações que ficaram desatualizadas pela mudança, com a atualização correspondente quando ela estiver no escopo.

Não reutilizar nem sobrescrever um log de outra mudança. O log deve ser criado na mesma entrega da alteração, antes de ela ser considerada concluída. Se uma validação pendente for concluída posteriormente, ou se uma decisão tornar um documento anterior obsoleto, atualizar o registro ou a documentação correspondente para manter o histórico consistente.

---

# 100. Princípio final

Toda decisão deve fortalecer pelo menos um destes pontos:

```text
Clareza
Portabilidade
Confiabilidade
Controle do usuário
Offline-first
Conhecimento conectado
Evolução futura
```

Se uma feature exige sacrificar vários deles, reavalie antes de implementá-la.

**Orbit deve continuar pertencendo ao usuário: seus projetos devem funcionar localmente, ser portáveis, compreensíveis, recuperáveis e evolutivos.**
