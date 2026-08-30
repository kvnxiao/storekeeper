import { Show, type VoidComponent } from "solid-js";
import type { ResourceType } from "@/modules/games/games.constants";
import type { GameId } from "@/modules/games/games.types";
import type { ResourceLimits } from "@/modules/resources/resources.types";
import { NotificationSection } from "@/modules/settings/components/NotificationSection";
import { SettingsCard } from "@/modules/settings/components/SettingsCard";
import { DetectedRegion } from "@/modules/settings/components/DetectedRegion";
import type { WuwaConfig } from "@/modules/settings/settings.types";
import { emptyWuwaConfig } from "@/modules/settings/settings.utils";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

export interface WuwaGameSectionProps {
  title: string;
  description: string;
  gameId: GameId;
  resourceTypes: readonly ResourceType[];
  config: WuwaConfig | null;
  resourceLimits?: Partial<Record<ResourceType, ResourceLimits>>;
  onChange: (config: WuwaConfig) => void;
}

export const WuwaGameSection: VoidComponent<WuwaGameSectionProps> = (props) => {
  const current = (): WuwaConfig => ({
    ...emptyWuwaConfig(props.resourceTypes),
    ...props.config,
  });

  return (
    <SettingsCard title={props.title} description={props.description}>
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
        <DetectedRegion gameId={props.gameId} uid={current().uid} />
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
