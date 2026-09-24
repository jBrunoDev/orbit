# 0005. Perfil local do Orbit

**Date**: 2026-09-24
**Status**: In Progress

## Summary

Esta decisão cria a área Profile como a central pessoal do Orbit no dispositivo atual. A página permite editar identidade, imagem, links e tecnologias, e apresenta uma leitura honesta dos Projects e conteúdos locais que já existem. Ela funciona sem conta e sem rede, portanto não cria ainda um perfil público nem uma camada de social.

## Context

O Orbit já inicializa um `local_profiles` e um `Personal Workspace`, mas esse perfil só possui identificador e datas. A referência em `Design/orbitProfilePage.png` descreve uma página de identidade e resumo de produção. Hoje a sidebar mostra nome e avatar fictícios que não têm persistência ou rota.

O valor funcional da página não é repetir totais estáticos. Ela precisa dar ao usuário um ponto confiável para identificar seu espaço local, voltar aos seus artefatos recentes e ajustar os dados que o aplicativo mostra. Projects ficam no catálogo global `orbit.db`, enquanto Notes, Docs, Canvases, revisões e simulações vivem em bancos separados de cada Project. Por isso a leitura agregada pertence ao lado Rust e não a um contador inventado no React.

## Requirements

**User stories**:

1. Como pessoa que usa Orbit localmente, quero definir como meu perfil aparece no aplicativo, para reconhecer meu espaço sem precisar criar uma conta.
2. Como pessoa que trabalha em vários Projects, quero abrir itens recentes e entender o que produzi, para retomar o trabalho com contexto.

**Acceptance criteria**:

1. **AC-1**: A rota `#profile` abre dentro de `AppShell`, preserva a sidebar compartilhada, usa a referência fornecida como fonte visual e destaca Profile na entrada de usuário da sidebar.
2. **AC-2**: O usuário pode editar nome, identificador local opcional, biografia, localização, website, GitHub, avatar, cover e tecnologias. Uma gravação bem sucedida continua disponível depois de reiniciar o aplicativo.
3. **AC-3**: Avatar e cover aceitam somente PNG, JPEG, WebP ou GIF não animado, validam conteúdo e tamanho antes de persistir e são copiados para o diretório de dados do Orbit. O cancelamento, um arquivo inválido ou uma falha de cópia preservam o perfil anterior.
4. **AC-4**: Overview mostra totais calculados de Projects ativos, Notes ativas, Templates favoritos e simulações salvas, mais os cinco Projects e atividades mais recentes disponíveis. O card Templates representa somente favoritos do usuário, mesmo que a referência use o rótulo genérico `Templates`. Totais nunca ficam gravados como cache autoritativo.
5. **AC-5**: Projects, Notes, Templates, Components, Simulations, Docs e Activity são abas funcionais. Cada lista tem estado vazio, carregando e erro, paginação por cursor e uma ação que navega ao recurso existente ou à área correta do Orbit.
6. **AC-6**: A atividade descreve somente eventos que existem em revisões persistidas ou em registros equivalentes. O produto não inventa atividade para completar a interface. Escritas de Notes e Docs passam a registrar revisão para que apareçam na atividade futura.
7. **AC-7**: A página continua utilizável sem rede, login, Clerk ou Supabase. Dados de identidade e imagens não são enviados para fora do dispositivo nesta entrega.
8. **AC-8**: Campos e interações possuem labels, foco visível, teclado para tabs e diálogo, texto alternativo para imagens e informação textual para estados e erros, sem depender somente de cor.

## Options considered

### Option 1: Tela somente visual com dados de exemplo

Esta opção reproduz a referência rapidamente no frontend.

**Pros**:

1. Menos código inicial.

**Cons**:

1. Nome, contadores e atividade deixam de representar o trabalho local.
2. A página se torna dívida de interface, pois todo comportamento real precisará ser refeito.

### Option 2: Perfil local persistido com leitura agregada sob demanda

Esta opção mantém a identidade no catálogo local e calcula o resumo a partir do catálogo e dos bancos dos Projects.

**Pros**:

1. Respeita o modelo local primeiro já aprovado.
2. Evita contador desatualizado e não introduz uma conta como pré requisito.
3. Reutiliza dados e rotas já existentes.

**Cons**:

1. A leitura precisa abrir mais de um banco quando há muitos Projects.
2. Algumas escritas existentes precisam registrar revisões para compor Activity.

### Option 3: Perfil cloud e social desde o início

Esta opção vincula o perfil ao login e à sincronização em nuvem antes de liberar a página.

**Pros**:

1. Deixa uma base pronta para identidade compartilhada entre dispositivos.

**Cons**:

1. Viola o uso local sem conta.
2. Exige decisões ainda não aprovadas sobre consentimento, merge, RLS, privacidade e conflitos.

## Decision

**Chosen option**: Option 2: Perfil local persistido com leitura agregada sob demanda.

O primeiro corte será um perfil exclusivamente local, com uma página completa e navegável que deriva seu resumo dos dados reais do dispositivo.

## Rationale

A opção escolhida resolve o trabalho imediato da página sem antecipar a arquitetura de Cloud Profile. `local_profiles` já é a identidade dona de bibliotecas e favoritos. Estender essa entidade evita criar uma segunda identidade e mantém a aplicação coerente quando não há conexão.

O backend vai agregar os dados porque é o único lugar que conhece os caminhos válidos de todos os `project.db`. A interface recebe uma projeção pronta para exibir e nunca recebe um caminho absoluto de arquivo. Isso preserva o limite entre UI, domínio e infraestrutura estabelecido pelo Orbit.

## Feature design

**Fonte de design**: `Design/orbitProfilePage.png`. A composição aprovada é cover, identidade, ações de edição, tags, abas, cards de totais, listas de recentes e tecnologias. `AppShell`, `AppSidebar`, tokens e primitives existentes permanecem a fonte para a estrutura compartilhada.

**Controles globais da referência**: a busca, o seletor de tema e as notificações mostrados no topo pertencem ao shell global. Esta entrega não os recria dentro de Profile. Ela apenas os reutiliza quando já existirem no `AppShell` compartilhado.

**Escopo deste corte**:

1. Inclui perfil local, importação local de imagem, links externos opcionais, tecnologias, resumo e navegação aos artefatos existentes.
2. Não inclui login, publicação, seguidores, pesquisa de pessoas, sincronização, upload em nuvem, notificações ou edição de dados de outro usuário.
3. A indicação de estado local substitui qualquer ponto online visto na referência. Um vínculo de conta futura poderá mostrar estado sincronizado quando essa arquitetura for aprovada.

**Data model sketch**:

| Entidade | Persistência | Campos e regras |
|---|---|---|
| `local_profiles` | `orbit.db`, existente | Mantém `id`, `created_at` e `updated_at`. Continua sendo a identidade local canônica. |
| `profile_details` | `orbit.db` | `profile_id` chave primária e chave externa para `local_profiles`, `display_name` obrigatório com 1 a 80 caracteres, `local_handle` opcional com 3 a 32 caracteres e índice único sem distinção de maiúsculas, `bio` até 280 caracteres, `location` até 80, `website_url` e `github_url` opcionais, `avatar_asset_id` e `cover_asset_id` opcionais, datas. A migration cria valores vazios e nome inicial `Orbit User` para perfis existentes. |
| `profile_assets` | `orbit.db` e filesystem | `id` UUID v7, `profile_id` chave externa, `kind` avatar ou cover, `stored_path` relativo e único, `media_type`, `byte_size`, `created_at`, `deleted_at`. Arquivo vive em `profiles/<profile id>/assets/`. Avatar limita 10 MB e cover 15 MB. |
| `profile_technologies` | `orbit.db` | `profile_id`, `slug` normalizado, `label`, `sort_order`, `created_at`. Chave primária composta por perfil e slug. Máximo de 20 itens por perfil, cada label com 1 a 32 caracteres. |
| `projects`, `template_favorites`, bibliotecas e bancos de Project | existentes | São fontes de leitura. Não recebem coluna de contador ou cópia de atividade. |
| `revisions`, `notes`, `docs`, `simulation_runs` | existentes em `project.db` | São fontes de Activity e das abas. Mutations de Notes e Docs passam a acrescentar `revisions` na mesma transação da escrita. |

**State transitions**:

```text
perfil inicial
→ rascunho de edição
→ validação local
→ perfil salvo

perfil salvo
→ imagem escolhida
→ arquivo validado e copiado
→ perfil salvo com novo asset

perfil salvo
→ imagem inválida, cancelada ou cópia falhou
→ perfil salvo anterior
```

Trocar imagem marca o asset anterior com `deleted_at` somente depois da nova cópia e da transação de perfil serem confirmadas. A remoção física fica para uma limpeza futura segura, nunca para o caminho de falha.

**API surface**:

| Command Tauri | Entradas principais | Saídas principais | Autorização | Erros principais |
|---|---|---|---|---|
| `get_local_profile` | nenhuma | detalhes, URLs seguras, tecnologias e estado local | perfil local atual | catálogo indisponível |
| `update_local_profile` | nome, handle opcional, bio, localização, URLs, tecnologias | perfil salvo | perfil local atual | campo inválido, handle em uso, limite excedido |
| `replace_profile_asset` | tipo, nome original, bytes | asset e perfil atualizado | perfil local atual | formato inválido, bytes acima do limite, cópia falhou |
| `remove_profile_asset` | tipo | perfil sem o asset | perfil local atual | tipo inválido |
| `read_profile_asset` | asset id | mídia e bytes | asset do perfil local atual | asset ausente ou removido |
| `get_profile_overview` | nenhuma | totais, Projects recentes, Activity recente, tecnologias | perfil local atual | banco de Project indisponível com aviso explícito |
| `list_profile_resources` | aba, cursor opcional, limite de 20 | itens, próximo cursor, Projects indisponíveis | perfil local atual | aba inválida, cursor inválido |

`replace_profile_asset` aceita somente bytes cuja assinatura seja PNG, JPEG, WebP ou GIF de quadro único. O tipo informado pelo frontend não é confiável. A implementação adiciona validação local de bytes e dimensões antes de gravar, sem depender de serviço externo.

**Value sourcing**:

| Ação | Valor produzido ou exibido | Fonte |
|---|---|---|
| Cabeçalho | nome, handle, bio, localização, links, avatar e cover | `profile_details` e `profile_assets` do perfil local |
| Tecnologias | rótulos e ordem | `profile_technologies` ordenada por `sort_order` |
| Cards de resumo | Projects, Notes, Templates e simulações | consultas atuais sobre `projects`, `notes`, `template_favorites` e `simulation_runs` sem persistir total |
| Projects recentes | nome, modo, data e destino | `projects` ativos ordenados por `updated_at` |
| Activity | verbo, recurso, Project e data | `revisions`, complementado por `simulation_runs` e novas revisões de Notes e Docs |
| Aba Templates | template e favorito | manifesto oficial e `template_favorites` |
| Aba Components | bibliotecas, Components pessoais e definições | tabelas de biblioteca do catálogo |
| Aba Docs, Notes e Simulations | título ou cenário, Project, data e destino | bancos `project.db` pertencentes ao perfil |

**Key invariants**:

1. Há somente um perfil local ativo por instalação nesta fase, resolvido por `local_profiles` existente.
2. Perfil, assets e tecnologias nunca são sincronizados, enviados ou expostos publicamente neste corte.
3. Um caminho de asset retornado para a UI é relativo e associado ao perfil atual. O frontend não escolhe caminho de destino.
4. Um Project na lixeira, uma Note arquivada ou um recurso com `deleted_at` não entra em totais nem listas regulares.
5. Os totais são derivados em cada leitura. Nenhuma mutation atualiza contadores gravados.
6. Um evento de Activity precisa ter um registro fonte e link para o Project certo. Se o Project estiver ausente ou ilegível, a UI informa a quantidade indisponível em vez de fabricar resultado.
7. Uma URL só é aberta se for `https` ou, para GitHub, `https://github.com/`. Links inválidos são recusados antes de salvar.

**Security model**:

Somente o processo local acessa o perfil. A UI chama comandos Tauri e não lê o filesystem diretamente. Imagens e texto do perfil são dados pessoais locais, não credenciais, e não devem entrar em `project.db`, revisões de Projects, logs remotos ou prompts de IA. Nenhum token, email, sessão ou segredo é exibido ou persistido nesta página.

**Configuration required**:

Não há variável de ambiente, segredo ou conta externa nova.

**UX e comportamento por área**:

| Área | Comportamento funcional |
|---|---|
| Entrada de usuário na sidebar | Clique em nome ou avatar abre `#profile`. O botão de settings preserva a rota de Settings quando ela existir, sem confundir Profile com configurações de IA. |
| Cabeçalho | Mostra cover ou gradiente padrão, avatar ou monograma e um chip textual `Perfil local` ao lado do identificador. O chip substitui o ponto online da referência e não comunica presença em rede. Também mostra dados disponíveis e botões Edit cover e Edit profile. Website e GitHub usam `tauri-plugin-opener` após validação. |
| Diálogo Edit profile | Abre com cópia do perfil salvo. Salvar valida todos os campos e atualiza em uma transação. Cancelar, Escape e fechar descartam somente o rascunho. O campo de tecnologias adiciona, remove e reordena antes de salvar. |
| Edit cover e avatar | Abrem seletor local. Há pré visualização apenas em memória e confirmação explícita antes da cópia. Ação Remove volta ao gradiente ou monograma padrão. |
| Overview | Mostra os quatro cards da referência, Projects recentes, Activity recente e tecnologias. Cada item navegável declara o destino em texto acessível. |
| Abas | Projects abre a lista de Projects ativos. Notes e Docs abrem recurso no Project correspondente. Templates abre o detalhe no catálogo. Components abre Components. Simulations abre Simulate com o Project e run selecionados quando a rota receber foco. Activity filtra a mesma projeção cronológica. |
| Estados | Sem conteúdo mostra uma explicação e ação coerente, como criar Project ou abrir Templates. Carregando usa skeleton. Erro preserva o último resultado útil e oferece tentar novamente. |

**Performance e consistência**:

1. Overview lê no máximo cinco itens recentes por categoria e pode ser cancelado ao sair da rota.
2. Listas usam cursor composto por data e id, limite fixo de 20 e consulta somente Projects ativos do workspace local.
3. A agregação abre bancos em modo leitura. Um banco que não puder ser aberto não derruba toda a página, mas produz aviso com contagem de Projects indisponíveis.
4. Para a primeira entrega não há cache persistido. Se a medição mostrar lentidão com muitos Projects, uma projeção invalidada por revision será uma decisão posterior.

**Critical test scenarios**:

1. Happy path: editar dados, tecnologias, avatar e cover, reiniciar o Orbit e conferir os mesmos dados, verifica **AC-2** e **AC-3**.
2. Failure case: escolher imagem acima do limite ou com assinatura inválida e confirmar que a imagem anterior continua visível, verifica **AC-3**.
3. Agregação: criar Project, Note, Doc, favorito e simulação em mais de um Project, reabrir Profile e conferir totais, abas, cursores e destinos, verifica **AC-4**, **AC-5** e **AC-6**.
4. Offline: executar a página com rede desligada e sem sessão, editar o perfil e navegar pelos recursos locais, verifica **AC-1** e **AC-7**.
5. Acessibilidade: usar somente teclado para abrir Profile, trocar aba, editar, cancelar, salvar e identificar uma mensagem de erro, verifica **AC-8**.

## Build plan

1. Criar a migration seguinte do catálogo para detalhes, assets e tecnologias do perfil, com valores iniciais seguros para perfis existentes, satisfaz **AC-2**, **AC-3** e **AC-7**.
2. Implementar gateway Rust de leitura, validação, atualização transacional e importação segura de assets, incluindo leitura sem expor caminhos locais, satisfaz **AC-2**, **AC-3** e **AC-7**.
3. Estender as mutations de Notes e Docs para gravar revision na mesma transação e criar a projeção paginada que agrega catálogo e bancos de Project, satisfaz **AC-4**, **AC-5** e **AC-6**.
4. Criar tipos, gateway e store de Profile no frontend, com estados de carregamento, erro, cursor e invalidação após salvar, satisfaz **AC-2**, **AC-4** e **AC-5**.
5. Adicionar rota `#profile`, clique no perfil da sidebar e `ProfilePage` composta com `AppShell`, sem alterar a infraestrutura visual compartilhada, satisfaz **AC-1** e **AC-8**.
6. Construir cabeçalho, diálogo de edição, seleção de imagens, tabs e listas a partir de `Design/orbitProfilePage.png`, usando tokens e componentes existentes, satisfaz **AC-1**, **AC-2**, **AC-3**, **AC-5** e **AC-8**.
7. Escrever testes Rust para migration, validação, atomicidade e agregação, mais testes frontend para rotas, estados e teclado. Depois validar no shell Tauri sem rede, satisfaz **AC-1** até **AC-8**.

## Consequences

**Positive**:

1. A interface passa a representar o trabalho local real, não uma pessoa fictícia.
2. O perfil ganha utilidade imediata sem ampliar a superfície de nuvem ou autenticação.
3. A atividade incentiva uma camada de revisões coerente entre recursos.

**Negative / tradeoffs**:

1. A leitura de Profile precisa visitar cada banco de Project e requer paginação e estados explícitos.
2. Esta fase deliberadamente não entrega a mesma identidade em outro computador.
3. Validação de imagem adiciona dependência Rust para decodificar formatos permitidos ou uma implementação equivalente segura.

**Neutral**:

1. A futura ligação com Cloud Profile poderá adicionar identificador externo e estado de sincronização sem trocar o dono local dos dados.
2. A referência visual usa conteúdo de exemplo. O produto deve mostrar defaults neutros até que o usuário informe seus próprios dados.

## Follow-up

1. Planejar vínculo opcional entre Local Profile e Cloud Profile somente depois de existir login desktop, consentimento e resolução de conflito aprovados.
2. Medir a agregação com muitos Projects antes de decidir por índice ou cache local.
3. Planejar rotas de foco para uma simulation run específica, caso Simulate ainda não exponha esse destino.
