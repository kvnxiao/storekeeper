import { useMutation } from "@tanstack/solid-query";
import BellRing from "lucide-solid/icons/bell-ring";
import { Show, type VoidComponent } from "solid-js";
import type { ResourceType } from "@/modules/games/games.constants";
import type { GameId } from "@/modules/games/games.types";
import type { ResourceLimits } from "@/modules/resources/resources.types";
import { previewNotificationMutationOptions } from "@/modules/settings/settings.query";
import type { ResourceNotificationConfig } from "@/modules/settings/settings.types";
import {
  enabledNotificationConfig,
  getNotifyMode,
  withNotifyAtValue,
  withNotifyMinutes,
  withNotifyMode,
} from "@/modules/settings/settings.utils";
import { Button } from "@/modules/ui/components/Button";
import { NumberField } from "@/modules/ui/components/NumberField";
import { SegmentedControl } from "@/modules/ui/components/SegmentedControl";
import { Switch } from "@/modules/ui/components/Switch";
import * as m from "@/paraglide/messages";

export interface NotificationResourceRowProps {
  gameId: GameId;
  resourceType: ResourceType;
  label: string;
  config: ResourceNotificationConfig | undefined;
  isStaminaResource: boolean;
  limits?: ResourceLimits;
  onChange: (config: ResourceNotificationConfig) => void;
}

// Evaluated at call time so labels follow the active locale
const modeItems = () => [
  { id: "minutes", label: m.settings_notification_minutes_before_full() },
  { id: "value", label: m.settings_notification_at_value() },
];

export const NotificationResourceRow: VoidComponent<NotificationResourceRowProps> = (props) => {
  const enabled = () => props.config?.enabled ?? false;
  const preview = useMutation(() => previewNotificationMutationOptions());

  const mode = () => (props.config ? getNotifyMode(props.config) : "minutes");

  const cooldownDescription = () => {
    if (props.config?.cooldown_minutes === 0) {
      return m.settings_notification_once();
    }
    if (props.isStaminaResource) {
      return m.settings_notification_renotify_stamina();
    }
    return m.settings_notification_renotify_cooldown();
  };

  const handleToggle = (checked: boolean) => {
    const config = props.config;
    if (checked) {
      props.onChange(enabledNotificationConfig(config, props.isStaminaResource));
    } else if (config) {
      props.onChange({ ...config, enabled: false });
    }
  };

  const handleModeChange = (newMode: string) => {
    if (props.config) {
      props.onChange(withNotifyMode(props.config, newMode === "value" ? "value" : "minutes"));
    }
  };

  return (
    <div class="grid grid-cols-[2rem_1fr] gap-x-2 gap-y-2">
      <div class="col-span-2 flex items-center gap-2">
        <Switch checked={enabled()} onChange={handleToggle}>
          {props.label}
        </Switch>
        <Button
          size="icon"
          variant="plain"
          aria-label={m.settings_notification_preview({ label: props.label })}
          onClick={() => preview.mutate({ gameId: props.gameId, resourceType: props.resourceType })}
          isPending={preview.isPending}
        >
          <Show when={!preview.isPending}>
            <BellRing aria-hidden="true" class="size-4" />
          </Show>
        </Button>
      </div>
      <Show when={enabled() ? props.config : undefined}>
        {(config) => (
          <>
            {/* Empty cell aligns with switch track width */}
            <div />
            <div class="space-y-3">
              <Show when={props.isStaminaResource}>
                <SegmentedControl
                  aria-label={m.settings_notification_mode()}
                  selectedKey={mode()}
                  onSelectionChange={handleModeChange}
                  items={modeItems()}
                />
                <Show
                  when={mode() === "minutes"}
                  fallback={
                    <NumberField
                      label={
                        props.limits
                          ? m.settings_notification_value_with_max({
                              maxValue: String(props.limits.maxValue),
                            })
                          : m.settings_notification_value()
                      }
                      value={config().notify_at_value ?? 1}
                      onChange={(value) => props.onChange(withNotifyAtValue(config(), value))}
                      minValue={1}
                      maxValue={props.limits?.maxValue ?? 9999}
                      step={1}
                    />
                  }
                >
                  <NumberField
                    label={m.settings_notification_minutes_before_full()}
                    value={config().notify_minutes_before_full ?? 0}
                    onChange={(value) => props.onChange(withNotifyMinutes(config(), value))}
                    minValue={0}
                    maxValue={
                      props.limits
                        ? Math.floor((props.limits.maxValue * props.limits.regenRateSeconds) / 60)
                        : 999
                    }
                    step={5}
                  />
                </Show>
              </Show>
              <NumberField
                label={m.settings_notification_cooldown()}
                description={cooldownDescription()}
                value={config().cooldown_minutes}
                onChange={(value) => props.onChange({ ...config(), cooldown_minutes: value })}
                minValue={0}
                maxValue={120}
                step={1}
              />
            </div>
          </>
        )}
      </Show>
    </div>
  );
};
