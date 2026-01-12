import { Section } from "@/modules/settings/components/Section";
import type { HoyolabSecrets } from "@/modules/settings/settings.types";
import { TextField } from "@/modules/ui/components/TextField";

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
      title="HoYoLab Authentication"
      description="Enter your HoYoLab cookies for Genshin Impact, Honkai: Star Rail, and Zenless Zone Zero. Get these from the HoYoLab website developer tools."
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
        placeholder="Your ltuid_v2 cookie value"
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
        placeholder="Your ltoken_v2 cookie value"
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
        placeholder="Your ltmid_v2 cookie value"
      />
    </Section>
  );
};
