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

export interface WuwaGameSectionProps {
  config: WuwaConfig | undefined;
  resourceLimits?: Partial<Record<string, ResourceLimits>>;
  onChange: (config: WuwaConfig) => void;
}

export const WuwaGameSection: VoidComponent<WuwaGameSectionProps> = (props) => {
  // Normalizes the possibly-undefined config once so every onChange handler
  // only states the field it changes.
  const current = (): WuwaConfig => ({ enabled: false, uid: "", ...props.config });

  return (
    <Section title={m.game_wuwa_name()} description={m.settings_game_configure_wuwa()}>
      <Switch
        checked={current().enabled}
        onChange={(checked) => props.onChange({ ...current(), enabled: checked })}
      >
        {m.settings_wuwa_enable_tracking()}
      </Switch>
      <Show when={current().enabled}>
        <TextField
          label={m.settings_game_uid()}
          value={current().uid}
          onChange={(value) => props.onChange({ ...current(), uid: value })}
          placeholder={m.settings_game_uid_placeholder()}
        />
        <NotificationSection
          gameId={GameId.WutheringWaves}
          resourceTypes={RESOURCE_TYPES}
          notifications={props.config?.notifications}
          resourceLimits={props.resourceLimits}
          onChange={(notifications) => props.onChange({ ...current(), notifications })}
        />
      </Show>
    </Section>
  );
};
