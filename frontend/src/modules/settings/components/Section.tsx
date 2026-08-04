import { type ParentComponent, Show } from "solid-js";

export interface SectionProps {
  title: string;
  description?: string;
}

export const Section: ParentComponent<SectionProps> = (props) => (
  <section class="rounded-lg border border-zinc-200 bg-white p-4 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
    <h2 class="mb-1 text-lg font-semibold text-zinc-950 dark:text-white">{props.title}</h2>
    <Show when={props.description}>
      <p class="mb-4 text-sm text-zinc-500 dark:text-zinc-400">{props.description}</p>
    </Show>
    <div class="space-y-4">{props.children}</div>
  </section>
);
