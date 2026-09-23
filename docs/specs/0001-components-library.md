# Components Library

**Status**: In Progress

## Summary

Components será o catálogo global local do Orbit. Ele separa o catálogo oficial, a biblioteca pessoal e as instâncias usadas em cada Canvas. A página permitirá encontrar, habilitar, favoritar e salvar Components para uso posterior em qualquer Project.

## Context

O Canvas já cria Components a partir de um catálogo estático, mas ainda não possui uma página global, biblioteca pessoal, favoritos nem bibliotecas opcionais. A coleção AWS já está no checkout local e contém SVGs oficiais em várias categorias e tamanhos.

> ⚠️ Premise note: tratar os 1.852 SVGs AWS como cards iguais ao catálogo principal tornaria busca, carregamento e navegação ruins. A primeira versão usa somente ícones de serviços AWS, com manifesto deduplicado e acesso progressivo.

## Requirements

- **AC-1** A página Components mostra catálogo oficial, busca, categorias, Featured e painel de detalhes segundo `Design/orbitComponentsPage.png`.
- **AC-2** Um perfil local novo recebe Client, API Server, Database, Cache e Load Balancer na biblioteca pessoal.
- **AC-3** O usuário pode adicionar e remover um Component oficial ou tecnológico da biblioteca pessoal global. A alteração vale para Projects atuais e futuros, mas não remove instâncias existentes no Canvas.
- **AC-4** O usuário pode favoritar um Component. Favoritar também o adiciona à biblioteca pessoal. Desfavoritar apenas o remove da seção Favoritos.
- **AC-5** O Canvas mostra Favoritos antes de Clients e depois mostra apenas os Components da biblioteca pessoal, agrupados por categoria.
- **AC-6** O usuário pode criar ou duplicar um Component personalizado global com nome, descrição, categoria, tags, cor e ícone de biblioteca habilitada. Cada inserção no Canvas é uma cópia independente.
- **AC-7** AWS Architecture Icons é uma biblioteca local, instalada mas desabilitada por padrão. Habilitá la vale para todo o perfil local. Desabilitá la apenas a oculta.
- **AC-8** A página mostra uma prévia AWS de no máximo duas fileiras e abre a coleção completa no modo `Components / AWS Architecture Icons`, com busca e categorias AWS.
- **AC-9** A coleção AWS inicial contém somente serviços. Recursos e grupos de arquitetura permanecem fora desta entrega.

## Decision

Usar SQLite no catálogo global para preferências e Components personalizados. Manter definições oficiais em manifestos TypeScript somente leitura. A biblioteca pessoal aponta para definitions oficiais ou guarda sua própria definição personalizada.

**Implementation skills**: none.

## Options considered

1. Manter somente catálogo estático no bundle. É simples, mas não suporta biblioteca pessoal nem favoritos.
2. Persistir todo o catálogo oficial em SQLite. Permite consultas iguais, mas duplica dados imutáveis e aumenta migrations sem benefício.
3. Catálogo oficial em manifestos locais e escolhas pessoais no SQLite. Escolha adotada, pois preserva uso offline e permite personalização global.

## Rationale

O catálogo deve ser disponível offline e carregado sem rede. A preferência pessoal é dado do usuário e precisa sobreviver entre Projects. Instâncias já colocadas em Canvas devem ser snapshots, pois uma alteração global não pode reescrever diagramas existentes.

## Feature design

**Data model sketch**:

| Entidade | Campos principais | Relações e regras |
|---|---|---|
| `component_libraries` | `id`, `label`, `enabled`, `sort_order`, `updated_at` | Uma linha por biblioteca local. `orbit-core` ativa e `aws` desativada no primeiro perfil. |
| `library_components` | `id`, `source_kind`, `definition_key`, `custom_definition_id`, `added_at`, `removed_at` | Biblioteca pessoal global. Uma origem ativa por usuário é única. |
| `component_favorites` | `library_component_id`, `created_at` | Um favorito por item da biblioteca. |
| `custom_component_definitions` | `id`, `name`, `description`, `category`, `tags`, `color`, `icon_library_id`, `icon_key`, `created_at`, `updated_at`, `deleted_at` | Campos básicos apenas. Pertence ao perfil local. |
| `aws_service_manifest` | bundle somente leitura | Chave, nome, categoria AWS e caminho relativo de um SVG de serviço. Não entra no SQLite. |

Definitions oficiais incluem tipo, label, descrição, categoria, tags, ícone, portas e defaults. Profiles tecnológicos como PostgreSQL Database, Redis Cache, Docker Container e Kubernetes Cluster ficam no catálogo `orbit-core` com ícones neutros do Orbit.

**State transitions**:

```text
Catalog component → added to library → favorited or available by category
custom draft → saved globally → added to library
AWS disabled ↔ AWS enabled
```

**API surface**:

| Command Tauri | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|
| `list_component_catalog` | query, category, libraryId | cards, categories, availability | local profile | invalid filter |
| `list_personal_library` | none | active Components and favorites | local profile | unavailable profile |
| `set_component_library_enabled` | libraryId, enabled | library state | local profile | unknown library |
| `add_library_component` | definition key or custom id | library item | local profile | duplicate source |
| `remove_library_component` | library item id | success | local profile | item is core base |
| `set_component_favorite` | library item id, favorite | favorite state | local profile | item not in library |
| `create_custom_component` | basic fields | custom definition and library item | local profile | invalid icon or empty name |
| `duplicate_custom_component` | source id, basic fields | custom definition and library item | local profile | unknown source |

**Value sourcing**:

| Action | Value produced or displayed | Source |
|---|---|---|
| Catalog search | label, category, tags and icon | official manifests and custom definitions |
| AWS preview | first services shown | AWS service manifest priority field |
| Canvas library | categories and favorite order | active `library_components` and `component_favorites` |
| Create custom Component | basic fields and selected icon | form input and enabled library manifest |

**Key invariants**:

- A hidden library never removes data, files or existing Canvas instances.
- Favoriting creates an active library membership before the favorite record.
- Core base Components cannot be removed from the initial library, but may be unfavorited.
- Only SVG service files selected by `aws_service_manifest` render in the AWS catalog.
- Custom Components only select icons from enabled local libraries.
- Canvas Components remain snapshots after insertion.

**Security model**:

All writes are local to the active profile. No network request, token, secret or cloud account is needed. The asset path comes only from the trusted local manifest, never from user supplied file paths.

**Critical test scenarios**:

- Happy path: enable AWS, add Lambda to the personal library and insert it from Canvas, verifies AC-3, AC-5, AC-7 and AC-8.
- Failure case: disable AWS after an AWS service is already present in Canvas, verifies AC-3 and AC-7.
- Failure case: attempt to create a custom Component with an icon from a disabled library, verifies AC-6 and AC-7.

## Build plan

1. Add catalog database migration and commands for library state, membership, favorites and custom definitions, including non destructive initialization for existing profiles, satisfies **AC-2**, **AC-3**, **AC-4**, **AC-6** and **AC-7**.
2. Split catalog manifests into Orbit core, technology profiles and AWS service manifest. Build the AWS manifest from local SVG service assets only, satisfies **AC-1**, **AC-7** and **AC-9**.
3. Add repository and Zustand state for catalog search, enabled libraries, personal library, favorites and custom Components, satisfies **AC-1** through **AC-7**.
4. Build Components page from the reference, including cards, filters, detail inspector, library actions and custom Component dialog, satisfies **AC-1**, **AC-3**, **AC-4** and **AC-6**.
5. Add AWS library callout, two row preview and full catalog mode with progressive SVG loading, satisfies **AC-7**, **AC-8** and **AC-9**.
6. Replace the Canvas library panel source with the personal library and Favorites first ordering, satisfies **AC-2**, **AC-3**, **AC-4** and **AC-5**.
7. Add Rust and frontend regression tests, then validate the full flow in the Tauri shell, satisfies **AC-1** through **AC-9**.

## Consequences

**Positive**:

- O usuário controla uma biblioteca pessoal reutilizável sem rede.
- AWS pode ser usada sem poluir a experiência padrão.
- Templates futuros usam definitions estáveis já escolhidas pelo usuário.

**Negative / tradeoffs**:

- O catálogo passa a ter migrations e estado global adicional.
- A criação inicial não inclui portas ou configuração de simulação para Components personalizados.

**Neutral**:

- A paginação da coleção AWS será feita em memória sobre o manifesto local, pois não existe serviço remoto.

## Follow-up

- [ ] Planejar recursos e grupos AWS como elementos estruturais do Canvas.
- [ ] Planejar propriedades avançadas, portas e comportamento de simulação para Components personalizados.
- [ ] Planejar outras bibliotecas locais de provedores após validar AWS.

## References

**Project sources**:

- `AGENTS.md`, princípios local first, assets no filesystem e Components reutilizáveis.
- `Design/orbitComponentsPage.png`, composição e linguagem visual.
- `assets/Icon-package_07312026.5846e92413caa21490223536cc97f1269e44fa92`, pacote AWS local.

**Links**:

- [AWS Architecture Icons](https://aws.amazon.com/architecture/icons/)
