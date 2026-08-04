import { Show, type VoidComponent } from "solid-js";
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
  class?: string;
}

export const ResourceIcon: VoidComponent<ResourceIconProps> = (props) => {
  return (
    <Show when={props.src} fallback={<IconPlaceholder size={props.size} class={props.class} />}>
      {(src) => (
        <img
          src={src()}
          alt=""
          class={resourceIconStyle({ size: props.size, class: props.class })}
        />
      )}
    </Show>
  );
};
