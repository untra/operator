// Renders a vendor brand logo as a 16px inline image. `name` is a brand
// basename (e.g. "ollama", "openrouter") served from `ui/public/icons/`, set by
// the backend `ModelServerKind::brand_icon()` and carried on `SectionRowDto`'s
// `brand_icon` field. Unlike `ConceptIcon` (a recolorable codicon font glyph),
// brand logos are full-color SVGs, so they don't inherit `currentColor`.

import { BrandIcon as SharedBrandIcon } from "@operator/webcomponents";

interface BrandIconProps {
  name: string;
  className?: string;
}

export function BrandIcon({ name, className }: BrandIconProps) {
  return <SharedBrandIcon className={className} src={`/icons/${name}.svg`} />;
}
