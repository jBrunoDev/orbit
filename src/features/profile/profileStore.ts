import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import { isTauriAvailable } from "../../shared/tauri";

export type LocalProfile = {
  id: string;
  displayName: string;
  localHandle?: string;
  bio: string;
  location: string;
  websiteUrl?: string;
  githubUrl?: string;
  avatarAssetId?: string;
  coverAssetId?: string;
  technologies: string[];
  updatedAt: number;
};

type ProfileAsset = { mediaType: string; bytes: number[] };

type ProfileState = {
  profile?: LocalProfile;
  avatarUrl?: string;
  coverUrl?: string;
  loading: boolean;
  load: () => Promise<void>;
  applyProfile: (profile: LocalProfile) => void;
};

let imageRequest = 0;

async function readAssetUrl(assetId?: string) {
  if (!assetId) return undefined;
  try {
    const asset = await invoke<ProfileAsset>("read_profile_asset", { assetId });
    return URL.createObjectURL(new Blob([new Uint8Array(asset.bytes)], { type: asset.mediaType }));
  } catch {
    return undefined;
  }
}

function revokeAssetUrl(url?: string) {
  if (url) URL.revokeObjectURL(url);
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  loading: false,
  load: async () => {
    if (!isTauriAvailable() || get().loading) return;
    set({ loading: true });
    try {
      get().applyProfile(await invoke<LocalProfile>("get_local_profile"));
    } catch {
      // The shared shell keeps the existing initials when the local profile is unavailable.
    } finally {
      set({ loading: false });
    }
  },
  applyProfile: (profile) => {
    const current = get();
    const avatarChanged = current.profile?.avatarAssetId !== profile.avatarAssetId;
    const coverChanged = current.profile?.coverAssetId !== profile.coverAssetId;
    const previousAvatarUrl = current.avatarUrl;
    const previousCoverUrl = current.coverUrl;
    if (!avatarChanged && !coverChanged) {
      set({ profile });
      return;
    }
    const request = ++imageRequest;

    set({
      profile,
      avatarUrl: avatarChanged ? undefined : current.avatarUrl,
      coverUrl: coverChanged ? undefined : current.coverUrl,
    });

    void Promise.all([
      avatarChanged ? readAssetUrl(profile.avatarAssetId) : Promise.resolve(current.avatarUrl),
      coverChanged ? readAssetUrl(profile.coverAssetId) : Promise.resolve(current.coverUrl),
    ]).then(([avatarUrl, coverUrl]) => {
      if (request !== imageRequest) {
        if (avatarChanged) revokeAssetUrl(avatarUrl);
        if (coverChanged) revokeAssetUrl(coverUrl);
        return;
      }
      if (avatarChanged) revokeAssetUrl(previousAvatarUrl);
      if (coverChanged) revokeAssetUrl(previousCoverUrl);
      set({ avatarUrl, coverUrl });
    });
  },
}));
