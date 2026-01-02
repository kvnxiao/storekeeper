import { createSignal, For, Show } from "solid-js";
import ResourceCard from "./ResourceCard";

interface Props {
  title: string;
  gameId: string;
  resources: Array<{ type: string; data: unknown }>;
}

function GameSection(props: Props) {
  const [expanded, setExpanded] = createSignal(true);

  return (
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded())}
        class="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
      >
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
          {props.title}
        </h2>
        <span
          class="text-gray-400 transform transition-transform"
          classList={{ "rotate-180": !expanded() }}
        >
          ▼
        </span>
      </button>

      <Show when={expanded()}>
        <div class="px-4 pb-4 grid grid-cols-2 gap-3">
          <For each={props.resources}>
            {(resource) => (
              <ResourceCard
                gameId={props.gameId}
                type={resource.type}
                data={resource.data}
              />
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

export default GameSection;
