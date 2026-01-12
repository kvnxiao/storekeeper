import { Section } from "@/modules/settings/components/Section";
import type { WuwaConfig } from "@/modules/settings/settings.types";
import { Switch } from "@/modules/ui/components/Switch";
import { TextField } from "@/modules/ui/components/TextField";

interface WuwaSectionProps {
  config: WuwaConfig | undefined;
  onChange: (config: WuwaConfig) => void;
}

export const WuwaSection: React.FC<WuwaSectionProps> = ({
  config,
  onChange,
}) => {
  const enabled = config?.enabled ?? false;
  const playerId = config?.player_id ?? "";

  return (
    <Section
      title="Wuthering Waves"
      description="Configure your Wuthering Waves account."
    >
      <Switch
        isSelected={enabled}
        onChange={(isSelected) =>
          onChange({
            enabled: isSelected,
            player_id: playerId,
          })
        }
      >
        Enable Wuthering Waves tracking
      </Switch>
      {enabled && (
        <TextField
          label="Player ID"
          value={playerId}
          onChange={(value) =>
            onChange({
              enabled,
              player_id: value,
            })
          }
          placeholder="Enter your Player ID"
        />
      )}
    </Section>
  );
};
