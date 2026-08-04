import { Show, type VoidComponent } from "solid-js";
import type { GameId } from "@/modules/games/games.types";
import type { ResourceLimits } from "@/modules/settings/components/NotificationResourceRow";
import { NotificationSection } from "@/modules/settings/components/NotificationSection";
import { Section } from "@/modules/settings/components/Section";
import type { HoyolabGameConfig } from "@/modules/settings/settings.types";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

export interface HoyolabGameSectionProps {
  title: string;
  description: string;
  gameId: GameId;
  resourceTypes: readonly string[];
  config: HoyolabGameConfig | undefined;
  resourceLimits?: Partial<Record<string, ResourceLimits>>;
  onChange: (config: HoyolabGameConfig) => void;
}

export const HoyolabGameSection: VoidComponent<HoyolabGameSectionProps> = (props) => {
  const enabled = () => props.config?.enabled ?? false;
  const uid = () => props.config?.uid ?? "";
  const autoClaimDailyRewards = () => props.config?.auto_claim_daily_rewards ?? false;

  return (
    <Section title={props.title} description={props.description}>
      <Switch
        checked={enabled()}
        onChange={(checked) =>
          props.onChange({
            ...props.config,
            enabled: checked,
            uid: uid(),
            auto_claim_daily_rewards: autoClaimDailyRewards(),
          })
        }
      >
        {m.settings_game_enable_tracking({ title: props.title })}
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
              auto_claim_daily_rewards: autoClaimDailyRewards(),
            })
          }
          placeholder={m.settings_game_uid_placeholder()}
        />
        <Switch
          checked={autoClaimDailyRewards()}
          onChange={(checked) =>
            props.onChange({
              ...props.config,
              enabled: enabled(),
              uid: uid(),
              auto_claim_daily_rewards: checked,
            })
          }
        >
          {m.settings_game_auto_claim()}
        </Switch>
        <NotificationSection
          gameId={props.gameId}
          resourceTypes={props.resourceTypes}
          notifications={props.config?.notifications}
          resourceLimits={props.resourceLimits}
          onChange={(notifications) =>
            props.onChange({
              ...props.config,
              enabled: enabled(),
              uid: uid(),
              auto_claim_daily_rewards: autoClaimDailyRewards(),
              notifications,
            })
          }
        />
      </Show>
    </Section>
  );
};
