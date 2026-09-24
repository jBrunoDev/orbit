export type SimulationComponent = { id: string; componentType: string; label: string; description: string; x: number; y: number; color: string };
export type SimulationConnection = { id: string; sourceComponentId: string; targetComponentId: string };
export type SimulationSnapshot = { canvas: { id: string; name: string }; components: SimulationComponent[]; connections: SimulationConnection[] };
export type SimulationScenario = "normal" | "high-load" | "slow-database" | "service-failure";
export type SimulationEvent = { id: string; at: string; level: "info" | "warn" | "error"; message: string; componentId?: string };
export type SimulationRun = { events: SimulationEvent[]; visited: string[]; metrics: { requests: number; successRate: number; latency: number; errors: number; throughput: number }; issues: string[] };

const latencies: Record<string, number> = { client: 2, "load-balancer": 12, "api-server": 32, database: 45, cache: 3 };

function random(seed: number) { let value = seed >>> 0; return () => { value = (value * 1664525 + 1013904223) >>> 0; return value / 4294967296; }; }
function componentLatency(component: SimulationComponent, scenario: SimulationScenario, next: () => number) { const base = latencies[component.componentType] ?? 24; return Math.round(base + next() * 8 + (scenario === "slow-database" && component.componentType === "database" ? 320 : 0)); }
function timeLabel(index: number) { return `00:00.${String(index * 137).padStart(3, "0")}`; }

export function simulate(snapshot: SimulationSnapshot, startId: string, scenario: SimulationScenario, rps: number, duration: number, failures: boolean, seed: number): SimulationRun {
  const bySource = new Map<string, SimulationConnection[]>();
  snapshot.connections.forEach((connection) => bySource.set(connection.sourceComponentId, [...(bySource.get(connection.sourceComponentId) ?? []), connection]));
  const components = new Map(snapshot.components.map((component) => [component.id, component]));
  const next = random(seed); const events: SimulationEvent[] = []; const visited: string[] = []; const queue = [startId]; const seen = new Set<string>(); let totalLatency = 0; let errors = 0;
  events.push({ id: "started", at: timeLabel(0), level: "info", message: `Simulação iniciada com seed ${seed}.` });
  while (queue.length && events.length < 24) {
    const id = queue.shift()!; if (seen.has(id)) continue; seen.add(id);
    const component = components.get(id); if (!component) continue; visited.push(id);
    const latency = componentLatency(component, scenario, next); totalLatency += latency;
    const fails = failures && (scenario === "service-failure" || next() < 0.08) && (component.componentType === "cache" || component.componentType === "api-server");
    if (fails) { errors += 1; events.push({ id: `${id}-failure`, at: timeLabel(events.length), level: "error", componentId: id, message: `${component.label} não respondeu. A rota foi marcada como falha.` }); }
    else { events.push({ id: `${id}-request`, at: timeLabel(events.length), level: latency > 180 ? "warn" : "info", componentId: id, message: `${component.label} processou a requisição em ${latency} ms.` }); }
    if (latency > 180) events.push({ id: `${id}-slow`, at: timeLabel(events.length), level: "warn", componentId: id, message: `${component.label} ultrapassou o limite de latência esperado.` });
    (bySource.get(id) ?? []).forEach((connection) => queue.push(connection.targetComponentId));
  }
  const requests = Math.max(1, Math.round(rps * duration)); const errorRate = errors ? Math.min(35, errors * 7 + (scenario === "high-load" ? 4 : 0)) : 0; const latency = Math.max(1, Math.round(totalLatency / Math.max(1, visited.length)));
  events.push({ id: "complete", at: timeLabel(events.length), level: errors ? "warn" : "info", message: `Execução concluída. ${requests.toLocaleString("pt-BR")} requisições processadas.` });
  return { events, visited, metrics: { requests, successRate: 100 - errorRate, latency, errors: Math.round(requests * errorRate / 100), throughput: rps }, issues: [ ...(snapshot.connections.length ? [] : ["O Canvas não possui conexões. A simulação executou apenas o nó inicial."]), ...(errors ? [`${errors} serviço(s) apresentaram falha no cenário atual.`] : []), ...(latency > 180 ? ["A latência média excedeu o limite observado para este cenário."] : []) ] };
}
