# Templates Library

**Status**: In Progress

## Summary

Templates dará ao usuário um caminho rápido para adicionar conteúdo inicial completo e editável a um Project local já existente. A página seguirá o design aprovado, mostrará templates oficiais, favoritos persistentes e detalhes no inspector. Cada aplicação cria cópias independentes de Canvas, Notes e Docs sem substituir o conteúdo do Project.

## Context

O Orbit já possui Projects locais, Canvas, assets e Notes, mas não possui Templates nem uma superfície Docs pronta. O usuário aprovou nove templates oficiais. A coleção precisa funcionar offline, sem marketplace, conta ou serviços externos e respeitar a hierarquia Workspace → Project → resources: a criação de Project permanece um fluxo explícito da Home.

> ⚠️ Premise note: a aplicação precisa persistir várias áreas do Project. Criar cada área em passos isolados deixaria conteúdo parcial após uma falha. A implementação deve usar uma transação no banco do Project.

## Requirements

- **AC-1** A rota Templates mostra busca, filtros, seções Popular, Development e Personal & Study, cards, inspector e sidebar conforme `Design/orbitTemplatesPage.png`.
- **AC-2** O catálogo inicial contém Web Application Architecture, Microservices Architecture, AWS Infrastructure, Next.js Application, API Architecture, Database Schema, Daily Planner, Study Notes e Meeting Notes.
- **AC-3** O usuário pode favoritar um template oficial. O favorito é persistido no perfil local e aparece antes das seções do catálogo.
- **AC-4** Cada `See all` abre uma visão interna da categoria escolhida, sem sair da área Templates.
- **AC-5** Ao escolher `Use Template`, o usuário escolhe um Project local existente e adiciona Canvas, Notes e Docs completos, editáveis e independentes.
- **AC-6** A aplicação registra a origem e a versão do template na revision local e abre o Canvas criado ao terminar.
- **AC-7** Criar AWS Infrastructure habilita a biblioteca AWS global do perfil local.
- **AC-8** Se qualquer etapa de geração falhar, o Orbit não mantém um Project parcial nem um diretório órfão.
- **AC-9** `New Template` permanece visível, mas indisponível, explicando que templates personalizados pertencem a uma etapa futura.

## Decision

Usar manifestos TypeScript somente leitura para templates oficiais, SQLite global para favoritos e uma operação Tauri para aplicar o conteúdo completo a um Project existente. O manifest contém snapshots de conteúdo, nunca referências mutáveis a outro Project.

**Implementation skills**: none.

## Options considered

1. Criar ou alterar o Project pela UI com várias chamadas separadas. É curto inicialmente, mas falha no meio e deixa dados incompletos.
2. Usar uma operação Tauri que aplica Canvas, Notes e Docs ao Project escolhido como uma única unidade. Escolha adotada, pois protege a integridade local.
3. Armazenar templates oficiais inteiros em SQLite. Permite consultas uniformes, mas duplica conteúdo imutável e exige migrations sem benefício no catálogo inicial.

## Rationale

O catálogo é pequeno, oficial e offline. Manifestos locais simplificam a leitura e preservam os SVGs e a estrutura como dados confiáveis do aplicativo. Uma operação única de aplicação combina a velocidade esperada pelo usuário com a garantia de que o conteúdo criado pertence a um Project escolhido explicitamente.

## Feature design

**Data model sketch**:

| Entidade | Persistência | Campos e regras |
|---|---|---|
| TemplateDefinition | manifesto TypeScript | `id`, `version`, `name`, `description`, `categories`, `tags`, `section`, `preview`, `canvas`, `notes`, `docs`, `requiresAws`; ID e versão são estáveis. |
| TemplateFavorite | `orbit.db` | `profile_id`, `template_id`, `created_at`; único por perfil e template. |
| Revision | `project.db` | `templateId` e `templateVersion`; registra cada aplicação sem limitar o Project a uma única origem. |
| Template content | `project.db` e filesystem | cópias de nodes, conexões, visuais, Notes, Docs e assets de preview quando necessários. |

**State transitions**:

```text
template selecionado
→ Project existente escolhido
→ conteúdo completo aplicado
→ Canvas criado aberto

template selecionado
→ falha de persistência
→ transação revertida e diretório removido
```

**API surface**:

| Command Tauri | Entradas principais | Saídas principais | Autorização | Erros principais |
|---|---|---|---|---|
| `list_templates` | busca opcional, categoria opcional | definições oficiais e favoritos | perfil local | filtro inválido |
| `set_template_favorite` | `templateId`, favorito | estado favorito | perfil local | template desconhecido |
| `apply_template_to_project` | `templateId`, `projectId` | `projectId`, `canvasId` | perfil local | Project ausente, template desconhecido, persistência falhou |

**Value sourcing**:

| Ação | Valor exibido ou produzido | Fonte |
|---|---|---|
| Lista de templates | nome, preview, tags, categoria e conteúdo | `TemplateDefinition` local |
| Favoritos | ordem e estado de estrela | `TemplateFavorite` do perfil local |
| Aplicação | Project de destino | seletor de Projects locais |
| Aplicação | conteúdo inicial | snapshot do `TemplateDefinition` |
| Revision | `templateId` e versão | `TemplateDefinition` selecionado |
| AWS Infrastructure | biblioteca AWS habilitada | `requiresAws` no manifesto |

**Key invariants**:

- Templates oficiais são imutáveis e nunca são alterados pelo Project que os recebe.
- O conteúdo existente do Project nunca é sobrescrito pela aplicação de um Template.
- Um favorito não cria cópia local do template.
- A biblioteca AWS só é habilitada automaticamente pelo template AWS Infrastructure.
- Falha em qualquer escrita reverte a transação do conteúdo aplicado.

**Security model**:

Todas as operações pertencem ao perfil local ativo. Não há rede, conta, segredo ou conteúdo público nesta entrega. Os manifestos e seus caminhos de asset são empacotados e confiáveis, nunca recebidos de entrada do usuário.

**Critical test scenarios**:

- Happy path: aplicar Web Application Architecture a um Project existente, abrir o Canvas criado e verificar Canvas, Notes, Docs e revision, valida **AC-5** e **AC-6**.
- Happy path: aplicar AWS Infrastructure e verificar a habilitação da biblioteca AWS, valida **AC-7**.
- Failure case: forçar falha ao persistir um recurso e verificar ausência de conteúdo parcial no Project, valida **AC-8**.
- Local profile: favoritar template, reiniciar o aplicativo e verificar seção Favoritos, valida **AC-3**.

## Build plan

1. Criar manifestos oficiais para os nove templates, incluindo snapshots de Canvas, Notes e Docs, satisfaz **AC-2** e **AC-5**.
2. Adicionar migration global para favoritos e migration de Project para metadata de template e Docs iniciais, satisfaz **AC-3**, **AC-5** e **AC-6**.
3. Implementar comandos de listagem, favorito e aplicação atômica no Project selecionado, satisfaz **AC-3**, **AC-5**, **AC-6**, **AC-7** e **AC-8**.
4. Criar estado TypeScript e rota Templates, incluindo seleção do Project de destino, navegação de categoria e abertura do Canvas criado, satisfaz **AC-4**, **AC-5** e **AC-6**.
5. Construir a página visual baseada no design, com busca, filtros, cards, inspector, favoritos e estado indisponível de `New Template`, satisfaz **AC-1**, **AC-3** e **AC-9**.
6. Adicionar testes Rust e frontend, depois validar a criação no shell Tauri, satisfaz **AC-1** até **AC-9**.

## Consequences

**Positive**:

- O usuário inicia um Canvas completo sem configurar manualmente a estrutura inicial.
- O conteúdo criado continua editável e local, sem acoplamento ao catálogo ou criação implícita de Project.
- O histórico de cada aplicação fica disponível para suporte e evolução futura.

**Negative / tradeoffs**:

- A aplicação exige migrations para Docs que ainda não existem no Project.
- Cada template precisa manter snapshots de conteúdo consistentes ao longo das versões.

**Neutral**:

- Templates personalizados, importação e marketplace não fazem parte desta entrega.
- A paginação não é necessária para nove templates oficiais, mas a busca e o filtro devem usar o manifesto de forma determinística.

## Follow-up

- [ ] Planejar templates personalizados, duplicação e importação após validar o catálogo oficial.
- [ ] Planejar atualização assistida de conteúdo criado por versões futuras de templates, sem sobrescrever conteúdo do usuário.
