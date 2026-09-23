import { Port } from "./types";

export function validatePortConnection(source: Port, target: Port) {
  const directionValid = (source.direction === "output" || source.direction === "bidirectional") && (target.direction === "input" || target.direction === "bidirectional");
  const protocolMatch = source.protocol === target.protocol || source.protocol === "data" || target.protocol === "data";
  return { allowed: true, warning: !directionValid || !protocolMatch, explanation: !directionValid ? "A direção dos ports é incomum." : !protocolMatch ? "Os protocolos são diferentes." : undefined };
}
