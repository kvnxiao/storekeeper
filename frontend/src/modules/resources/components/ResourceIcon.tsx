import type { VoidComponent } from "solid-js";

export interface ResourceIconProps {
  src: string;
}

export const ResourceIcon: VoidComponent<ResourceIconProps> = (props) => (
  <img src={props.src} alt="" class="size-7 shrink-0 rounded object-contain" />
);
