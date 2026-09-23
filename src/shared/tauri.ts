type TauriWindow = Window & {
  __TAURI_INTERNALS__?: { invoke?: unknown };
};

export function isTauriAvailable() {
  return (
    typeof (window as TauriWindow).__TAURI_INTERNALS__?.invoke === "function"
  );
}
