import {
  getResourceDisplayName,
  STAMINA_RESOURCE_TYPES,
} from "@/modules/games/games.constants";
import type { GameId } from "@/modules/games/games.types";
import {
  NotificationResourceRow,
  type ResourceLimits,
} from "@/modules/settings/components/NotificationResourceRow";
import type { ResourceNotificationConfig } from "@/modules/settings/settings.types";
import * as m from "@/paraglide/messages";

interface NotificationSectionProps {
  gameId: GameId;
  resourceTypes: readonly string[];
  notifications:
    | Partial<Record<string, ResourceNotificationConfig>>
    | undefined;
  resourceLimits?: Partial<Record<string, ResourceLimits>>;
  onChange: (
    notifications: Partial<Record<string, ResourceNotificationConfig>>,
  ) => void;
}

export const NotificationSection: React.FC<NotificationSectionProps> = ({
  gameId,
  resourceTypes,
  notifications,
  resourceLimits,
  onChange,
}) => {
  return (
    <div className="space-y-3">
      <div>
        <h3 className="text-sm font-semibold text-zinc-950 dark:text-white">
          {m.settings_notifications_title()}
        </h3>
        <p className="text-xs text-zinc-500 dark:text-zinc-400">
          {m.settings_notifications_description()}
        </p>
      </div>
      {resourceTypes.map((type) => (
        <NotificationResourceRow
          key={type}
          gameId={gameId}
          resourceType={type}
          label={getResourceDisplayName(type)}
          config={notifications?.[type]}
          isStaminaResource={STAMINA_RESOURCE_TYPES.has(type)}
          limits={resourceLimits?.[type]}
          onChange={(resourceConfig) =>
            onChange({ ...notifications, [type]: resourceConfig })
          }
        />
      ))}
    </div>
  );
};
