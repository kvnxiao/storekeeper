import type React from "react";
import { tv, type VariantProps } from "tailwind-variants";

import { RESOURCE_ICONS } from "@/modules/resources/resources.icons";
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
  type: string;
  className?: string;
}

export const ResourceIcon: React.FC<ResourceIconProps> = ({
  type,
  size,
  className,
}) => {
  const iconPath = RESOURCE_ICONS[type];

  if (!iconPath) {
    return <IconPlaceholder size={size} className={className} />;
  }

  return (
    <img
      src={iconPath}
      alt=""
      className={resourceIconStyle({ size, className })}
    />
  );
};
