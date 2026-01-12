import { Section } from "@/modules/settings/components/Section";
import type { KuroSecrets } from "@/modules/settings/settings.types";
import { TextField } from "@/modules/ui/components/TextField";

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
      title="Kuro Games Authentication"
      description="For Wuthering Waves. The oauth_code is automatically loaded from the Kuro SDK launcher cache. Only set this if you need to override the automatic detection."
    >
      <TextField
        label="OAuth Code (Optional Override)"
        type="password"
        value={secrets.oauth_code}
        onChange={(value) =>
          onChange({
            ...secrets,
            oauth_code: value,
          })
        }
        placeholder="Leave empty to use automatic detection"
      />
    </Section>
  );
};
