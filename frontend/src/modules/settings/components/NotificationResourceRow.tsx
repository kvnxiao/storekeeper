import { BellAlertIcon } from "@heroicons/react/20/solid";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";
import type { ResourceNotificationConfig } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { NumberField } from "@/modules/ui/components/NumberField";
import { Switch } from "@/modules/ui/components/Switch";

interface NotificationResourceRowProps {
  gameId: string;
  resourceType: string;
  label: string;
  config: ResourceNotificationConfig | undefined;
  onChange: (config: ResourceNotificationConfig) => void;
}

const DEFAULT_CONFIG: ResourceNotificationConfig = {
  enabled: true,
  notify_minutes_before_full: 0,
  cooldown_minutes: 30,
};

export const NotificationResourceRow: React.FC<
  NotificationResourceRowProps
> = ({ gameId, resourceType, label, config, onChange }) => {
  const enabled = config?.enabled ?? false;
  const [isTesting, setIsTesting] = useState(false);

  const handleToggle = useCallback(
    (isSelected: boolean) => {
      if (isSelected) {
        onChange(config ? { ...config, enabled: true } : DEFAULT_CONFIG);
      } else if (config) {
        onChange({ ...config, enabled: false });
      }
    },
    [config, onChange],
  );

  const handleTest = useCallback(async () => {
    setIsTesting(true);
    try {
      await invoke("send_test_notification", {
        gameId,
        resourceType,
      });
    } finally {
      setIsTesting(false);
    }
  }, [gameId, resourceType]);

  return (
    <div className="grid grid-cols-[2rem_1fr] gap-x-2 gap-y-2">
      <div className="col-span-2 flex items-center gap-2">
        <Switch isSelected={enabled} onChange={handleToggle}>
          {label}
        </Switch>
        <Button
          size="icon"
          variant="plain"
          aria-label={`Test ${label} notification`}
          onPress={() => void handleTest()}
          isPending={isTesting}
        >
          {!isTesting && <BellAlertIcon className="h-4 w-4" />}
        </Button>
      </div>
      {enabled && config && (
        <>
          {/* Empty cell aligns with switch track width */}
          <div />
          <div className="space-y-3">
            <NumberField
              label="Notify minutes before full"
              value={config.notify_minutes_before_full}
              onChange={(value) =>
                onChange({ ...config, notify_minutes_before_full: value })
              }
              minValue={0}
              maxValue={999}
              step={5}
            />
            <NumberField
              label="Cooldown minutes"
              description={
                config.cooldown_minutes === 0
                  ? "0 = notify once, no repeat"
                  : undefined
              }
              value={config.cooldown_minutes}
              onChange={(value) =>
                onChange({ ...config, cooldown_minutes: value })
              }
              minValue={0}
              maxValue={120}
              step={5}
            />
          </div>
        </>
      )}
    </div>
  );
};
