# Simulation Engine

**Status**: Assumed

## Summary

Esta entrega transforma o Canvas atual em uma execução local e reproduzível. Ela usa os Components, Connections, cenário, seed e configurações para gerar eventos, métricas, estados de saúde e problemas visíveis. O design de `Design/orbitSimulatePage.png` orienta a composição sem substituir o App Shell compartilhado.

## Requirements

- AC-1 A rota Simulate abre pelo menu compartilhado e permite escolher um Project local.
- AC-2 A execução lê o Canvas e percorre as Connections a partir do primeiro Component disponível.
- AC-3 Run, Pause, Step e Reset alteram o estado real da execução.
- AC-4 Cenários, requests por segundo, duração, seed e falhas modificam os eventos e métricas.
- AC-5 Logs, métricas, saúde e problemas mostram dados derivados da execução.

## Assumption built on

O Canvas é a fonte dos Components e Connections. O motor determinístico é executado localmente na interface nesta primeira fatia e usa valores padrão por tipo de Component. O histórico persistido de Simulation Runs será a próxima fatia, pois exige uma migration e comandos próprios para retenção e comparação.

## Feature design

| Saída | Fonte |
|---|---|
| Componentes e conexões | `load_canvas` do Project local |
| Ordem dos eventos | percurso dirigido das Connections |
| Latência | tipo do Component, cenário e gerador por seed |
| Requests e throughput | requests por segundo e duração lógica |
| Falhas e problemas | cenário, opção de falhas e percurso executado |

## Build plan

- [x] Criar rota e superfície Simulate, satisfaz AC-1.
- [x] Implementar motor determinístico e controles de execução, satisfaz AC-2 até AC-4.
- [x] Exibir logs, métricas, saúde e problemas derivados, satisfaz AC-5.
- [x] Validar build e fluxo visual local.

## Consequences

A simulação não finge dados estáticos, mas ainda não salva histórico entre sessões. A seed permite repetir a mesma execução enquanto Canvas e configurações não mudarem.
