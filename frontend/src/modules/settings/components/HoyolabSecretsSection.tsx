import { Section } from "@/modules/settings/components/Section";
import type { HoyolabSecrets } from "@/modules/settings/settings.types";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

interface HoyolabSecretsSectionProps {
  secrets: HoyolabSecrets;
  onChange: (secrets: HoyolabSecrets) => void;
}

export const HoyolabSecretsSection: React.FC<HoyolabSecretsSectionProps> = ({
  secrets,
  onChange,
}) => {
  return (
    <Section
      title={m.settings_hoyolab_title()}
      description={m.settings_hoyolab_description()}
    >
      <TextField
        label="ltuid_v2"
        type="password"
        value={secrets.ltuid_v2}
        onChange={(value) =>
          onChange({
            ...secrets,
            ltuid_v2: value,
          })
        }
        placeholder={m.settings_hoyolab_ltuid_placeholder()}
      />
      <TextField
        label="ltoken_v2"
        type="password"
        value={secrets.ltoken_v2}
        onChange={(value) =>
          onChange({
            ...secrets,
            ltoken_v2: value,
          })
        }
        placeholder={m.settings_hoyolab_ltoken_placeholder()}
      />
      <TextField
        label="ltmid_v2"
        type="password"
        value={secrets.ltmid_v2}
        onChange={(value) =>
          onChange({
            ...secrets,
            ltmid_v2: value,
          })
        }
        placeholder={m.settings_hoyolab_ltmid_placeholder()}
      />
    </Section>
  );
};
