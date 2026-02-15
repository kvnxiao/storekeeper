import { RESOURCE_DISPLAY_NAMES } from "@/modules/games/games.constants";
import { NotificationResourceRow } from "@/modules/settings/components/NotificationResourceRow";
import type { ResourceNotificationConfig } from "@/modules/settings/settings.types";

interface NotificationSectionProps {
  gameId: string;
  resourceTypes: readonly string[];
  notifications:
    | Partial<Record<string, ResourceNotificationConfig>>
    | undefined;
  onChange: (
    notifications: Partial<Record<string, ResourceNotificationConfig>>,
  ) => void;
}

export const NotificationSection: React.FC<NotificationSectionProps> = ({
  gameId,
  resourceTypes,
  notifications,
  onChange,
}) => {
  return (
    <div className="space-y-3">
      <div>
        <h3 className="text-sm font-semibold text-zinc-950 dark:text-white">
          Notifications
        </h3>
        <p className="text-xs text-zinc-500 dark:text-zinc-400">
          Configure desktop notifications for this game's resources.
        </p>
      </div>
      {resourceTypes.map((type) => (
        <NotificationResourceRow
          key={type}
          gameId={gameId}
          resourceType={type}
          label={RESOURCE_DISPLAY_NAMES[type] ?? type}
          config={notifications?.[type]}
          onChange={(resourceConfig) =>
            onChange({ ...notifications, [type]: resourceConfig })
          }
        />
      ))}
    </div>
  );
};
