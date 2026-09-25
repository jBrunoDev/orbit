import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./ModuleAssetImage.css";

type Asset = { mediaType: string; bytes: number[] };

export function ModuleAssetImage({ module, assetId, alt }: { module: "docs" | "notes"; assetId: string; alt: string }) {
  const [src, setSrc] = useState<string>();
  const [unavailable, setUnavailable] = useState(false);

  useEffect(() => {
    let active = true;
    setSrc(undefined);
    setUnavailable(false);
    void invoke<Asset>(module === "docs" ? "read_doc_asset" : "read_note_asset", { input: { assetId } })
      .then((asset) => {
        if (!active) return;
        let binary = "";
        asset.bytes.forEach((byte) => { binary += String.fromCharCode(byte); });
        setSrc(`data:${asset.mediaType};base64,${btoa(binary)}`);
      })
      .catch(() => { if (active) setUnavailable(true); });
    return () => { active = false; };
  }, [assetId, module]);

  if (unavailable) return <span className="module-asset-unavailable">Imagem indisponível</span>;
  if (!src) return <span className="module-asset-loading">Carregando imagem...</span>;
  return <img className="module-asset-image" src={src} alt={alt} />;
}
