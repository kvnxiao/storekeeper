import { Section } from "@/modules/settings/components/Section";
import type { KuroSecrets } from "@/modules/settings/settings.types";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

interface KuroSecretsSectionProps {
  secrets: KuroSecrets;
  onChange: (secrets: KuroSecrets) => void;
}

export const KuroSecretsSection: React.FC<KuroSecretsSectionProps> = ({
  secrets,
  onChange,
}) => {
  return (
    <Section
      title={m.settings_kuro_title()}
      description={m.settings_kuro_description()}
    >
      <TextField
        label={m.settings_kuro_oauth_label()}
        type="password"
        value={secrets.oauth_code}
        onChange={(value) =>
          onChange({
            ...secrets,
            oauth_code: value,
          })
        }
        placeholder={m.settings_kuro_oauth_placeholder()}
      />
    </Section>
  );
};
