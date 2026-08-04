import { For, type VoidComponent } from "solid-js";
import { getResourceDisplayName, STAMINA_RESOURCE_TYPES } from "@/modules/games/games.constants";
import type { GameId } from "@/modules/games/games.types";
import {
  NotificationResourceRow,
  type ResourceLimits,
} from "@/modules/settings/components/NotificationResourceRow";
import type { ResourceNotificationConfig } from "@/modules/settings/settings.types";
import * as m from "@/paraglide/messages";

export interface NotificationSectionProps {
  gameId: GameId;
  resourceTypes: readonly string[];
  notifications: Partial<Record<string, ResourceNotificationConfig>> | undefined;
  resourceLimits?: Partial<Record<string, ResourceLimits>>;
  onChange: (notifications: Partial<Record<string, ResourceNotificationConfig>>) => void;
}

export const NotificationSection: VoidComponent<NotificationSectionProps> = (props) => {
  return (
    <div class="space-y-3">
      <div>
        <h3 class="text-sm font-semibold text-zinc-950 dark:text-white">
          {m.settings_notifications_title()}
        </h3>
        <p class="text-xs text-zinc-500 dark:text-zinc-400">
          {m.settings_notifications_description()}
        </p>
      </div>
      <For each={props.resourceTypes}>
        {(type) => (
          <NotificationResourceRow
            gameId={props.gameId}
            resourceType={type}
            label={getResourceDisplayName(type)}
            config={props.notifications?.[type]}
            isStaminaResource={STAMINA_RESOURCE_TYPES.has(type)}
            limits={props.resourceLimits?.[type]}
            onChange={(resourceConfig) =>
              props.onChange({ ...props.notifications, [type]: resourceConfig })
            }
          />
        )}
      </For>
    </div>
  );
};
