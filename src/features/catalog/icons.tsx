import type { ReactNode } from "react";

export const catalogIconFallback = "service";

type CatalogIconProps = {
  icon?: string;
  className?: string;
};

const sharedStroke = {
  fill: "none",
  stroke: "currentColor",
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  strokeWidth: 2.1,
};

function iconPaths(icon: string): ReactNode {
  switch (icon) {
    case "client":
      return <><rect {...sharedStroke} x="14" y="5" width="20" height="38" rx="4" /><path {...sharedStroke} d="M20 10h8M22 37h4" /><circle {...sharedStroke} cx="24" cy="40" r=".6" /></>;
    case "api":
      return <><rect {...sharedStroke} x="7" y="9" width="34" height="12" rx="3" /><rect {...sharedStroke} x="7" y="27" width="34" height="12" rx="3" /><path {...sharedStroke} d="M16 15h16M16 33h16" /><circle {...sharedStroke} cx="12" cy="15" r="1" /><circle {...sharedStroke} cx="12" cy="33" r="1" /></>;
    case "worker":
      return <><rect {...sharedStroke} x="10" y="8" width="28" height="32" rx="5" /><path {...sharedStroke} d="m26 13-7 12h6l-3 10 8-14h-6l2-8Z" /></>;
    case "loadBalancer":
      return <><circle {...sharedStroke} cx="10" cy="24" r="3" /><circle {...sharedStroke} cx="38" cy="12" r="3" /><circle {...sharedStroke} cx="38" cy="36" r="3" /><path {...sharedStroke} d="M13 24h9m0 0 13-12m-13 12 13 12" /></>;
    case "cdn":
      return <><circle {...sharedStroke} cx="24" cy="24" r="17" /><path {...sharedStroke} d="M7 24h34M24 7c5 5 7 11 7 17s-2 12-7 17M24 7c-5 5-7 11-7 17s2 12 7 17" /><path {...sharedStroke} d="M10 15h28M10 33h28" /></>;
    case "database":
      return <><ellipse {...sharedStroke} cx="24" cy="11" rx="15" ry="6" /><path {...sharedStroke} d="M9 11v22c0 3 7 6 15 6s15-3 15-6V11" /><path {...sharedStroke} d="M9 22c0 3 7 6 15 6s15-3 15-6" /></>;
    case "cache":
      return <><path {...sharedStroke} d="m24 7 16 8-16 8L8 15l16-8Z" /><path {...sharedStroke} d="m8 23 16 8 16-8M8 31l16 8 16-8" /></>;
    case "queue":
      return <><rect {...sharedStroke} x="7" y="11" width="34" height="26" rx="4" /><path {...sharedStroke} d="M17 11v26M29 11v26M10 17h4m8 0h4m8 0h4" /></>;
    case "python":
      return <><path {...sharedStroke} d="M24 7h7c4 0 6 3 6 7v6H21c-4 0-7-3-7-7 0-3 3-6 10-6Z" /><path {...sharedStroke} d="M24 41h-7c-4 0-6-3-6-7v-6h16c4 0 7 3 7 7 0 3-3 6-10 6Z" /><circle fill="currentColor" cx="30" cy="12" r="1.5" /><circle fill="currentColor" cx="18" cy="36" r="1.5" /></>;
    case "typescript":
      return <><rect {...sharedStroke} x="7" y="7" width="34" height="34" rx="4" /><path {...sharedStroke} d="M15 17h13M21.5 17v14M28 23c1.5-1.4 6-1.1 6 1.5 0 3.8-7 1.9-7 5.5 0 2.5 3.9 2.5 6.5.8" /></>;
    case "storage":
      return <><path {...sharedStroke} d="m24 6 16 9v18l-16 9-16-9V15l16-9Z" /><path {...sharedStroke} d="m8 15 16 9 16-9M24 24v18" /></>;
    case "note":
      return <><path {...sharedStroke} d="M12 6h18l6 6v30H12z" /><path {...sharedStroke} d="M30 6v8h8M17 22h14M17 29h14M17 36h9" /></>;
    case "service":
    default:
      return <><rect {...sharedStroke} x="8" y="11" width="32" height="26" rx="5" /><path {...sharedStroke} d="M15 18h18M15 24h11M15 30h15" /><circle fill="currentColor" cx="34" cy="24" r="1.5" /></>;
  }
}

export function CatalogIcon({ icon = catalogIconFallback, className }: CatalogIconProps) {
  return <svg aria-hidden="true" className={["canvas-catalog-icon", className].filter(Boolean).join(" ")} viewBox="0 0 48 48" focusable="false">
    {iconPaths(icon)}
  </svg>;
}
