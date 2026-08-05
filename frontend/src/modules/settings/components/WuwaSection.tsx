import { Show, type VoidComponent } from "solid-js";
import { WuwaResource } from "@/modules/games/games.constants";
import { GameId } from "@/modules/games/games.types";
import type { ResourceLimits } from "@/modules/resources/resources.types";
import { NotificationSection } from "@/modules/settings/components/NotificationSection";
import { Section } from "@/modules/settings/components/Section";
import type { WuwaConfig } from "@/modules/settings/settings.types";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

const RESOURCE_TYPES = Object.values(WuwaResource);

export interface WuwaSectionProps {
  config: WuwaConfig | undefined;
  resourceLimits?: Partial<Record<string, ResourceLimits>>;
  onChange: (config: WuwaConfig) => void;
}

export const WuwaSection: VoidComponent<WuwaSectionProps> = (props) => {
  const enabled = () => props.config?.enabled ?? false;
  const uid = () => props.config?.uid ?? "";

  return (
    <Section title={m.game_wuwa_name()} description={m.settings_game_configure_wuwa()}>
      <Switch
        checked={enabled()}
        onChange={(checked) =>
          props.onChange({
            ...props.config,
            enabled: checked,
            uid: uid(),
          })
        }
      >
        {m.settings_wuwa_enable_tracking()}
      </Switch>
      <Show when={enabled()}>
        <TextField
          label={m.settings_game_uid()}
          value={uid()}
          onChange={(value) =>
            props.onChange({
              ...props.config,
              enabled: enabled(),
              uid: value,
            })
          }
          placeholder={m.settings_game_uid_placeholder()}
        />
        <NotificationSection
          gameId={GameId.WutheringWaves}
          resourceTypes={RESOURCE_TYPES}
          notifications={props.config?.notifications}
          resourceLimits={props.resourceLimits}
          onChange={(notifications) =>
            props.onChange({
              ...props.config,
              enabled: enabled(),
              uid: uid(),
              notifications,
            })
          }
        />
      </Show>
    </Section>
  );
};
