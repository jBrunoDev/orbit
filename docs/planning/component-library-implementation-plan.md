# Planejamento de implementação

## Biblioteca de Components do Orbit

**Data:** 2026-09-21  
**Modo:** Feature  
**Status:** Implementado parcialmente, aguardando verificação manual no shell Tauri  
**Área:** Canvas e Components  
**Abordagem:** Tracer Bullet, primeiro uma fatia completa de catálogo, painel, criação no Canvas e persistência existente, depois refinamentos de busca, edição e verificação.

## Resumo

O Orbit ganhará uma biblioteca lateral de Components baseada no catálogo offline existente. O usuário poderá pesquisar por categoria, arrastar um item para o Canvas ou clicar para adicioná-lo no centro da área visível. A implementação reutilizará o domínio atual do Canvas, o mecanismo de persistência já exposto por `canvasStore` e a linguagem visual existente.

O catálogo e o bundle de ícones citados no briefing não existem neste checkout. Eles são uma pré condição do primeiro marco. O plano não cria ícones novos, não baixa recursos e não substitui o catálogo por dados inventados.

## Contexto verificado

O Canvas atual está em `src/components/CanvasPage.tsx` e usa `@xyflow/react`, `ReactFlowProvider`, `useCanvasStore`, `componentRegistry` e um node local chamado `orbit`. O store já oferece criação, movimento, conexão, atualização, exclusão e seleção de Components através de comandos Tauri.

O layout atual usa `AppShell` com a sidebar principal e um painel de propriedades à direita. O espaço adequado para a biblioteca é uma coluna lateral própria entre a sidebar do aplicativo e a área do Canvas, mantendo o inspector à direita.

O repositório não possui `.git` neste diretório, não possui `docs/scope` nem `docs/specs`, e não possui `src/features/catalog` ou `src/assets/icons`. O planejamento deve ser aplicado após a restauração desses artefatos ou após a aprovação de um trabalho separado para fornecê-los.

## Requisitos e critérios de aceite

### AC 1. Catálogo isolado por uma interface

Deve existir `src/features/catalog/repository.ts` com `CatalogRepository` contendo `getCategories()`, `getItems(category?)`, `getItemByType(type)` e `search(query)`. A implementação em memória deve ler `catalog.ts` e exportar uma instância padrão. A UI deve importar somente o repository ou uma fachada equivalente, nunca `catalog.ts` diretamente.

### AC 2. Busca previsível e offline

A busca deve ignorar acentos e diferença entre maiúsculas e minúsculas. Ela deve comparar o termo normalizado com `label`, `category` e `type`. A busca não pode depender de rede, embeddings ou outro serviço externo.

### AC 3. Painel de biblioteca

`src/features/library/LibraryPanel.tsx` deve exibir campo de busca, categorias Clients, Compute, Networking, Data, Languages e Tech/Infra, estado recolhido ou expandido e itens com ícone, label e estado de hover. Quando não houver resultado, deve haver um estado vazio compreensível.

### AC 4. Arrastar para o Canvas

Cada item arrastável deve usar HTML5 drag and drop e gravar `application/orbit-node` no `dataTransfer`, contendo o `type` do item. O Canvas deve aceitar `onDragOver` e `onDrop`, converter a posição com `screenToFlowPosition` de `useReactFlow` e criar um node com ID único.

### AC 5. Adição por clique

O clique em um item deve criar o mesmo tipo de Component no centro da área visível do Canvas. O comportamento deve compartilhar a mesma função de criação usada pelo drop, evitando divergência entre os dois caminhos.

### AC 6. Node genérico

`src/features/canvas/OrbitNode.tsx` deve ser registrado como `orbitNode` e receber `catalogType`, `label` e `subtitle` em `data`. O visual deve manter o card atual, com ícone à esquerda, título, subtítulo e handles laterais. O ícone deve ser resolvido pelo catálogo através de `getItemByType`.

### AC 7. Edição inline

Duplo clique no título ou no subtítulo deve permitir editar o texto. A edição deve continuar compatível com o inspector e com `updateComponent` do store. O estado de edição deve ter confirmação por blur ou Enter e cancelamento por Escape, sem quebrar seleção ou arraste.

### AC 8. Compatibilidade do Canvas

Zoom, pan, seleção, conexões, minimap, controles, movimentação e exclusão existentes devem continuar funcionando. Os nodes de exemplo Client, API Server, Database e Cache devem ser migrados para `OrbitNode` quando isso não exigir alteração estrutural no backend. Caso a migração seja incompatível com o snapshot atual, o node anterior deve continuar registrado em paralelo durante esta entrega.

### AC 9. Limites da entrega

Não fazem parte desta entrega salvar ou carregar diagramas novos, exportação, undo e redo, banco de dados real para o catálogo ou sincronização em nuvem.

### AC 10. Verificação

Após a implementação, typecheck e build devem passar. O teste manual deve cobrir busca por `load`, arraste de Load Balancer, arraste de Python, clique em um item, conexão entre dois nodes, edição de subtítulo e funcionamento sem internet. O relatório deve listar arquivos criados e alterados e informar itens sem ícone ou com fallback.

## Decisão

Recomenda-se uma camada de acesso em memória sobre o catálogo estático, com o catálogo tratado como fonte de dados e o repository como fronteira de aplicação. O painel usa o repository e emite uma intenção de criação para o Canvas. O Canvas traduz essa intenção para o modelo de Component já persistido pelo store. O React Flow permanece um detalhe da camada de UI, enquanto o tipo de catálogo e os dados do Component continuam sendo o contrato do domínio.

### Escolhas de implementação

| Tema | Decisão | Motivo | Alternativa descartada |
|---|---|---|---|
| Fonte de dados | `InMemoryCatalogRepository` sobre `catalog.ts` | Funciona offline e preserva a futura troca por SQLite | Importar o catálogo diretamente nos componentes |
| Normalização | `normalize` com `normalize('NFD')`, remoção de marcas Unicode e `toLocaleLowerCase` | Cobre acentos e caixa sem dependência nova | Busca exata ou biblioteca de fuzzy search |
| Transporte do drop | HTML5 `dataTransfer` com `application/orbit-node` | É nativo do navegador e não adiciona pacote | Dependência externa de drag and drop |
| Posicionamento | `screenToFlowPosition` | Respeita pan e zoom atuais | Cálculo manual baseado em bounding boxes |
| Identidade | UUID ou gerador já aceito pelo domínio atual | Evita colisão entre criações por clique e drop | Índice baseado no tamanho do array |
| Persistência | Reutilizar `useCanvasStore` e comandos Tauri existentes | Mantém o fluxo atual de Canvas | Criar um segundo store para a biblioteca |
| Compatibilidade visual | Classes e tokens CSS existentes do Canvas e Orbit | Mantém a linguagem escura atual | Novo tema ou cores locais espalhadas |

## Modelo de dados

Não há migração de banco nesta entrega. O catálogo é somente leitura e vive no bundle. O Component criado continua usando o snapshot atual do Canvas.

| Entidade | Campos usados | Relação |
|---|---|---|
| CatalogItem | `type`, `label`, `category`, `icon`, `color`, `description`, `ports`, `simulationDefaults` | Item de catálogo, sem persistência própria |
| PersistedComponent | `id`, `canvasId`, `componentType`, `label`, `description`, `x`, `y`, `width`, `height`, `color`, `data`, `ports` | Muitos Components pertencem a um Canvas |
| CatalogRepository | quatro operações de leitura | Consumido pelo painel e pelo node |

O `type` do catálogo deve ser compatível com o `componentType` aceito pelo store. Se o catálogo incluir linguagens ou itens sem ports, o adaptador deve fornecer defaults explícitos, sem alterar silenciosamente o tipo do item.

## Interfaces e fluxo

### Repository

```ts
interface CatalogRepository {
  getCategories(): string[];
  getItems(category?: string): CatalogItem[];
  getItemByType(type: string): CatalogItem | undefined;
  search(query: string): CatalogItem[];
}
```

Os tipos exatos devem reutilizar `types.ts` do catálogo quando ele estiver presente. `getCategories` deve preservar a ordem oficial do catálogo. `getItems` sem categoria deve retornar todos os itens. `search` com consulta vazia deve retornar todos os itens ou a mesma visão agrupada do painel.

### Ações de UI

| Ação | Entrada | Saída | Erros e estados |
|---|---|---|---|
| Pesquisar | Texto do campo | Itens filtrados, agrupados por categoria | Estado vazio quando a lista fica vazia |
| Expandir categoria | Identificador da categoria | Categoria aberta ou fechada | Estado local, sem persistência |
| Arrastar item | `CatalogItem.type` | Evento `application/orbit-node` | Ignorar drop sem type válido |
| Soltar no Canvas | Evento de drop e viewport | Component criado na posição convertida | Mostrar erro existente do Canvas se a criação falhar |
| Clicar item | `CatalogItem.type` | Component criado no centro visível | Usar o mesmo adaptador de criação do drop |
| Editar node | Campo e texto | `updateComponent` | Escape cancela, blur ou Enter confirma |

### Origem dos valores

| Valor necessário | Fonte |
|---|---|
| Label, categoria e type | CatalogRepository |
| Ícone | `getItemByType(catalogType)` e bundle offline |
| Posição de drop | Evento de ponteiro convertido por `screenToFlowPosition` |
| Posição de clique | Viewport atual do React Flow e dimensões do Canvas |
| ID do Component | Gerador de ID definido no adaptador de criação |
| Ports, cor e defaults | Item de catálogo, com fallback explícito quando permitido |
| Persistência | `useCanvasStore.addComponent` ou nova ação mínima que receba posição |
| Texto editado | Evento de edição do `OrbitNode`, enviado a `updateComponent` |

## Segurança e autorização

Não há autenticação, autorização, PII ou integração externa. O catálogo é offline e somente leitura. Nenhum token, segredo ou dado de usuário novo deve ser introduzido.

## Estados e casos de falha

1. Catálogo vazio ou indisponível: o painel exibe estado vazio com mensagem de configuração, sem quebrar o Canvas.
2. Item sem ícone: usar o fallback definido pelo bundle ou `shape`, registrar o item no relatório de verificação e não buscar na internet.
3. Drop sem MIME esperado: ignorar sem criar node.
4. Type desconhecido: rejeitar a criação com erro local compreensível, sem inserir Component inválido.
5. Falha no comando Tauri: preservar o estado anterior e exibir o erro já usado pelo Canvas.
6. Busca sem resultados: mostrar estado vazio e permitir limpar a consulta.
7. Clique antes de o Canvas estar pronto: desabilitar a ação ou enfileirar somente quando houver `canvasId` e viewport disponíveis.
8. Edição vazia: manter o valor anterior ou aplicar o label padrão do catálogo, conforme a regra atual do inspector. Esta decisão deve ser confirmada durante a implementação antes de codificar a validação.
9. ID repetido: o gerador deve produzir uma nova identidade sem depender do tamanho da lista.

## Acessibilidade e desempenho

Itens devem ser botões ou elementos com papel e foco equivalentes, ter nome acessível, foco visível e indicação que também podem ser clicados. O drag and drop não pode ser o único caminho. Categorias devem usar controles de botão com `aria-expanded`. O painel deve funcionar em janela menor sem esconder o inspector.

O repository pode manter os dados em memória. A lista pode ser calculada com `useMemo` a partir da consulta e da categoria, sem busca remota. Não adicionar virtualização ou fuzzy search antes de medir necessidade.

## Plano de execução

### Marco 0. Restaurar pré requisitos

1. Confirmar ou restaurar `src/features/catalog/catalog.ts`, `types.ts`, `icons.ts`, `index.ts` e `src/assets/icons/bundle.json`.
2. Confirmar que o bundle contém os ícones usados pelo catálogo, incluindo Load Balancer e Python.
3. Corrigir somente os imports quebrados existentes em `OrbitIcon` se os artefatos restaurados usarem outro caminho.
4. Registrar qualquer item sem ícone antes de iniciar a UI.

### Marco 1. Repository e adaptador de criação

1. Criar `src/features/catalog/repository.ts`.
2. Definir a interface e a implementação em memória.
3. Implementar normalização de acentos e caixa.
4. Criar um adaptador de criação que converta `CatalogItem` para `ComponentDefinition` ou estenda minimamente `addComponent` para aceitar posição.
5. Garantir que a posição recebida pelo drop e pelo clique seja preservada.

### Marco 2. Node genérico

1. Extrair o node atual de `CanvasPage.tsx` para `src/features/canvas/OrbitNode.tsx`.
2. Alterar o tipo registrado para `orbitNode`, mantendo um alias temporário se o snapshot exigir `orbit`.
3. Resolver ícone por `getItemByType`.
4. Implementar edição por duplo clique com confirmação, cancelamento e chamada a `updateComponent`.
5. Migrar os quatro exemplos atuais ou manter os dois registros durante a transição.

### Marco 3. Painel e integração visual

1. Criar `src/features/library/LibraryPanel.tsx`.
2. Implementar busca, categorias, recolhimento, hover, foco e estado vazio.
3. Inserir o painel como coluna esquerda específica do layout do Canvas.
4. Remover ou reduzir a lista de favoritos somente se a biblioteca cobrir a mesma responsabilidade, preservando ações existentes enquanto necessário.
5. Reutilizar classes, tokens e cores já presentes em `CanvasPage.css`, `CanvasLayout.css` e `AppShell.css`.

### Marco 4. Drop, clique e regressão do Canvas

1. Envolver o Canvas com o contexto necessário para `useReactFlow`.
2. Adicionar `onDragOver` com `preventDefault` e `onDrop` com validação do MIME.
3. Converter coordenadas com `screenToFlowPosition`.
4. Implementar clique no centro da área visível.
5. Verificar seleção, pan, zoom, handles, conexão, minimap, movimento e exclusão.

### Marco 5. Verificação e entrega

1. Rodar typecheck e build.
2. Subir o app em modo dev pelo comando definido no projeto.
3. Executar o roteiro manual do briefing.
4. Verificar o bundle sem acesso à rede.
5. Corrigir erros encontrados.
6. Produzir relatório com arquivos criados, arquivos alterados e itens sem ícone ou com fallback.

## Testes críticos

| Cenário | Critério |
|---|---|
| Busca `load` | Load Balancer aparece na categoria correta |
| Busca com acento e caixa diferente | Resultado permanece equivalente |
| Arraste de Load Balancer | Node é criado na posição sob o cursor |
| Arraste de Python | Item de Languages é criado usando ícone offline |
| Clique em item | Node aparece no centro visível |
| Conexão | Handles e `onConnect` continuam funcionando |
| Edição de subtítulo | Texto atualizado no node e no inspector |
| Busca sem resultados | Estado vazio aparece e não quebra categorias |
| Drop inválido | Nenhum node é criado |
| Ícone ausente | Fallback é visível, local e relatado |
| Janela menor | Painel e inspector continuam acessíveis |

## Consequências

Esta solução mantém o catálogo substituível e o comportamento offline, mas exige um contrato estável entre tipos do catálogo, `ComponentDefinition` e o snapshot persistido. Componentes catalogados que não coincidirem com os tipos atualmente aceitos no backend precisarão de um adaptador explícito ou de uma extensão futura do domínio.

O painel adicionará uma coluna ao layout do Canvas e pode exigir ajuste responsivo. A edição inline aumenta a superfície do node e deve ser testada para não capturar eventos de arraste. O catálogo continua somente leitura nesta entrega, portanto criação e edição de itens personalizados permanecem fora do escopo.

## Pendências antes do desenvolvimento

1. Restaurar ou fornecer os arquivos do catálogo e do bundle offline.
2. Confirmar a regra para subtítulo vazio: preservar o valor anterior ou voltar ao default do catálogo.
3. Confirmar o comando de desenvolvimento, pois `package.json` possui `dev` para Vite, mas o teste manual precisa do shell Tauri para validar persistência.
4. Depois deste planejamento, executar `/develop` para implementação e `/check verify` para validação do fluxo manual.

## Racional

## Implementação realizada

### Arquivos criados

* `src/features/catalog/types.ts`, contrato dos itens e portas do catálogo.
* `src/features/catalog/catalog.ts`, catálogo offline inicial com seis categorias.
* `src/features/catalog/repository.ts`, interface e implementação em memória com busca normalizada.
* `src/features/catalog/icons.ts`, `CatalogIcon` e fallback local.
* `src/features/catalog/index.ts`, exports públicos da feature.
* `src/features/library/LibraryPanel.tsx`, pesquisa, categorias recolhíveis, clique e drag and drop.
* `src/features/canvas/OrbitNode.tsx`, node genérico com handles, ícone do catálogo e edição inline.
* `src/assets/icons/bundle.json`, manifesto local do bundle e relação de ícones esperados.

### Arquivos alterados

* `src/components/CanvasPage.tsx`, integração do painel, drop com `screenToFlowPosition`, clique centralizado e registro de `orbitNode`.
* `src/components/CanvasLayout.css`, coluna visual da biblioteca e estados do painel e do node.
* `src/features/canvas/application/canvasStore.ts`, suporte a posições explícitas e dados de projeto no node.
* `src/features/canvas/domain/componentRegistry.ts`, aceitação de tipos de catálogo além dos exemplos iniciais.
* `src/shared/OrbitIcon.tsx`, fallback SVG local quando um asset não estiver disponível.

### Verificação executada

`npm.cmd run build` passou com typecheck e build Vite. A verificação manual no aplicativo Tauri ainda precisa confirmar criação persistida, conexão entre nodes, edição inline e funcionamento sem rede.

### Fallbacks de ícone

Como os SVGs originais citados no briefing não estavam presentes, o bundle foi documentado e `OrbitIcon` usa um fallback SVG local para os nomes sem asset. A implementação não baixa ícones e não depende de internet. Quando o bundle original for restaurado, o fallback será substituído automaticamente pelos assets encontrados pelo glob existente.

O plano segue o stack já aprovado no `AGENTS.md`, reaproveita o store e o React Flow existentes e mantém o catálogo atrás de uma interface. A ordem prioriza uma fatia completa e verificável, deixando banco real, sincronização, exportação e histórico fora da mudança.
