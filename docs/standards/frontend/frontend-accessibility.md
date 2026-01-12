# Accessibility Standards

## React Aria Foundation

All interactive components use [React Aria Components](https://react-spectrum.adobe.com/react-aria/), providing:

- Proper ARIA attributes automatically
- Keyboard navigation
- Focus management
- Screen reader support

### Use Semantic Components

```tsx
// Bad: Manual ARIA on div
<div role="button" tabIndex={0} onClick={handleClick}>
  <Cog6ToothIcon />
</div>

// Good: React Aria Button
<Button onPress={handleClick} aria-label="Settings">
  <Cog6ToothIcon aria-hidden="true" />
</Button>
```

**Never add `role` attributes manually** - React Aria handles this.

## Touch Targets

Minimum 44x44px touch targets per [WCAG 2.5.5](https://www.w3.org/WAI/WCAG21/Understanding/target-size.html).

Use the `touch-target` utility for small interactive elements:

```tsx
// Small icon button needs expanded touch target
<Button aria-label="Close" className="touch-target">
  <XMarkIcon className="h-4 w-4" aria-hidden="true" />
</Button>
```

The utility expands the hit area on touch devices only:

```css
@utility touch-target {
  position: relative;
  &::before {
    content: "";
    position: absolute;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    min-height: 44px; min-width: 44px;
    @media (pointer: fine) { display: none; }
  }
}
```

## Reduced Motion

Respect user preference for reduced motion.

### CSS Global Rule

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

### Motion Library

```tsx
import { useReducedMotion } from "motion/react";

const shouldReduceMotion = useReducedMotion();
const variants = shouldReduceMotion ? reducedVariants : fullVariants;
```

## Icon Accessibility

### Decorative Icons

Icons alongside text are decorative:

```tsx
// Good: Icon hidden from screen readers
<Button>
  <CheckIcon aria-hidden="true" />
  Submit
</Button>
```

### Icon-Only Buttons

Icon-only buttons require `aria-label`:

```tsx
// Good: Accessible icon button
<Button aria-label="Close dialog">
  <XMarkIcon aria-hidden="true" />
</Button>
```

**Always include both** `aria-hidden` on the icon and `aria-label` on the button.

## Focus Management

### Visible Focus Indicators

Use `focus-visible:` for keyboard-only focus rings:

```tsx
// Bad: Focus on all interactions
className="focus:ring-2 focus:ring-blue-500"

// Good: Keyboard focus only
className="focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
```

**Never remove focus outlines** without replacement.

### Focus Trapping

React Aria handles focus trapping in modals, dialogs, and popovers automatically.

## Form Accessibility

React Aria's slot system associates labels, descriptions, and errors:

```tsx
<TextField>
  <Label>Email</Label>
  <Input />
  <Text slot="description">We'll never share your email.</Text>
  <FieldError>Please enter a valid email.</FieldError>
</TextField>
```

This automatically sets `aria-describedby` and `aria-errormessage`.

## Color Contrast

Ensure text meets WCAG AA contrast ratios:

- Normal text: 4.5:1
- Large text (18px+ or 14px+ bold): 3:1

Semantic color tokens are designed to meet these requirements.

## Checklist

- [ ] Interactive elements use React Aria components
- [ ] Icon-only buttons have `aria-label`
- [ ] Decorative icons have `aria-hidden="true"`
- [ ] Touch targets meet 44x44px minimum
- [ ] Animations respect `prefers-reduced-motion`
- [ ] Focus indicators use `focus-visible:`
- [ ] No custom `role` attributes (React Aria handles)
