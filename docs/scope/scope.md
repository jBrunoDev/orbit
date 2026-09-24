# Scope: Orbit

Orbit é um aplicativo desktop local para pensamento visual, documentação e arquitetura de sistemas. Este escopo acompanha a entrega de Docs e da arquitetura assistida por IA sobre a base local já existente.

**Build approach:** Tracer Bullet (cada marco entrega um caminho funcional de interface, estado e persistência).
**Workflow:** Beta (`/check verify`, depois `/test`). O padrão é adequado para dados locais, exportação e credenciais pessoais.

_Estas etapas são recomendações para manter o trabalho organizado. Você pode ajustar a ordem ou pular uma etapa se o contexto do projeto mudar._

## At a glance

| # | Feature | Phase | Status |
|---|---------|-------|--------|
| 1 | Base local do Orbit | Existing | existing |
| 2 | Docs local | Slice 1 | in-progress |
| 3 | Vínculos entre Docs e Canvas | Slice 2 | planned |
| 4 | Fundação de IA e configurações | Slice 3 | in-progress |
| 5 | Ingestão de fontes e modelo de arquitetura | Slice 4 | in-progress |
| 6 | Geração revisável de Docs e Canvas | Slice 5 | in-progress |

## Existing

### 1. Base local do Orbit · existing
Projects locais, Canvas persistido, Notes, Templates, Components e superfícies de Simulate já formam a base sobre a qual Docs será construída. code in `src/` e `src-tauri/`

## Slice 1: Docs local

### 2. Docs local · in-progress
Criar a área Docs fiel à referência, com árvore de páginas e pastas, Markdown, preview seguro, autosave e exportação local. A spec já decidiu o modelo, as rotas e a persistência.
**Done when:** o usuário cria, organiza, edita, remove, restaura e exporta documentos Markdown locais, com autosave confiável e índice de seções.
- [x] Design it (spec): [0004 Docs e arquitetura assistida por IA](../specs/0004-docs-ai-architecture.md)
- [x] Build it: `/develop docs local` · code in `src/features/docs/` e `src-tauri/src/lib.rs`
  - [x] Migration, comandos Tauri, gateway e store (AC 3, AC 5, AC 7) · code in `src-tauri/src/lib.rs` e `src/features/docs/`
  - [x] Explorer, editor, preview, outline e autosave (AC 1 a AC 5) · code in `src/features/docs/components/DocsPage.tsx`
  - [x] Exportação e testes de persistência (AC 6 e AC 7) · code in `src-tauri/src/lib.rs`
- [ ] Verify it: `/check verify docs local`
- [ ] Test it: `/test docs local`

## Slice 2: Vínculos entre Docs e Canvas

### 3. Vínculos entre Docs e Canvas
Conectar documentos e Canvases por relações persistidas, sem duplicar o conteúdo ou tornar qualquer um deles imutável.
**Done when:** o usuário cria um vínculo, abre o Canvas pelo documento, encontra o documento pelo Canvas e preserva a relação após reabrir o Project.
- [ ] Build it: `/develop docs canvas links`

## Slice 3: Fundação de IA e configurações

### 4. Fundação de IA e configurações
Preparar flags, Settings, cofre de credenciais e contratos de provider, sem enviar conteúdo nem fingir análise enquanto o provider não estiver implementado.
**Done when:** a interface apresenta estados honestos de flag, provider e chave, e os segredos pessoais ficam fora de SQLite, arquivos de Project e frontend.
- [ ] Build it: `/develop ai settings stubs`

## Slice 4: Ingestão de fontes e modelo de arquitetura

### 5. Ingestão de fontes e modelo de arquitetura
Adicionar entrada por repositório GitHub, documento ou ambos, com consentimento, filtros, limites, evidências e `ArchitectureModel` validado.
**Done when:** uma fonte permitida produz um rascunho local com nós, arestas, evidências, confiança e hipóteses, sem criar entidades reais do Project.
- [ ] Build it: `/develop architecture ingestion`

## Slice 5: Geração revisável de Docs e Canvas

### 6. Geração revisável de Docs e Canvas
Conectar o provider OpenAI ao modelo intermediário, derivar documento e Canvas editável, e permitir aceitar os artefatos separadamente em transações seguras.
**Done when:** o usuário revisa, edita e aceita só o documento, só o Canvas ou ambos, com layout legível, evidências e revisions preservadas.
- [ ] Build it: `/develop ai docs canvas generation`

## Deferred

1. Editor de blocos rico. Markdown continua como formato canônico até haver uma necessidade comprovada.
2. Exportação de pacote com assets.
3. OAuth GitHub, mantendo o token pessoal como opção.
4. Comments, histórico visual completo e colaboração em Docs.
5. IA habilitada por padrão antes de avaliações com repositórios de arquitetura conhecida.

## Legend

`existing` registra uma base anterior a este fluxo. `in-progress` já tem uma spec aceita e está pronto para construção. `planned` representa o próximo marco ainda não iniciado.

Cada marco deve preservar a regra local first do Orbit. O próximo passo é o primeiro checkbox não marcado: `/develop docs local`.
