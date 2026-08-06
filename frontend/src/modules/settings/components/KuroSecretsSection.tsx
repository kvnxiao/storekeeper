import type { VoidComponent } from "solid-js";
import { SettingsCard } from "@/modules/settings/components/SettingsCard";
import type { KuroSecrets } from "@/modules/settings/settings.types";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

export interface KuroSecretsSectionProps {
  secrets: KuroSecrets;
  onChange: (secrets: KuroSecrets) => void;
}

export const KuroSecretsSection: VoidComponent<KuroSecretsSectionProps> = (props) => {
  return (
    <SettingsCard title={m.settings_kuro_title()} description={m.settings_kuro_description()}>
      <TextField
        label={m.settings_kuro_oauth_label()}
        type="password"
        value={props.secrets.oauth_code}
        onChange={(value) =>
          props.onChange({
            ...props.secrets,
            oauth_code: value,
          })
        }
        placeholder={m.settings_kuro_oauth_placeholder()}
      />
    </SettingsCard>
  );
};
