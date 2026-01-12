import { Section } from "@/modules/settings/components/Section";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";

interface HoyolabGameConfig {
  enabled: boolean;
  uid: string;
  auto_claim_daily_rewards: boolean;
}

interface HoyolabGameSectionProps {
  title: string;
  description: string;
  config: HoyolabGameConfig | undefined;
  onChange: (config: HoyolabGameConfig) => void;
}

export const HoyolabGameSection: React.FC<HoyolabGameSectionProps> = ({
  title,
  description,
  config,
  onChange,
}) => {
  const enabled = config?.enabled ?? false;
  const uid = config?.uid ?? "";
  const autoClaimDailyRewards = config?.auto_claim_daily_rewards ?? false;

  return (
    <Section title={title} description={description}>
      <Switch
        isSelected={enabled}
        onChange={(isSelected) =>
          onChange({
            enabled: isSelected,
            uid,
            auto_claim_daily_rewards: autoClaimDailyRewards,
          })
        }
      >
        Enable {title} tracking
      </Switch>
      {enabled && (
        <>
          <TextField
            label="UID"
            value={uid}
            onChange={(value) =>
              onChange({
                enabled,
                uid: value,
                auto_claim_daily_rewards: autoClaimDailyRewards,
              })
            }
            placeholder="Enter your UID"
          />
          <Switch
            isSelected={autoClaimDailyRewards}
            onChange={(isSelected) =>
              onChange({
                enabled,
                uid,
                auto_claim_daily_rewards: isSelected,
              })
            }
          >
            Auto-claim daily rewards
          </Switch>
        </>
      )}
    </Section>
  );
};
