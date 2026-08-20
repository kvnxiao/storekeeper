---
paths: **/*.{ts,tsx,js,jsx}
description: "TanStack Form rules for SolidJS; headless form modules owning formOptions, schema validation, and submission, with components rendering accessor-shaped fields and narrow subscriptions."
---

# Forms

Use `@tanstack/solid-form` when validation, submission state, field arrays, or cross-field workflows are substantial. Keep simple forms local when native controls and a small submit handler express the full behavior.

## Form Modules Own Options, Schema, and Submission (Default)

When a form has a shared schema, reusable options, or a multi-step submission workflow, extract a module that exports those contracts and delegates submission to domain actions or mutations. Keep view-specific defaults and a short submit handler in the component when extraction would only move code.

```ts
import { formOptions } from "@tanstack/solid-form";
import * as v from "valibot";

export const checkoutSchema = v.object({
  email: v.pipe(v.string(), v.email()),
  quantity: v.pipe(v.number(), v.minValue(1)),
});

export type CheckoutInput = v.InferOutput<typeof checkoutSchema>;

export const checkoutFormOptions = formOptions({
  defaultValues: { email: "", quantity: 1 } as CheckoutInput,
  validators: { onChange: checkoutSchema },
});

export async function submitCheckout(value: CheckoutInput) {
  return cart.checkout(value);
}
```

Validation uses a Standard Schema library (valibot, zod, arktype) passed to `validators` — business rules live in the schema, not scattered across inline JSX validator closures. Inline field validators are for UI-scoped concerns (for example an `onChangeAsync` availability check, whose function should itself be imported from the module).

## Components Render Fields (Required)

The component spreads the module's options into `createForm`; Solid adapter options are function-wrapped, and the component decides only how fields look. `field` is an accessor: use `field().state.value` and `field().handleChange(…)`. Omitting the `field()` call reads the accessor object instead of its value.

```tsx
const CheckoutForm: Component = () => {
  const form = createForm(() => ({
    ...checkoutFormOptions,
    onSubmit: ({ value }) => submitCheckout(value),
  }));

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        form.handleSubmit();
      }}
    >
      <form.Field name="email">
        {(field) => (
          <>
            <input
              value={field().state.value}
              onBlur={field().handleBlur}
              onInput={(e) => field().handleChange(e.currentTarget.value)}
            />
            <Show when={!field().state.meta.isValid}>
              <em>
                {field().state.meta.errors.map((e) => e?.message).join(", ")}
              </em>
            </Show>
          </>
        )}
      </form.Field>
      <form.Subscribe selector={(state) => state.canSubmit}>
        {(canSubmit) => (
          <button type="submit" disabled={!canSubmit()}>
            Submit
          </button>
        )}
      </form.Subscribe>
    </form>
  );
};
```

## Subscribe Narrowly (Default)

Default derived form state such as `canSubmit` and `isSubmitting` to a narrow `form.Subscribe` selector. Subscribe to the full state only when the view consumes most of it. Array fields use `<form.Field name="items" mode="array">` with `pushValue`, `removeValue`, and `swapValues` on the field.

## Draft State Stays in the Form (Default)

Default in-progress values to the form instance. Mirror a draft only when it must survive form disposal, synchronize with another view, or become an offline snapshot. Route substantial submissions through the form module; a simple local form can submit directly to its domain action.
