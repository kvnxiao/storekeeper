import { WuwaResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import type { ResourceLimits } from "@/modules/settings/components/NotificationResourceRow";
import { NotificationSection } from "@/modules/settings/components/NotificationSection";
import { Section } from "@/modules/settings/components/Section";
import type { WuwaConfig } from "@/modules/settings/settings.types";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

const RESOURCE_TYPES = [WuwaResource.Waveplates] as const;

interface WuwaSectionProps {
  config: WuwaConfig | undefined;
  resourceLimits?: Partial<Record<string, ResourceLimits>>;
  onChange: (config: WuwaConfig) => void;
}

export const WuwaSection: React.FC<WuwaSectionProps> = ({ config, resourceLimits, onChange }) => {
  const enabled = config?.enabled ?? false;
  const uid = config?.uid ?? "";

  return (
    <Section title={m.game_wuwa_name()} description={m.settings_game_configure_wuwa()}>
      <Switch
        isSelected={enabled}
        onChange={(isSelected) =>
          onChange({
            ...config,
            enabled: isSelected,
            uid,
          })
        }
      >
        {m.settings_wuwa_enable_tracking()}
      </Switch>
      {enabled && (
        <>
          <TextField
            label={m.settings_game_uid()}
            value={uid}
            onChange={(value) =>
              onChange({
                ...config,
                enabled,
                uid: value,
              })
            }
            placeholder={m.settings_game_uid_placeholder()}
          />
          <NotificationSection
            gameId={GameId.WutheringWaves}
            resourceTypes={RESOURCE_TYPES}
            notifications={config?.notifications}
            resourceLimits={resourceLimits}
            onChange={(notifications) =>
              onChange({
                ...config,
                enabled,
                uid,
                notifications,
              })
            }
          />
        </>
      )}
    </Section>
  );
};
