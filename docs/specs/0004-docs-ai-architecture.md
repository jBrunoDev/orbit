# Docs e arquitetura assistida por IA

**Status**: Accepted

## Summary

Esta especificação cria a área Docs do Orbit como um editor local de documentação de Project, fiel a `Design/orbitDocsPage.png`, e prepara a integração futura de IA sem tornar a IA necessária para criar, editar, exportar ou abrir documentos.

Docs usa Markdown como formato canônico. A IA recebe uma fonte de repositório, documento, ou ambas, cria primeiro um modelo de arquitetura estruturado e verificável, e usa esse mesmo modelo para produzir um rascunho de documentação e um Canvas editável. O usuário pode aprovar somente o documento, somente o Canvas, ou ambos.

## Context

O Orbit já tem React, TypeScript, Tauri, SQLite por Project, Zustand, React Flow, `AppShell`, `AppSidebar`, Notes e Canvas. A migration de Templates já criou uma tabela `docs`, e Templates já podem semear conteúdo nela, mas não existe rota, comando Tauri, estado de interface ou página Docs.

O produto exige uso local sem conta e sem rede para o fluxo principal. Logo, criar, editar, pesquisar, salvar automaticamente e exportar Docs precisa funcionar sem IA. A análise de código é uma capacidade opcional, com consentimento explícito antes de qualquer conteúdo local ser enviado ao provedor.

## Requirements

1. **AC 1** A rota `#docs/:projectId/:documentId?` abre uma área Docs que reutiliza `AppShell` e `AppSidebar`, sem alterar o contrato visual global.
2. **AC 2** A página reproduz a composição da referência: explorer de Docs à esquerda, editor e preview no centro e índice de headings à direita.
3. **AC 3** O usuário cria, edita, renomeia, organiza, remove e restaura Docs locais. A remoção é soft delete.
4. **AC 4** O conteúdo persistido é Markdown. O usuário alterna entre edição, preview e modo dividido. HTML arbitrário não é executado no preview.
5. **AC 5** Cada alteração relevante faz autosave local serializado, mostra estado de salvamento e não perde a última edição em blur, troca de documento ou atalho de salvar.
6. **AC 6** O usuário exporta um documento em Markdown UTF 8 para um destino escolhido explicitamente.
7. **AC 7** A estrutura de dados preserva documentos já criados por Templates e permite espaços, pastas, ordenação, relações e revisions sem reescrever conteúdo do usuário.
8. **AC 8** Um documento pode ter relação persistida com um Canvas, e ambos continuam plenamente editáveis após a criação do vínculo.
9. **AC 9** A configuração de IA contém flag, provider, modelo e estado da chave, mas a chave OpenAI e o token GitHub não são gravados em SQLite, arquivos de Project ou variáveis `VITE_`.
10. **AC 10** A interface de IA oferece entrada por repositório GitHub, arquivo ou combinação dos dois, com stubs honestos quando a feature ou o provider estiver desabilitado.
11. **AC 11** A geração futura produz primeiro um `ArchitectureModel` validado. Documento e Canvas são rascunhos derivados desse mesmo modelo.
12. **AC 12** Um Canvas gerado usa tipos e ports do catálogo existente quando possível, marca inferências e incertezas, recebe layout sem sobreposição e pode ser aprovado independentemente do documento.
13. **AC 13** Nenhum dado de geração é gravado nas entidades reais antes de revisão explícita. A aceitação de cada artefato é transacional e registra revision e vínculo.

## Decision

Usar Markdown com preview seguro como formato canônico inicial. Ele é portátil, funciona offline, reutiliza a direção já iniciada em Notes e torna a exportação trivial. Editor de blocos é uma evolução possível por meio de uma abstração de editor, não uma dependência desta entrega.

Criar uma árvore de Docs semelhante à referência, com um espaço padrão por Project e nós de página ou pasta. A árvore é específica da feature, enquanto a navegação global permanece no `AppSidebar` existente.

Usar Tauri como fronteira de persistência e de rede. React e Zustand nunca recebem a chave OpenAI ou o token GitHub. O provider fica atrás de `AiProvider`; a primeira configuração é pessoal por instalação. Um token GitHub fine grained, somente leitura e guardado no cofre do sistema é a escolha inicial para repositórios privados. OAuth é uma melhoria futura.

Usar repositório mais documentos opcionais como modo recomendado. O repositório é a fonte técnica principal. O documento acrescenta intenção, decisões e sistemas externos que ainda não estão representados no código.

**Implementation skills**: `develop`, `check`, `test`.

## Options considered

1. Editor de blocos como formato primário. Oferece edição rica, mas introduz uma estrutura de conteúdo nova, aumenta o custo de exportação e não reutiliza o padrão atual de Notes.
2. Markdown persistido, com preview seguro. Escolha adotada. Mantém portabilidade, uso offline e exportação simples, sem impedir um editor rico futuro.
3. Chamar a IA pelo frontend. Descartado. Exporia credenciais e permitiria que código local fosse enviado sem uma fronteira desktop controlada.
4. Gerar documentação e Canvas diretamente de texto livre. Descartado. Dois prompts independentes podem divergir e não preservam evidência para a arquitetura desenhada.
5. Usar somente repositório ou somente documento. Ambos continuam disponíveis, mas repositório com documento complementar é o modo recomendado porque combina fatos de código com contexto que não está no código.

## Rationale

Markdown resolve a necessidade atual com o menor acréscimo de complexidade, pois a aplicação já armazena Notes nesse formato. A árvore de páginas e pastas permite reproduzir a referência sem transformar a navegação global em uma cópia por feature.

Tauri já é a fronteira local do Orbit para SQLite e filesystem. Mantê lo também como fronteira de segredo e rede preserva o princípio local first. A primeira integração de GitHub usa token pessoal de leitura restrita porque não cria um segundo fluxo de autenticação para o produto. OAuth deverá entrar somente quando houver necessidade clara de múltiplas contas ou seleção recorrente de repositórios.

O modelo intermediário é a decisão que torna a IA auditável. Evidência, confiança e hipótese ficam no mesmo dado que deriva Markdown e Canvas, permitindo revisão em vez de apresentar uma inferência como se fosse descoberta confirmada.

## Data model

Manter a tabela existente `docs` para preservar Templates. A próxima migration de `project.db` deve adicionar campos de modo aditivo.

| Entidade | Campos principais | Regras |
|---|---|---|
| `doc_spaces` | `id`, `project_id`, `name`, `sort_order`, datas, `deleted_at` | Cada Project recebe um espaço padrão `Docs`. |
| `docs` | campos atuais, `space_id`, `parent_id`, `kind`, `slug`, `sort_order`, `origin`, `generation_id` | `kind` é `page` ou `folder`. Página usa Markdown. Pasta não precisa de conteúdo. |
| `entity_relations` | `id`, `project_id`, tipo e id de origem, tipo e id de destino, `relation_type`, `metadata_json`, data | Substitui vínculos ad hoc e prepara Knowledge Graph. |
| `ai_generations` | `id`, `project_id`, `provider`, `input_mode`, `status`, manifestos e rascunhos JSON, erro, datas | Somente rascunhos. Não é documento ou Canvas real. |
| `revisions` | entidade existente | Criação, atualização, remoção, restauração e aceitação de geração registram payload seguro. |

`docs.parent_id` referencia outro item `docs` do mesmo Project. Uma página ou pasta não pode ser seu próprio pai, nem entrar em um descendente. A migration deve criar índices por Project, espaço, pai, ordenação e `deleted_at`.

Relação de documento para Canvas:

```text
documento  ── documents ou generated_from ──> Canvas
Canvas     ── documented_by ────────────────> documento
```

O segundo sentido é uma consulta derivada, não uma cópia de relacionamento.

## Feature design

### Value sourcing

| Ação ou visualização | Valor produzido ou exibido | Fonte nomeada |
|---|---|---|
| Explorer | nome, tipo, hierarquia e ordem | `docs` e `doc_spaces` do `project.db` |
| Editor | título e Markdown | `docs.title` e `docs.content` |
| Preview | HTML seguro e headings | Markdown do draft atual, renderizado localmente |
| Índice | texto, nível e âncora de cada seção | headings do Markdown do draft atual |
| Estado de autosave | rascunho, salvando, salvo ou erro | fila do `docsStore` e resposta do comando Tauri |
| Exportação | nome e bytes do arquivo | slug e conteúdo persistido do documento |
| Canvas relacionado | Canvas e relação | `entity_relations` mais Canvas do `project.db` |
| Credencial exibida | provider e estado mascarado | `ai_settings` e consulta ao cofre nativo, sem retornar segredo |
| Progresso da análise | etapa, porcentagem e erro | `ai_generations.status` e evento Tauri associado |
| Rascunho de IA | Markdown, nodes, arestas, evidências e confiança | `ai_generations` e `ArchitectureModel` validado |
| Layout do Canvas | posição e dimensões de cada node | layout determinístico derivado de nodes e edges do modelo |

### Invariants

1. Um documento removido não desaparece fisicamente até uma política de retenção posterior.
2. Todo documento, Canvas e relação pertence a um único Project local.
3. O conteúdo real do Project só recebe uma geração após aprovação explícita.
4. Um erro ou cancelamento nunca remove o documento aberto nem uma geração já aceita.
5. Credenciais e conteúdo de segredo não entram em revision, log, rascunho de IA ou exportação.
6. O Canvas aceito mantém seus IDs e pode ser alterado pelo usuário sem a IA.

### Critical test scenarios

1. Criar uma página, editar título e Markdown, trocar de documento imediatamente e verificar a última revisão persistida. Valida AC 3, AC 4 e AC 5.
2. Criar pasta, mover uma página, tentar criar ciclo de árvore e verificar erro sem alteração parcial. Valida AC 3 e AC 7.
3. Abrir documento vindo de Template e verificar conteúdo preservado após a migration. Valida AC 7.
4. Exportar Markdown para caminho escolhido e verificar conteúdo UTF 8 e nome por slug. Valida AC 6.
5. Criar vínculo entre documento e Canvas, reabrir o Project e navegar pelos dois sentidos. Valida AC 8.
6. Desligar flag de IA e verificar ação explicativa, sem chamada de rede. Valida AC 9 e AC 10.
7. Simular resposta com schema inválido, provider indisponível e cancelamento, e verificar que nenhuma entidade real foi criada. Valida AC 10, AC 11 e AC 13.
8. Aceitar somente Canvas e depois somente documento em gerações distintas, verificando revision, vínculo e edição normal dos itens criados. Valida AC 12 e AC 13.

## UI design

```text
AppShell existente
├── AppSidebar existente
└── DocsPage
    ├── DocsExplorer
    │   ├── título, criar documento e criar pasta
    │   ├── busca
    │   └── árvore de espaços, pastas e páginas
    ├── DocumentWorkspace
    │   ├── breadcrumb, status local e exportação
    │   ├── título
    │   ├── toolbar Markdown
    │   ├── MarkdownEditor
    │   └── MarkdownPreview
    └── DocumentInspector
        ├── Conteúdo, com índice de headings
        ├── Links rápidos e Canvas relacionado
        └── geração de IA, quando a flag permitir
```

O design usa os tokens e primitives existentes. `AppShell`, `AppSidebar`, marca Orbit, header global e navegação não são recriados. O explorer e o inspector pertencem a `src/features/docs`.

Estados necessários:

```text
carregando lista
espaço vazio
documento vazio
rascunho alterado
salvando localmente
salvo localmente
erro recuperável
removido, com restaurar
```

O preview processa somente Markdown permitido. Links, imagens e futuras extensões Orbit devem passar por renderer conhecido e sanitização. Não usar `dangerouslySetInnerHTML` para conteúdo do documento.

## Rotas e interface Tauri

| Superfície | Contrato |
|---|---|
| `#docs/:projectId` | Abre o último documento disponível ou estado vazio. |
| `#docs/:projectId/:documentId` | Abre um documento específico. |
| `#docs/:projectId/new` | Cria um rascunho de página e redireciona para ele. |
| `#docs/:projectId/:documentId/review-generation` | Abre revisão de geração pendente. |
| `#settings/ai` | Mostra provider, feature flag, modelo, privacidade e estado mascarado de credenciais. |

| Comando Tauri | Entrada | Saída |
|---|---|---|
| `list_docs` | Project, espaço e pai opcionais | árvore ou itens ordenados |
| `create_doc` | Project, espaço, pai, tipo e título | documento criado |
| `get_doc` | Project e documento | documento completo |
| `update_doc` | Project, documento, título, conteúdo, pai e ordem | documento persistido |
| `trash_doc` e `restore_doc` | Project e documento | estado atualizado |
| `export_doc_markdown` | Project, documento e destino escolhido | arquivo exportado |
| `create_entity_relation` e `list_entity_relations` | Project e entidades | vínculos persistidos |
| `get_ai_capabilities` | nenhum | flags e providers disponíveis |
| `get_ai_settings` e `save_ai_settings` | configuração não secreta | configuração pública |
| `save_openai_api_key` e `delete_openai_api_key` | segredo somente na fronteira Rust | estado mascarado |
| `start_architecture_analysis` | fonte e opções | `generationId` |
| `get_architecture_analysis` e `cancel_architecture_analysis` | geração | estado e rascunhos |
| `apply_generation_review` | itens aceitos e edições | entidades persistidas |

Os comandos de IA começam como stubs e retornam `feature_disabled`, `provider_not_configured` ou `not_implemented`. Eles não devem fingir uma análise.

## Autosave e exportação

O store Docs segue o padrão de `notesStore`, com gateway Tauri separado. O editor mantém `draftRevision` e uma fila de escrita. Cada mudança agenda autosave após aproximadamente 700 ms. Uma escrita mais antiga nunca pode sobrescrever uma revisão mais nova.

Antes de blur, troca de documento, desmontagem ou `Ctrl S`, o editor espera a última gravação pendente. Falha mantém o rascunho local, explica o erro e permite nova tentativa. O estado `salvo localmente` só aparece após resposta do comando Tauri.

A primeira exportação é `.md` UTF 8, com nome derivado do slug e destino escolhido pelo usuário. Recursos anexados e exportação de pacote com assets pertencem a uma fase posterior, para não prometer links `asset://` portáveis antes de existir um empacotador.

## IA e configuração

```text
src/config/features.ts
src/features/docs/
src/features/ai-architecture/
src-tauri/src/docs/
src-tauri/src/ai/
```

```text
AiProvider
├── id()
├── is_configured()
├── validate_connection()
└── analyze(request) -> ArchitectureAnalysisResult

DisabledAiProvider
OpenAiProvider
```

`VITE_ORBIT_AI_STUBS_ENABLED` pode expor ou ocultar a interface de desenvolvimento, pois não é segredo. A habilitação do usuário, provider, modelo, limites e versão do consentimento ficam em `orbit.db`. Chaves não entram em `.env` distribuído, SQLite, logs, payload de revision ou dados do Project.

No Windows, a chave OpenAI e o token GitHub ficam no Windows Credential Manager. Outros sistemas usam o cofre nativo equivalente. O token GitHub é fine grained, somente leitura de conteúdo, limitado aos repositórios selecionados quando possível.

## Pipeline de análise

```text
repositório, documento ou ambos
             ↓
validação local e consentimento
             ↓
manifesto de fontes, normalização e chunks
             ↓
extratores determinísticos e evidências
             ↓
ArchitectureModel validado
             ↓
Markdown draft + Canvas draft
             ↓
revisão independente
             ↓
commit transacional do item aceito
```

### Repositório GitHub

1. Validar URL e acesso.
2. Buscar árvore e arquivos permitidos sem executar código do repositório.
3. Ler sinais objetivos em `README`, manifestos de dependência, Docker, deploy, CI, rotas, serviços externos e variáveis de ambiente seguras.
4. Criar chunks com hash, caminho, trecho e motivo de seleção.
5. Enviar somente o conjunto aprovado ao provider depois de consentimento.

### Upload de documento

1. Aceitar Markdown, texto, PDF e DOCX.
2. Verificar extensão, MIME, tamanho, contagem de páginas e texto extraível.
3. Rejeitar macros, executáveis e conteúdo binário inesperado.
4. Normalizar texto e preservar página ou seção como evidência.
5. Criar chunks iguais aos do modo de repositório.

### Modo combinado

O código e configuração têm precedência para fatos técnicos. O documento complementa decisões, contexto de produto e serviços ainda não modelados em código. Em conflitos, o resultado mostra a diferença para revisão, nunca escolhe silenciosamente.

## ArchitectureModel

```json
{
  "schemaVersion": 1,
  "project": {
    "name": "string",
    "summary": "string",
    "sourceMode": "repository | document | repository_plus_document"
  },
  "sources": [
    {
      "id": "src-001",
      "kind": "repository_file | uploaded_document",
      "path": "string",
      "digest": "sha256 string",
      "selectedReason": "string"
    }
  ],
  "nodes": [
    {
      "id": "stable-id",
      "kind": "web_client | mobile_client | api | service | worker | database | cache | queue | storage | load_balancer | cdn | external_service | unknown",
      "name": "string",
      "technology": ["string"],
      "runtime": "string",
      "exposure": "public | internal | external | unknown",
      "confidence": 0.0,
      "evidence": [{ "sourceId": "src-001", "locator": "string", "quote": "string" }],
      "properties": {}
    }
  ],
  "edges": [
    {
      "id": "stable-id",
      "sourceNodeId": "stable-id",
      "targetNodeId": "stable-id",
      "protocol": "http | sql | event | data | unknown",
      "relationship": "calls | reads_writes | publishes | consumes | serves | unknown",
      "label": "string",
      "confidence": 0.0,
      "evidence": []
    }
  ],
  "assumptions": [
    {
      "id": "stable-id",
      "statement": "string",
      "confidence": 0.0,
      "reason": "string"
    }
  ],
  "documentationOutline": ["string"]
}
```

O schema TypeScript, a validação runtime, o contrato Rust e o JSON Schema de Structured Outputs devem derivar da mesma definição para evitar divergência. Cada objeto exige `additionalProperties: false` no JSON Schema estrito.

## Mapeamento para Canvas

| Tipo do modelo | Tipo existente do catálogo | Ports padrão ou derivados |
|---|---|---|
| `web_client`, `mobile_client`, `client` | `web-client`, `mobile-client`, `client` | HTTP de saída |
| `api`, `gateway` | `api-server` | HTTP de entrada e saída, dados ou SQL quando exigido |
| `service`, `external_service` | `service` | HTTP ou dados derivados das arestas |
| `worker` | `worker` | evento de entrada, dados de saída |
| `database` | `database` ou `postgresql-database` | SQL de entrada |
| `cache` | `cache` ou `redis-cache` | dados de entrada e saída |
| `queue` | `queue` | evento de entrada e saída |
| `storage` | `storage` | dados de entrada e saída |
| `load_balancer` | `load-balancer` | HTTP de entrada e saída |
| `cdn` | `cdn` | HTTP de entrada e saída |
| `unknown` | `custom` | ports derivados, com metadados preservados |

O comando de aceitação cria Canvas, componentes, ports e conexões em uma transação. O layout é determinístico, por camadas de fluxo e domínio, com espaçamento e detecção de colisão. Metadados do node guardam `confidence`, `evidence`, `assumptions` e `originalKind`. O Canvas mostra marcação visual para confiança reduzida, mas continua editável.

## Limites de repositório e privacidade

Prioridade de entrada:

```text
README e documentação
manifestos de dependência
Docker, deploy e CI
env example sanitizado
árvore de pastas
entrypoints, rotas e integrações
arquivos de domínio selecionados
```

Ignorar por padrão:

```text
.git, node_modules, vendor, dist, build, target, coverage,
.next, cache, binários, imagens, minificados, arquivos gerados
e arquivos de segredo
```

O pipeline limita número de arquivos, bytes por arquivo, bytes totais e tamanho de chunk. Nunca envia `.env` real, credenciais, certificados, chaves privadas ou token. Quando truncar uma análise, exibe quais fontes ficaram fora e reduz a confiança em vez de fingir cobertura completa.

## Segurança e falhas

| Situação | Comportamento |
|---|---|
| feature flag desligada | Interface explica que a IA está desabilitada. |
| chave ausente | Ação abre Settings, sem enviar conteúdo. |
| token privado inválido | Erro de acesso, sem expor token. |
| arquivo inválido | Rejeita antes de extrair ou enviar conteúdo. |
| provider falha ou expira | Geração fica em erro recuperável, rascunhos existentes são preservados. |
| schema inválido | Não cria Canvas nem Docs. Mantém diagnóstico de validação. |
| usuário cancela | Para trabalho pendente e preserva somente dados locais já confirmados. |
| aprovação parcial | Cria somente o artefato aceito e registra vínculo de origem disponível. |
| falha de persistência | Reverte a transação e não deixa entidade parcial. |

## Build plan

1. Criar migration aditiva para árvore de Docs, relações e revisions de Docs. Preservar registros atuais de Templates. Satisfaz AC 3, AC 7 e AC 8.
2. Criar tipos, gateway, Zustand store, comandos Tauri e testes de CRUD, soft delete, ordenação, relação e exportação. Satisfaz AC 3, AC 5, AC 6, AC 7 e AC 8.
3. Criar `DocsPage`, explorer, editor, preview seguro, índice e estados de autosave. Validar fidelidade contra `Design/orbitDocsPage.png` sem mudar shared chrome. Satisfaz AC 1 a AC 6.
4. Criar flag, Settings, fronteira de cofre seguro, contratos de provider e comandos stub. Satisfaz AC 9 e AC 10.
5. Criar schema único `ArchitectureModel`, validadores, mapeador de catálogo e layout determinístico em memória, com fixtures. Satisfaz AC 11 e AC 12.
6. Implementar ingestão local de repositório e documentos, consentimento, chunking, evidências e limites. Satisfaz AC 10 e AC 11.
7. Implementar `OpenAiProvider`, drafts, revisão independente e aceitação transacional. Satisfaz AC 11, AC 12 e AC 13.
8. Rodar testes Rust e frontend, build, e validação manual no WebView Tauri para autosave, exportação, árvore, preview, Canvas gerado e estados de falha.

## Consequences

**Positivas**

1. Docs entrega valor local antes de qualquer chave ou rede.
2. Markdown permanece portátil e exportável.
3. Documento e Canvas não se contradizem, pois derivam do mesmo modelo.
4. A IA fica substituível e não acopla a UI à OpenAI.
5. Evidências e confiança tornam hipóteses revisáveis em vez de apresentá las como fatos.

**Tradeoffs aceitos**

1. O primeiro editor não tem blocos ricos nativos.
2. A primeira exportação não empacota assets.
3. GitHub privado começa com token manual, não OAuth.
4. Análise de repositórios grandes pode exigir múltiplos estágios e mostrar cobertura parcial.

## Follow up

1. Adicionar editor de blocos como implementação alternativa de `DocumentEditorAdapter` se Markdown deixar de atender o uso real.
2. Exportar pacote de Markdown e assets após definir um formato portátil de anexos.
3. Adicionar OAuth GitHub sem remover suporte ao token pessoal.
4. Avaliar Docs History e Comments depois que revisions de Docs estiverem expostas.
5. Criar avaliações com repositórios de arquitetura conhecida antes de habilitar IA por padrão.

## References

1. `AGENTS.md`, princípios local first, Docs, Canvas, IDs, relations, secrets e IA.
2. `DESIGN_RULES.md`, AppShell e AppSidebar compartilhados.
3. `Design/orbitDocsPage.png`, referência visual.
4. `src/features/notes`, padrão atual de Markdown, Zustand e autosave.
5. [Structured model outputs, OpenAI API](https://developers.openai.com/api/docs/guides/structured-outputs), contrato JSON Schema estrito para o modelo intermediário.
