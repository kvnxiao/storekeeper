---
paths: **/*.{ts,tsx,js,jsx}
description: "TanStack Form rules for SolidJS; headless form modules owning formOptions, schema validation, and submission, with components rendering accessor-shaped fields and narrow subscriptions."
---

# Forms

`@tanstack/solid-form` is the form library. Forms are where UI state and business logic meet, so the state architecture split is applied at the line where it holds: field values, touched/dirty flags, and error display are UI machinery and live in the component's form instance; what *valid* means, the default values, and what submission *does* are headless domain code.

## Form Modules Own Options, Schema, and Submission

Each form gets a module exporting the validation schema, shared `formOptions`, and a submit function that delegates to domain actions or mutations. All three are testable as plain TypeScript without rendering anything.

```ts
// src/state/checkout-form.ts
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

## Components Render Fields

The component spreads the module's options into `createForm` — options are function-wrapped, like every Solid adapter — and decides only how fields look. `field` is an accessor: `field().state.value`, `field().handleChange(…)`; forgetting the `field()` call is the classic React-porting mistake.

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

## Subscribe Narrowly

Read derived form state (`canSubmit`, `isSubmitting`) through `form.Subscribe` with a `selector` rather than subscribing to whole form state; the selected value arrives as an accessor. Array fields use `<form.Field name="items" mode="array">` with `pushValue`/`removeValue`/`swapValues` on the field.

## Draft State Stays in the Form

The form instance is the UI-local home for in-progress values — do not mirror drafts into signals or stores. A value graduates out of the form only on submit, through the module's submit function, which is the single place form data touches business state or the server (typically via a mutation; see the data fetching rules).
