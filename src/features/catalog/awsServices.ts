import type { CatalogItem } from "./types";

const assetRoot = "../../../assets/Icon-package_07312026.5846e92413caa21490223536cc97f1269e44fa92/Architecture-Service-Icons_07312026";

const serviceAssets = import.meta.glob<string>(
  "../../../assets/Icon-package_07312026.5846e92413caa21490223536cc97f1269e44fa92/Architecture-Service-Icons_07312026/{Arch_Artificial-Intelligence,Arch_Blockchain,Arch_Business-Applications,Arch_Cloud-Financial-Management,Arch_Compute,Arch_Containers,Arch_Databases,Arch_Developer-Tools,Arch_End-User-Computing,Arch_Front-End-Web-Mobile,Arch_Games,Arch_General-Icons}/64/*.svg",
  { eager: true, import: "default", query: "?url" },
);

function readable(value: string) {
  return value.replace(/^Arch_/, "").replace(/_64$/, "").replace(/-/g, " ").replace(/_/g, " ").replace(/\bAws\b/g, "AWS").replace(/\bApi\b/g, "API");
}

function itemFromAsset([filePath, assetPath]: [string, string]): CatalogItem {
  const segments = filePath.split("/");
  const folder = segments[segments.length - 3] ?? "Arch_General-Icons";
  const fileName = (segments[segments.length - 1] ?? "Arch_AWS-Service_64.svg").replace(".svg", "");
  const category = readable(folder);
  const label = readable(fileName);
  return {
    type: `aws:${folder}:${fileName}`.toLowerCase(),
    libraryId: "aws",
    label,
    category,
    icon: "service",
    color: "yellow",
    description: `AWS service: ${label}`,
    tags: ["AWS", category],
    assetPath,
    ports: [],
    simulationDefaults: {},
  };
}

export const awsServiceItems = Object.entries(serviceAssets)
  .map(itemFromAsset)
  .sort((left, right) => left.label.localeCompare(right.label));

export const awsPreviewItems = awsServiceItems.slice(0, 8);
export const awsServiceAssetRoot = assetRoot;
