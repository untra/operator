export interface BrandIconProps {
  src: string;
  className?: string;
  label?: string;
}

export function BrandIcon({ src, className, label }: BrandIconProps) {
  return (
    <img
      className={className}
      src={src}
      width={16}
      height={16}
      alt={label ?? ""}
      aria-hidden={label ? undefined : "true"}
    />
  );
}
