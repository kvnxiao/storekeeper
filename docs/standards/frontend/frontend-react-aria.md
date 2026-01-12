# React Aria Integration

All interactive components wrap [React Aria Components](https://react-spectrum.adobe.com/react-aria/) with styled variants using `tailwind-variants`.

## Component Pattern

```tsx
import { Button as AriaButton, type ButtonProps as AriaButtonProps } from "react-aria-components";
import { tv, type VariantProps } from "tailwind-variants";

const buttonStyle = tv({
  base: ["px-4 py-2 rounded-lg font-semibold"],
  variants: {
    variant: { solid: "bg-primary text-white", outline: "border border-input" },
    size: { sm: "text-xs px-2", md: "text-sm px-3" },
  },
  defaultVariants: { variant: "solid", size: "md" },
});

type ButtonStyleProps = VariantProps<typeof buttonStyle>;

export interface ButtonProps extends AriaButtonProps, ButtonStyleProps {
  className?: string;
}

export const Button: React.FC<ButtonProps> = ({ variant, size, className, ...props }) => (
  <AriaButton
    {...props}
    className={composeRenderProps(className, (cln) => buttonStyle({ variant, size, className: cln }))}
  />
);
```

## Props Interface Pattern

Always extend React Aria props with style variants:

```tsx
// Bad: Loose typing
interface ButtonProps {
  onClick?: () => void;
  children: React.ReactNode;
}

// Good: Composed props
export interface ButtonProps extends AriaButtonProps, ButtonStyleProps {
  className?: string;
}
```

## composeRenderProps

Use `composeRenderProps` from `react-aria-components` to merge classNames while preserving render prop access:

```tsx
// Bad: Losing render state access
<AriaSwitch className={switchStyle()}>
  {children}
</AriaSwitch>

// Good: Access render props for state-dependent styling
<AriaSwitch className={composeRenderProps(className, (cn) => style({ className: cn }))}>
  {(renderProps) => (
    <>
      <div className={trackStyle(renderProps)}>
        <span className={handleStyle(renderProps)} />
      </div>
      {children}
    </>
  )}
</AriaSwitch>
```

## Router-Integrated Links

Use `createLink` from TanStack Router to create router-aware components:

```tsx
import { createLink, type LinkComponent } from "@tanstack/react-router";
import { Link as RACLink, type LinkProps as RACLinkProps } from "react-aria-components";
import { buttonStyle, type ButtonStyleProps } from "@/modules/ui/components/Button";

const ButtonLinkBase: React.FC<RACLinkProps & ButtonStyleProps> = ({
  variant, size, className, children, ...props
}) => (
  <RACLink
    {...props}
    className={composeRenderProps(className, (cn) =>
      buttonStyle({ variant, size, className: cn }),
    )}
  >
    {children}
  </RACLink>
);

export const ButtonLink: LinkComponent<typeof ButtonLinkBase> = createLink(ButtonLinkBase);

// Usage
<ButtonLink to="/settings" variant="plain">Settings</ButtonLink>
```

## Checklist

- [ ] Props interface extends Aria + style variant props
- [ ] `className?: string` included in props
- [ ] `composeRenderProps` used for dynamic className
- [ ] Router links use `createLink` wrapper
