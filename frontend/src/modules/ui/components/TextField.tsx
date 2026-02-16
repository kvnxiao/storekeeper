import { EyeIcon, EyeSlashIcon } from "@heroicons/react/20/solid";
import { useState } from "react";
import {
  Button as AriaButton,
  TextField as AriaTextField,
  type TextFieldProps as AriaTextFieldProps,
  composeRenderProps,
  Input,
  Label,
  Text,
} from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";
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
    "outline-none focus:ring-2 focus:ring-blue-500",
    "disabled:bg-zinc-100 disabled:text-zinc-400 disabled:ring-zinc-950/5",
    "dark:disabled:bg-zinc-900 dark:disabled:text-zinc-500",
    "invalid:ring-red-500",
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

type FieldStyleProps = VariantProps<typeof fieldStyle>;
type InputStyleProps = VariantProps<typeof inputStyle>;

export interface TextFieldProps
  extends Omit<AriaTextFieldProps, "type">,
    FieldStyleProps,
    InputStyleProps {
  label?: string;
  description?: string;
  placeholder?: string;
  className?: string;
}

export const TextField: React.FC<TextFieldProps> = ({
  label,
  description,
  placeholder,
  type = "text",
  className,
  ...props
}) => {
  const [revealed, setRevealed] = useState(false);
  const isPassword = type === "password";

  return (
    <AriaTextField
      {...props}
      className={composeRenderProps(className, (cn) =>
        fieldStyle({ className: cn }),
      )}
    >
      {label && <Label className={labelStyle()}>{label}</Label>}
      <div className="relative">
        <Input
          type={isPassword && revealed ? "text" : type}
          placeholder={placeholder}
          className={inputStyle({ type: type as "text" | "password" })}
        />
        {isPassword && (
          <AriaButton
            aria-label={
              revealed
                ? m.textfield_hide_password()
                : m.textfield_show_password()
            }
            onPress={() => setRevealed((v) => !v)}
            className="absolute inset-y-0 right-0 flex items-center pr-2.5 text-zinc-400 hover:text-zinc-600 dark:text-zinc-500 dark:hover:text-zinc-300"
          >
            {revealed ? (
              <EyeSlashIcon className="h-4 w-4" />
            ) : (
              <EyeIcon className="h-4 w-4" />
            )}
          </AriaButton>
        )}
      </div>
      {description && (
        <Text slot="description" className={descriptionStyle()}>
          {description}
        </Text>
      )}
    </AriaTextField>
  );
};
