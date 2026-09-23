# Prompt de implementação: página Canvas

## Status

Este prompt foi gerado em 21 de setembro de 2026 a partir de `Design/orbitCanvasPage.png`. Ele define a próxima etapa de implementação e deve ser aprovado antes de qualquer código da página Canvas ser alterado.

## Contexto confirmado

O Orbit é um aplicativo desktop com Tauri 2, React, TypeScript e Tailwind. A Home atual já possui o app shell e os tokens visuais em `src/App.css`. O backend Rust mantém um catálogo local em SQLite e um `project.db` por Project, mas ainda não possui entidades, migrations ou commands para Canvas.

Não existe uma biblioteca de edição de grafos instalada. A primeira entrega deve usar uma superfície visual controlada pelo React e dados de demonstração, sem introduzir React Flow, autenticação, sincronização, servidor local ou persistência de nós. A modelagem e persistência reais do Canvas ficam para uma etapa posterior.

## Objetivo

Implementar a tela desktop Canvas fiel à referência `Design/orbitCanvasPage.png`, como uma experiência de visualização de um Project chamado `E-commerce Architecture`.

A tela deve reutilizar a identidade visual da Home: fundo escuro, bordas discretas, brilho roxo, tipografia clara e ícones locais presentes em `assets/icons`.

## Escopo visual

### Estrutura

1. Manter a sidebar existente e marcar `Canvas` como item ativo.
2. Criar uma barra superior do Project com voltar, avançar, nome do Project, estado Private, ações Simulate, Share, avatar e menu.
3. Exibir as abas Canvas, Notes e Run no topo direito, com Canvas ativa.
4. Criar uma barra de ferramentas flutuante com seleção, mão, retângulo, texto, conexão, comentário, círculo e controles adicionais. Os controles podem ser estáticos nesta fase.
5. Criar a área central como uma malha pontilhada escura, ocupando o espaço principal disponível.
6. Criar o painel Properties à direita, com a aba AI Assist inativa e os grupos mostrados na referência.
7. Criar minimapa e controles de zoom no canto inferior direito da área Canvas.

### Conteúdo do Canvas

Representar visualmente o fluxo de arquitetura da referência, usando cards com bordas coloridas, conectores e textos:

1. Título manuscrito `E-commerce Architecture` e lista MVP no canto superior esquerdo.
2. Nota adesiva amarela de melhorias.
3. Fluxo principal: Client, CDN, Load Balancer, dois API Server, Worker, Database, Cache e Storage.
4. Grupo tracejado External Services, com Payment Provider e Notifications.
5. Conectores com cores coerentes: verde para request, roxo para response e evento, amarelo para assíncrono.
6. Checklist Próximos passos, nota adesiva verde e legenda no rodapé.
7. Anotações manuscritas decorativas que indiquem decisões de arquitetura.

Use os SVGs locais correspondentes a navegação e elementos de canvas. Não voltar a usar CSS mask para ícones, pois esse mecanismo falhou no WebView do Tauri. Os SVGs devem continuar a ser carregados pelo mecanismo já usado na Home.

## Comportamento da primeira entrega

1. A Home deve conseguir abrir esta tela ao selecionar Canvas, sem alterar a criação local de Projects existente.
2. O botão de voltar deve retornar à Home.
3. O estado ativo da sidebar e da aba Canvas deve refletir a tela atual.
4. Zoom, ferramentas, Properties, Simulate, Share, Notes, Run e AI Assist são visuais e não devem prometer funções ainda não implementadas.
5. Não introduzir servidor local, APIs remotas, Clerk, Supabase ou sincronização.
6. Não alterar textos, layout ou cores já aprovados da Home fora do necessário para navegação entre as telas.

## Arquitetura esperada

1. Separar a tela Canvas da Home em componentes React próprios, evitando concentrar toda a nova interface em `src/App.tsx`.
2. Extrair dados de demonstração do fluxo, propriedades e cards para estruturas tipadas, separadas da renderização.
3. Reutilizar tokens existentes de `src/App.css`; acrescentar somente estilos necessários e específicos do Canvas.
4. Manter a camada Rust inalterada nesta entrega, pois não haverá persistência do Canvas.
5. Preservar acessibilidade básica: botões reais, rótulos acessíveis, foco visível e controles decorativos fora da ordem de tabulação quando aplicável.

## Fora do escopo

1. Arrastar, soltar, redimensionar ou conectar nós.
2. Salvar, carregar ou versionar o estado do Canvas em SQLite.
3. Editor de texto e edição de propriedades.
4. Importação de assets, comentários, simulação, IA, login, sincronização ou colaboração.
5. Responsividade mobile. A referência e a primeira entrega são desktop.

## Critérios de aceite

1. A navegação Home e Canvas funciona na prévia Vite e no contexto Tauri quando disponível.
2. A tela Canvas reproduz hierarquia, painéis, fluxo visual, cores de conectores e densidade da referência sem copiar imagens rasterizadas dela.
3. Ícones não exibem caixas brancas no WebView do Tauri.
4. A Home continua visualmente inalterada, exceto pela navegação necessária.
5. `npm.cmd run build` conclui sem erros.
6. A documentação registra a implementação concluída e distingue claramente a interface demonstrativa da futura persistência e edição reais.

## Verificação antes de começar

Antes de implementar, conferir que `src/App.tsx`, `src/App.css`, `assets/icons/README.md` e `src-tauri/src/lib.rs` correspondem a este contexto. Caso a arquitetura tenha mudado, atualizar este prompt antes de escrever código.
