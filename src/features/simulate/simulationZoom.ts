const minimumZoom = 0.45;
const maximumZoom = 2.5;
const zoomStep = 0.12;

function readBasePosition(element: HTMLElement, axis: "left" | "top") {
  const key = axis === "left" ? "simulationBaseLeft" : "simulationBaseTop";
  const stored = element.dataset[key];
  if (stored) return Number(stored);
  const value = Number.parseFloat(element.style[axis]);
  element.dataset[key] = String(value);
  return value;
}

function applyZoom(board: HTMLElement, zoom: number) {
  board.dataset.simulationZoom = String(zoom);
  const bounds = board.getBoundingClientRect();
  const centerX = bounds.width / 2;
  const centerY = bounds.height / 2;
  board.querySelectorAll<HTMLElement>(".simulation-node").forEach((node) => {
    const baseLeft = readBasePosition(node, "left");
    const baseTop = readBasePosition(node, "top");
    node.style.left = `${centerX + (baseLeft - centerX) * zoom}px`;
    node.style.top = `${centerY + (baseTop - centerY) * zoom}px`;
    node.style.transform = `translate(-50%, -50%) scale(${zoom})`;
  });
  const connections = board.querySelector<SVGElement>("svg");
  if (connections) {
    connections.style.transformOrigin = "center";
    connections.style.transform = `scale(${zoom})`;
  }
}

window.addEventListener("wheel", (event) => {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const board = target.closest<HTMLElement>(".simulation-board");
  if (!board) return;
  event.preventDefault();
  const current = Number(board.dataset.simulationZoom ?? "1");
  const next = Math.min(maximumZoom, Math.max(minimumZoom, current + (event.deltaY < 0 ? zoomStep : -zoomStep)));
  applyZoom(board, Number(next.toFixed(2)));
}, { passive: false });
