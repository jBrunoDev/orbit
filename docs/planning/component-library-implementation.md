# Implementação da biblioteca de Components

## Estado

Implementação inicial concluída e typecheck validado. O teste manual no shell Tauri permanece pendente.

## O que foi entregue

O Canvas agora possui uma biblioteca lateral offline com seis categorias. A busca usa `label`, `category` e `type`, removendo acentos e ignorando maiúsculas e minúsculas.

Cada item pode ser clicado para aparecer no centro da área visível ou arrastado para qualquer ponto do Canvas. O drop usa `application/orbit-node` e converte a posição com `screenToFlowPosition`.

Os nodes usam `orbitNode`, resolvem seus ícones pelo `CatalogRepository`, preservam handles tipados e permitem editar título e subtítulo com duplo clique. A criação continua usando o store e os comandos Tauri existentes.

## Arquivos novos

| Arquivo | Responsabilidade |
|---|---|
| `src/features/catalog/types.ts` | Tipos de itens e portas do catálogo |
| `src/features/catalog/catalog.ts` | Dados offline iniciais |
| `src/features/catalog/repository.ts` | Fronteira de acesso e busca |
| `src/features/catalog/icons.ts` | `CatalogIcon` e fallback |
| `src/features/catalog/index.ts` | Exports da feature |
| `src/features/library/LibraryPanel.tsx` | Painel, categorias e drag and drop |
| `src/features/canvas/OrbitNode.tsx` | Node genérico e edição inline |
| `src/assets/icons/bundle.json` | Manifesto do bundle local |

## Arquivos alterados

`CanvasPage.tsx`, `CanvasLayout.css`, `canvasStore.ts`, `componentRegistry.ts` e `OrbitIcon.tsx` foram ajustados para integrar o painel, aceitar posições de criação, registrar `orbitNode` e usar fallback local quando um SVG não existir.

## Verificação

```text
npm.cmd run build
```

Resultado: typecheck e build Vite passaram.

## Pendências de validação manual

1. Abrir o shell Tauri e confirmar que um projeto local carrega.
2. Buscar `load` e confirmar o Load Balancer.
3. Arrastar Load Balancer e Python para o Canvas.
4. Clicar em um item e confirmar criação no centro visível.
5. Conectar dois nodes.
6. Editar o subtítulo com duplo clique.
7. Reabrir o projeto e confirmar persistência.
8. Testar sem rede e confirmar que nenhum ícone solicita recurso externo.

## Ícones

Os SVGs referenciados no briefing não estavam presentes no checkout. Por isso, o catálogo usa o manifesto offline e `OrbitIcon` usa um desenho local de fallback para nomes sem asset. Não houve download de ícones. Ao restaurar os SVGs originais em `src/assets/icons`, o glob existente poderá resolvê-los automaticamente.
