import { useState } from "react";
import { Disclosure, DisclosurePanel } from "react-aria-components";

import { ResourceCard } from "@/components/ResourceCard";
import type { GameId, GameResource } from "@/types";

interface Props {
  title: string;
  gameId: GameId;
  resources: GameResource[];
}

export const GameSection: React.FC<Props> = ({ title, gameId, resources }) => {
  const [isExpanded, setIsExpanded] = useState(true);

  return (
    <Disclosure
      isExpanded={isExpanded}
      onExpandedChange={setIsExpanded}
      className="bg-white dark:bg-gray-800 rounded-lg shadow-sm overflow-hidden"
    >
      <h2>
        <button
          type="button"
          slot="trigger"
          className="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors cursor-pointer"
          onClick={() => setIsExpanded((prev) => !prev)}
        >
          <span className="text-lg font-semibold text-gray-900 dark:text-white">
            {title}
          </span>
          <span
            className={`text-gray-400 transform transition-transform duration-200 ${
              isExpanded ? "" : "-rotate-90"
            }`}
          >
            ▼
          </span>
        </button>
      </h2>
      <DisclosurePanel>
        <div className="px-4 pb-4 grid grid-cols-2 gap-3">
          {resources.map((resource, index) => (
            <ResourceCard
              key={`${gameId}-${resource.type}-${index}`}
              gameId={gameId}
              type={resource.type}
              data={resource.data}
            />
          ))}
        </div>
      </DisclosurePanel>
    </Disclosure>
  );
};
