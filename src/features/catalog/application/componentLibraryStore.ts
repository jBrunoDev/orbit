import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import { isTauriAvailable } from "../../../shared/tauri";

export type ComponentLibrary = { id: string; label: string; enabled: boolean };

type ComponentLibraryState = {
  libraries: ComponentLibrary[];
  loading: boolean;
  error?: string;
  load: () => Promise<void>;
  setAwsEnabled: (enabled: boolean) => Promise<void>;
};

export const useComponentLibraryStore = create<ComponentLibraryState>((set) => ({
  libraries: [],
  loading: true,
  load: async () => {
    if (!isTauriAvailable()) {
      set({ libraries: [], loading: false });
      return;
    }
    set({ loading: true, error: undefined });
    try {
      const libraries = await invoke<ComponentLibrary[]>("list_component_libraries");
      set({ libraries, loading: false });
    } catch (error) {
      set({ loading: false, error: error instanceof Error ? error.message : "Não foi possível carregar as bibliotecas." });
    }
  },
  setAwsEnabled: async (enabled) => {
    if (!isTauriAvailable()) return;
    const library = await invoke<ComponentLibrary>("set_component_library_enabled", { input: { libraryId: "aws", enabled } });
    set((state) => ({ libraries: state.libraries.map((item) => item.id === library.id ? library : item) }));
  },
}));
