# Performance Standards

## React Rendering

### Memoization

Use `useMemo` for expensive computations:

```tsx
// Bad: Recalculates every render
const timeRemaining = formatTimeRemaining(data.fullAt, tick);

// Good: Memoized calculation
const timeRemaining = useMemo(
  () => formatTimeRemaining(data.fullAt, tick),
  [data.fullAt, tick]
);
```

### Stable References

Avoid inline objects/arrays in props:

```tsx
// Bad: New object every render
<Component style={{ color: "red" }} />
<Component items={[1, 2, 3]} />

// Good: Stable reference
const style = useMemo(() => ({ color: "red" }), []);
const items = useMemo(() => [1, 2, 3], []);
<Component style={style} items={items} />
```

### Callback Memoization

Use `useCallback` for callbacks passed to children:

```tsx
// Bad: New function every render
<Button onPress={() => handleClick(id)}>

// Good: Stable callback
const handlePress = useCallback(() => handleClick(id), [id]);
<Button onPress={handlePress}>
```

## Animation Performance

### Transform Over Layout

Animate transform and opacity, not layout properties:

```tsx
// Bad: Animating layout properties
animate={{ height: "auto", top: 100, width: "50%" }}

// Good: Animating transform/opacity
animate={{ opacity: 1, y: 0, scale: 1 }}
```

### Reduced Motion Support

Always provide reduced motion alternatives:

```tsx
import { useReducedMotion } from "motion/react";

// Define both variants
export const cardItemVariants: Variants = {
  hidden: { opacity: 0, y: 10 },
  visible: { opacity: 1, y: 0 },
};

export const cardItemVariantsReduced: Variants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1 },
};

// Usage
const shouldReduceMotion = useReducedMotion();
const variants = shouldReduceMotion ? cardItemVariantsReduced : cardItemVariants;
const transition = shouldReduceMotion ? { duration: 0 } : springTransition;

<motion.div variants={variants} transition={transition} />
```

### Centralized Animation Variants

Keep animation definitions in `src/components/utils/animations.ts`:

```tsx
export const springTransition: Transition = {
  type: "spring",
  stiffness: 300,
  damping: 30,
};

export const cardContainerVariants: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { when: "beforeChildren", staggerChildren: 0.05 },
  },
};
```

## Bundle Size

### Tree-Shakeable Imports

Import specific modules, not entire libraries:

```tsx
// Bad: Imports entire icon set
import * as Icons from "@heroicons/react/24/outline";

// Good: Named import
import { Cog6ToothIcon } from "@heroicons/react/24/outline";
```

```tsx
// Bad: Full lodash
import _ from "lodash";

// Good: Specific function
import debounce from "lodash/debounce";
```

## List Rendering

### Stable Keys

Use unique, stable keys for dynamic lists:

```tsx
// Bad: Index as key for dynamic list
{items.map((item, index) => <Item key={index} />)}

// Good: Stable unique key
{resources.map((resource, index) => (
  <ResourceCard key={`${gameId}-${resource.type}-${index}`} />
))}
```

**Note**: Index keys are acceptable for static lists that never reorder.

## State Colocation

Keep state close to where it's used:

```tsx
// Bad: Global state for local concern
const [isExpanded, setIsExpanded] = useAtom(expandedAtom);

// Good: Local state
const [isExpanded, setIsExpanded] = useState(false);
```

Use Jotai atoms only for truly shared state.

## Checklist

- [ ] Expensive computations wrapped in `useMemo`
- [ ] Callbacks in props use `useCallback`
- [ ] No inline object/array literals in props
- [ ] Animations use transform/opacity
- [ ] Reduced motion alternatives provided
- [ ] Tree-shakeable imports only
- [ ] Stable keys for list items
