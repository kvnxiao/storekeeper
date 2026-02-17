import { invoke } from "@tauri-apps/api/core";
import { Section } from "@/modules/settings/components/Section";
import type { GeneralConfig } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { NumberField } from "@/modules/ui/components/NumberField";
import { Select, SelectItem } from "@/modules/ui/components/Select";
import { Switch } from "@/modules/ui/components/Switch";
import * as m from "@/paraglide/messages";

interface GeneralSectionProps {
  config: GeneralConfig;
  onChange: (config: GeneralConfig) => void;
}

export const GeneralSection: React.FC<GeneralSectionProps> = ({
  config,
  onChange,
}) => {
  return (
    <Section
      title={m.settings_general_title()}
      description={m.settings_general_description()}
    >
      <NumberField
        label={m.settings_general_poll_interval()}
        description={m.settings_general_poll_description()}
        value={config.poll_interval_secs}
        onChange={(value) =>
          onChange({
            ...config,
            poll_interval_secs: value,
          })
        }
        minValue={60}
        maxValue={3600}
        step={60}
      />
      <Switch
        isSelected={config.start_minimized}
        onChange={(isSelected) =>
          onChange({
            ...config,
            start_minimized: isSelected,
          })
        }
      >
        {m.settings_general_start_minimized()}
      </Switch>
      <Switch
        isSelected={config.autostart}
        onChange={(isSelected) =>
          onChange({
            ...config,
            autostart: isSelected,
          })
        }
      >
        {m.settings_general_autostart()}
      </Switch>
      <Select
        label={m.settings_general_language()}
        value={config.language ?? "auto"}
        onChange={(value) =>
          onChange({
            ...config,
            language: value === "auto" ? null : String(value),
          })
        }
      >
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="auto">
          {m.settings_general_language_system_default()}
        </SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="en">{m.settings_general_language_english()}</SelectItem>
      </Select>
      <Select
        label={m.settings_general_log_level()}
        value={config.log_level}
        onChange={(value) =>
          onChange({
            ...config,
            log_level: String(value),
          })
        }
      >
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="error">{m.settings_general_log_error()}</SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="warn">{m.settings_general_log_warning()}</SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="info">{m.settings_general_log_info()}</SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="debug">{m.settings_general_log_debug()}</SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="trace">{m.settings_general_log_trace()}</SelectItem>
      </Select>
      <Button color="light" onPress={() => invoke("open_config_folder")}>
        {m.settings_general_open_config()}
      </Button>
    </Section>
  );
};
