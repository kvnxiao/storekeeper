import type { ResourceLimits } from "@/modules/settings/components/NotificationResourceRow";
import { NotificationSection } from "@/modules/settings/components/NotificationSection";
import { Section } from "@/modules/settings/components/Section";
import type { WuwaConfig } from "@/modules/settings/settings.types";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

const RESOURCE_TYPES = ["waveplates"] as const;

interface WuwaSectionProps {
  config: WuwaConfig | undefined;
  resourceLimits?: Partial<Record<string, ResourceLimits>>;
  onChange: (config: WuwaConfig) => void;
}

export const WuwaSection: React.FC<WuwaSectionProps> = ({
  config,
  resourceLimits,
  onChange,
}) => {
  const enabled = config?.enabled ?? false;
  const playerId = config?.player_id ?? "";

  return (
    <Section
      title={m.game_wuthering_waves()}
      description={m.settings_game_configure_wuwa()}
    >
      <Switch
        isSelected={enabled}
        onChange={(isSelected) =>
          onChange({
            ...config,
            enabled: isSelected,
            player_id: playerId,
          })
        }
      >
        {m.settings_wuwa_enable_tracking()}
      </Switch>
      {enabled && (
        <>
          <TextField
            label={m.settings_wuwa_player_id()}
            value={playerId}
            onChange={(value) =>
              onChange({
                ...config,
                enabled,
                player_id: value,
              })
            }
            placeholder={m.settings_wuwa_player_id_placeholder()}
          />
          <NotificationSection
            gameId="WUTHERING_WAVES"
            resourceTypes={RESOURCE_TYPES}
            notifications={config?.notifications}
            resourceLimits={resourceLimits}
            onChange={(notifications) =>
              onChange({
                ...config,
                enabled,
                player_id: playerId,
                notifications,
              })
            }
          />
        </>
      )}
    </Section>
  );
};
