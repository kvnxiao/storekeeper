import type { VoidComponent } from "solid-js";
import { Section } from "@/modules/settings/components/Section";
import type { HoyolabSecrets } from "@/modules/settings/settings.types";
import { TextField } from "@/modules/ui/components/TextField";
import * as m from "@/paraglide/messages";

export interface HoyolabSecretsSectionProps {
  secrets: HoyolabSecrets;
  onChange: (secrets: HoyolabSecrets) => void;
}

export const HoyolabSecretsSection: VoidComponent<HoyolabSecretsSectionProps> = (props) => {
  return (
    <Section title={m.settings_hoyolab_title()} description={m.settings_hoyolab_description()}>
      <TextField
        label="ltuid_v2"
        type="password"
        value={props.secrets.ltuid_v2}
        onChange={(value) =>
          props.onChange({
            ...props.secrets,
            ltuid_v2: value,
          })
        }
        placeholder={m.settings_hoyolab_ltuid_placeholder()}
      />
      <TextField
        label="ltoken_v2"
        type="password"
        value={props.secrets.ltoken_v2}
        onChange={(value) =>
          props.onChange({
            ...props.secrets,
            ltoken_v2: value,
          })
        }
        placeholder={m.settings_hoyolab_ltoken_placeholder()}
      />
      <TextField
        label="ltmid_v2"
        type="password"
        value={props.secrets.ltmid_v2}
        onChange={(value) =>
          props.onChange({
            ...props.secrets,
            ltmid_v2: value,
          })
        }
        placeholder={m.settings_hoyolab_ltmid_placeholder()}
      />
    </Section>
  );
};
