import { useEffect, useState } from "react";
import { useProfileStore } from "./profileStore";

export function ProfileAvatar({ alt, fallback }: { alt: string; fallback: string }) {
  const avatarUrl = useProfileStore((state) => state.avatarUrl);
  const [readyUrl, setReadyUrl] = useState<string>();

  useEffect(() => {
    if (!avatarUrl) {
      setReadyUrl(undefined);
      return;
    }

    let active = true;
    const image = new Image();
    image.onload = () => {
      if (active) setReadyUrl(avatarUrl);
    };
    image.onerror = () => {
      if (active) setReadyUrl(undefined);
    };
    image.src = avatarUrl;
    return () => {
      active = false;
    };
  }, [avatarUrl]);

  if (!readyUrl) return <span>{fallback}</span>;

  return <img className="profile-avatar-image" src={readyUrl} alt={alt} onError={() => setReadyUrl(undefined)} />;
}
