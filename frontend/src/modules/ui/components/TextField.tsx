import * as TextFieldPrimitive from "@kobalte/core/text-field";
import Eye from "lucide-solid/icons/eye";
import EyeOff from "lucide-solid/icons/eye-off";
import { createSignal, Show, type VoidComponent } from "solid-js";
import { tv } from "tailwind-variants";
import * as m from "@/paraglide/messages";

const fieldStyle = tv({
  base: "group flex flex-col gap-1 font-sans",
});

const labelStyle = tv({
  base: "text-sm font-medium text-zinc-950 dark:text-white",
});

const descriptionStyle = tv({
  base: "text-sm text-zinc-500 dark:text-zinc-400",
});

const inputStyle = tv({
  base: [
    "w-full rounded-lg bg-white px-3 py-1.5 text-sm",
    "text-zinc-950 placeholder:text-zinc-500 dark:bg-zinc-800/50 dark:text-white dark:placeholder:text-zinc-400",
    "shadow-sm ring-1 ring-zinc-950/10 dark:ring-white/10",
    "outline-none focus-visible:ring-2 focus-visible:ring-ring",
    "disabled:bg-zinc-100 disabled:text-zinc-400 disabled:ring-zinc-950/5",
    "dark:disabled:bg-zinc-900 dark:disabled:text-zinc-500",
    "data-[invalid]:ring-red-500",
    // Hide native password reveal (WebView2/Edge)
    "[&::-ms-reveal]:hidden",
  ],
  variants: {
    type: {
      text: "",
      password: "pr-9 font-mono",
    },
  },
  defaultVariants: {
    type: "text",
  },
});

export interface TextFieldProps {
  label?: string;
  description?: string;
  placeholder?: string;
  type?: "text" | "password";
  class?: string;
  value: string;
  onChange: (value: string) => void;
}

export const TextField: VoidComponent<TextFieldProps> = (props) => {
  const [revealed, setRevealed] = createSignal(false);
  const isPassword = () => props.type === "password";

  return (
    <TextFieldPrimitive.Root
      value={props.value}
      onChange={(value) => props.onChange(value)}
      class={fieldStyle({ class: props.class })}
    >
      <Show when={props.label}>
        <TextFieldPrimitive.Label class={labelStyle()}>{props.label}</TextFieldPrimitive.Label>
      </Show>
      <div class="relative">
        <TextFieldPrimitive.Input
          type={isPassword() && !revealed() ? "password" : "text"}
          placeholder={props.placeholder}
          class={inputStyle({ type: isPassword() ? "password" : "text" })}
        />
        <Show when={isPassword()}>
          <button
            type="button"
            aria-label={revealed() ? m.textfield_hide_password() : m.textfield_show_password()}
            onClick={() => setRevealed((v) => !v)}
            class="absolute inset-y-0 right-0 flex items-center pr-2.5 text-zinc-400 hover:text-zinc-600 dark:text-zinc-500 dark:hover:text-zinc-300"
          >
            <Show when={revealed()} fallback={<Eye aria-hidden="true" class="size-4" />}>
              <EyeOff aria-hidden="true" class="size-4" />
            </Show>
          </button>
        </Show>
      </div>
      <Show when={props.description}>
        <TextFieldPrimitive.Description class={descriptionStyle()}>
          {props.description}
        </TextFieldPrimitive.Description>
      </Show>
    </TextFieldPrimitive.Root>
  );
};
