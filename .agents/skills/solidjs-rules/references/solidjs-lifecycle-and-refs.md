---
paths: **/*.{tsx,jsx}
description: "SolidJS lifecycle and ref rules; onMount/onCleanup pairing, no cleanup-return from effects, ref timing, and signal refs for conditional elements."
---

# Lifecycle and Refs

Components run once, so lifecycle hooks cover mount and disposal: `onMount` runs once after the component's elements are in the DOM, and `onCleanup` runs when the owning scope disposes — on unmount, or before each re-run when registered inside an effect or memo. There is no `componentDidUpdate` equivalent; reactive computations handle updates.

## Pair Every Imperative Resource With `onCleanup` (Required)

Register cleanup in the same scope that creates the resource: intervals, `window` listeners, observers, third-party widget instances.

```tsx
onMount(() => {
  const chart = new Chart(el, options);
  onCleanup(() => chart.destroy());
});
```

## Effects Do Not Return Cleanup Functions (Required)

Returning a function from `createEffect` does nothing. Register `onCleanup` inside the effect instead; it runs before each re-run and on disposal.

```tsx
createEffect(() => {
  const id = setInterval(tick, delay());
  return () => clearInterval(id);
});
```

`onCleanup` registers disposal with the effect owner:

```tsx
createEffect(() => {
  const id = setInterval(tick, delay());
  onCleanup(() => clearInterval(id));
});
```

## Refs: Assigned During Render, Ready in `onMount` (Required)

Use a definite-assignment local with the `ref` attribute. The ref is set before `onMount`; perform DOM measurement in `onMount`, never in the component body.

```tsx
let el!: HTMLDivElement;

onMount(() => setWidth(el.getBoundingClientRect().width));

return <div ref={el} />;
```

## Signal Refs for Conditional Elements (Required)

Inside `<Show>` or other control flow, a plain local can be unset or stale. Store the ref in a signal so consumers observe the element's lifetime. Solid does not unobserve a removed element or clear stored references automatically, so branch cleanup must perform both operations.

```tsx
const [el, setEl] = createSignal<HTMLDivElement>();

createEffect(() => {
  const node = el();
  if (!node) return;
  observer.observe(node);
});

<Show when={open()}>
  {() => {
    let node!: HTMLDivElement;
    onCleanup(() => {
      observer.unobserve(node);
      setEl(undefined);
    });
    return (
      <div
        ref={(value) => {
          node = value;
          setEl(value);
        }}
      />
    );
  }}
</Show>
```

## Plain `let` Replaces `useRef` Boxes (Default)

Any `let` in the component body is a stable instance variable because the function runs once. Non-reactive instance state does not need a mutable-box wrapper.
