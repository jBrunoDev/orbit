Contexto
Orbit é um app desktop de system design (canvas com cards conectados por edges), feito com React + @xyflow/react. Outro agente já preparou a biblioteca de ícones em src/features/catalog/ (catalog.ts, icons.ts, types.ts, index.ts) e o bundle offline em src/assets/icons/bundle.json. Leia esses arquivos primeiro e REUTILIZE tudo: não recrie ícones, não baixe nada novo, não busque ícones online.

Objetivo
Implementar uma "biblioteca de componentes" (banco de componentes) no estilo Blackbird: um painel lateral com todos os itens do catálogo, de onde o usuário arrasta (ou clica) para trazer o componente ao canvas.

Requisitos

1. Camada de acesso aos dados: src/features/catalog/repository.ts
   - Defina a interface CatalogRepository com: getCategories(), getItems(category?), getItemByType(type), search(query).
   - Implemente InMemoryCatalogRepository lendo de catalog.ts e exporte uma instância padrão.
   - A UI só pode acessar o catálogo por essa interface (nunca importar catalog.ts direto), para que no futuro o repository possa ser trocado por um banco real.
   - A busca deve ignorar acentos e maiúsculas, e casar por label, category e type.

2. Painel: src/features/library/LibraryPanel.tsx
   - Painel lateral esquerdo, com campo de busca no topo e categorias colapsáveis (Clients, Compute, Networking, Data, Languages, Tech/Infra).
   - Cada item mostra o ícone (<CatalogIcon />) e o label, com hover state.
   - Estado vazio quando a busca não retorna nada.
   - Reaproveite as cores, tokens e classes do tema escuro que o app já usa. Não invente um tema novo.

3. Drag-and-drop e clique
   - Arrastar: use HTML5 drag and drop com dataTransfer no formato "application/orbit-node" contendo o type do item.
   - No canvas, implemente onDragOver e onDrop e converta a posição com screenToFlowPosition (useReactFlow), criando o node com id único.
   - Fallback: clicar no item adiciona o node no centro da área visível do canvas.

4. Node genérico: src/features/canvas/OrbitNode.tsx
   - Registre como nodeType "orbitNode" e alimente por data: { catalogType, label, subtitle }.
   - O visual deve ser igual ao dos cards atuais do canvas: ícone à esquerda, título e subtítulo, handles nas laterais.
   - O ícone vem do catálogo via getItemByType(catalogType).
   - Duplo clique no título ou no subtítulo permite editar o texto.
   - Se for simples, migre os nodes de exemplo atuais (Client, API Server, Database, Cache) para usar OrbitNode. Se a mudança for grande, mantenha os antigos funcionando lado a lado.

5. Integração
   - Coloque o LibraryPanel no layout principal ao lado do canvas, sem quebrar zoom, pan, conexões nem seleção.
   - Mudanças em arquivos existentes devem ser mínimas e só as necessárias para montar o painel e o onDrop.

6. Fora de escopo (não implementar agora): salvar/carregar diagramas, exportação, undo/redo, banco de dados real.

7. Verificação
   - Rode typecheck e build e corrija os erros.
   - Suba o app em modo dev e teste manualmente: buscar "load", arrastar um Load Balancer, arrastar uma linguagem (ex.: Python), clicar em um item, conectar dois nodes, editar o subtítulo.
   - Confirme que nenhum ícone depende de internet.

Entrega
Liste os arquivos criados e os arquivos existentes alterados, com uma linha sobre cada mudança. Informe também qualquer item do catálogo que ficou sem ícone ou com fallback.