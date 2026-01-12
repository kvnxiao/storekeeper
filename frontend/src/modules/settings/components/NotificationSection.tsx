import { Section } from "@/modules/settings/components/Section";
import type { NotificationConfig } from "@/modules/settings/settings.types";
import { NumberField } from "@/modules/ui/components/NumberField";
import { Switch } from "@/modules/ui/components/Switch";

interface NotificationSectionProps {
  config: NotificationConfig;
  onChange: (config: NotificationConfig) => void;
}

export const NotificationSection: React.FC<NotificationSectionProps> = ({
  config,
  onChange,
}) => {
  return (
    <Section
      title="Notifications"
      description="Configure desktop notification behavior."
    >
      <Switch
        isSelected={config.enabled}
        onChange={(isSelected) =>
          onChange({
            ...config,
            enabled: isSelected,
          })
        }
      >
        Enable desktop notifications
      </Switch>
      <NumberField
        label="Notification Cooldown (minutes)"
        description="Minimum time between notifications for the same resource."
        value={config.cooldown_minutes}
        onChange={(value) =>
          onChange({
            ...config,
            cooldown_minutes: value,
          })
        }
        minValue={5}
        maxValue={120}
        step={5}
      />
    </Section>
  );
};
