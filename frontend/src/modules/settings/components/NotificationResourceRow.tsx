import { BellAlertIcon } from "@heroicons/react/20/solid";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";
import type { ResourceNotificationConfig } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { NumberField } from "@/modules/ui/components/NumberField";
import { SegmentedControl } from "@/modules/ui/components/SegmentedControl";
import { Switch } from "@/modules/ui/components/Switch";
import * as m from "@/paraglide/messages";

type NotifyMode = "minutes" | "value";

function getNotifyMode(config: ResourceNotificationConfig): NotifyMode {
  if (config.notify_at_value != null) return "value";
  return "minutes";
}

/** Resource limits derived from backend data, used for input constraints */
export interface ResourceLimits {
  /** Maximum resource value (e.g., 160 for resin, 240 for trailblaze power) */
  maxValue: number;
  /** Seconds to regenerate one unit */
  regenRateSeconds: number;
}

interface NotificationResourceRowProps {
  gameId: string;
  resourceType: string;
  label: string;
  config: ResourceNotificationConfig | undefined;
  isStaminaResource: boolean;
  limits?: ResourceLimits;
  onChange: (config: ResourceNotificationConfig) => void;
}

const DEFAULT_CONFIG: ResourceNotificationConfig = {
  enabled: true,
  cooldown_minutes: 30,
};

export const NotificationResourceRow: React.FC<
  NotificationResourceRowProps
> = ({
  gameId,
  resourceType,
  label,
  config,
  isStaminaResource,
  limits,
  onChange,
}) => {
  const enabled = config?.enabled ?? false;
  const [isPreviewing, setIsPreviewing] = useState<boolean>(false);

  const mode = config ? getNotifyMode(config) : "minutes";

  const modeItems = [
    { id: "minutes", label: m.settings_notification_minutes_before_full() },
    { id: "value", label: m.settings_notification_at_value() },
  ] as const;

  const handleToggle = useCallback(
    (isSelected: boolean) => {
      if (isSelected) {
        if (!config) {
          onChange(DEFAULT_CONFIG);
        } else if (isStaminaResource) {
          onChange({ ...config, enabled: true });
        } else {
          // Cooldown resources: clear threshold fields so backend uses "notify when complete"
          onChange({
            ...config,
            enabled: true,
            notify_minutes_before_full: null,
            notify_at_value: null,
          });
        }
      } else if (config) {
        onChange({ ...config, enabled: false });
      }
    },
    [config, isStaminaResource, onChange],
  );

  const handleModeChange = useCallback(
    (newMode: string) => {
      if (!config) return;
      if (newMode === "value") {
        onChange({
          ...config,
          notify_minutes_before_full: null,
          notify_at_value: config.notify_at_value ?? 0,
        });
      } else {
        onChange({
          ...config,
          notify_at_value: null,
          notify_minutes_before_full: config.notify_minutes_before_full ?? 0,
        });
      }
    },
    [config, onChange],
  );

  const handlePreview = useCallback(async () => {
    setIsPreviewing(true);
    try {
      await invoke("send_preview_notification", {
        gameId,
        resourceType,
      });
    } finally {
      setIsPreviewing(false);
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
          aria-label={m.settings_notification_preview({ label })}
          onPress={() => void handlePreview()}
          isPending={isPreviewing}
        >
          {!isPreviewing && <BellAlertIcon className="h-4 w-4" />}
        </Button>
      </div>
      {enabled && config && (
        <>
          {/* Empty cell aligns with switch track width */}
          <div />
          <div className="space-y-3">
            {isStaminaResource && (
              <>
                <SegmentedControl
                  aria-label={m.settings_notification_mode()}
                  selectedKey={mode}
                  onSelectionChange={handleModeChange}
                  items={[...modeItems]}
                />
                {mode === "minutes" ? (
                  <NumberField
                    label={m.settings_notification_minutes_before_full()}
                    value={config.notify_minutes_before_full ?? 0}
                    onChange={(value) =>
                      onChange({
                        ...config,
                        notify_minutes_before_full: value,
                        notify_at_value: null,
                      })
                    }
                    minValue={0}
                    maxValue={
                      limits
                        ? Math.floor(
                            (limits.maxValue * limits.regenRateSeconds) / 60,
                          )
                        : 999
                    }
                    step={5}
                  />
                ) : (
                  <NumberField
                    label={
                      limits
                        ? m.settings_notification_value_with_max({
                            maxValue: String(limits.maxValue),
                          })
                        : m.settings_notification_value()
                    }
                    value={config.notify_at_value ?? 0}
                    onChange={(value) =>
                      onChange({
                        ...config,
                        notify_at_value: value,
                        notify_minutes_before_full: null,
                      })
                    }
                    minValue={1}
                    maxValue={limits?.maxValue ?? 9999}
                    step={1}
                  />
                )}
              </>
            )}
            <NumberField
              label={m.settings_notification_cooldown()}
              description={
                config.cooldown_minutes === 0
                  ? m.settings_notification_once()
                  : isStaminaResource
                    ? m.settings_notification_renotify_stamina()
                    : m.settings_notification_renotify_cooldown()
              }
              value={config.cooldown_minutes}
              onChange={(value) =>
                onChange({ ...config, cooldown_minutes: value })
              }
              minValue={0}
              maxValue={120}
              step={1}
            />
          </div>
        </>
      )}
    </div>
  );
};
