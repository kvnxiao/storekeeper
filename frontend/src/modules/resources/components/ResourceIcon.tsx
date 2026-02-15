import type React from "react";
import { tv, type VariantProps } from "tailwind-variants";
import { IconPlaceholder } from "@/modules/ui/components/IconPlaceholder";

const resourceIconStyle = tv({
  base: "shrink-0 rounded object-contain",
  variants: {
    size: {
      sm: "size-5",
      md: "size-7",
    },
  },
  defaultVariants: {
    size: "md",
  },
});

type ResourceIconStyleProps = VariantProps<typeof resourceIconStyle>;

export interface ResourceIconProps extends ResourceIconStyleProps {
  src?: string;
  className?: string;
}

export const ResourceIcon: React.FC<ResourceIconProps> = ({
  src,
  size,
  className,
}) => {
  if (!src) {
    return <IconPlaceholder size={size} className={className} />;
  }

  return (
    <img src={src} alt="" className={resourceIconStyle({ size, className })} />
  );
};
