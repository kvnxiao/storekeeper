import { Show, type VoidComponent } from "solid-js";
import type { ResourceType } from "@/modules/games/games.constants";
import type { GameId } from "@/modules/games/games.types";
import type { ResourceLimits } from "@/modules/resources/resources.types";
import { NotificationSection } from "@/modules/settings/components/NotificationSection";
import { SettingsCard } from "@/modules/settings/components/SettingsCard";
import type { HoyolabGameConfig } from "@/modules/settings/settings.types";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

export interface HoyolabGameSectionProps {
  title: string;
  description: string;
  gameId: GameId;
  resourceTypes: readonly ResourceType[];
  config: HoyolabGameConfig | undefined;
  resourceLimits?: Partial<Record<ResourceType, ResourceLimits>>;
  onChange: (config: HoyolabGameConfig) => void;
}

export const HoyolabGameSection: VoidComponent<HoyolabGameSectionProps> = (props) => {
  // Normalize the optional config once; each onChange handler then updates one
  // field.
  const current = (): HoyolabGameConfig => ({
    enabled: false,
    uid: "",
    auto_claim_daily_rewards: false,
    ...props.config,
  });

  return (
    <SettingsCard title={props.title} description={props.description}>
      <Switch
        checked={current().enabled}
        onChange={(checked) => props.onChange({ ...current(), enabled: checked })}
      >
        {m.settings_game_enable_tracking({ title: props.title })}
      </Switch>
      <Show when={current().enabled}>
        <TextField
          label={m.settings_game_uid()}
          value={current().uid}
          onChange={(value) => props.onChange({ ...current(), uid: value })}
          placeholder={m.settings_game_uid_placeholder()}
        />
        <Switch
          checked={current().auto_claim_daily_rewards}
          onChange={(checked) =>
            props.onChange({ ...current(), auto_claim_daily_rewards: checked })
          }
        >
          {m.settings_game_auto_claim()}
        </Switch>
        <NotificationSection
          gameId={props.gameId}
          resourceTypes={props.resourceTypes}
          notifications={props.config?.notifications}
          resourceLimits={props.resourceLimits}
          onChange={(notifications) => props.onChange({ ...current(), notifications })}
        />
      </Show>
    </SettingsCard>
  );
};
