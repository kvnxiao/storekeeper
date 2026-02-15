import { invoke } from "@tauri-apps/api/core";
import { Section } from "@/modules/settings/components/Section";
import type { GeneralConfig } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { NumberField } from "@/modules/ui/components/NumberField";
import { Select, SelectItem } from "@/modules/ui/components/Select";
import { Switch } from "@/modules/ui/components/Switch";

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
      title="General"
      description="Application-wide settings for polling and startup behavior."
    >
      <NumberField
        label="Poll Interval (seconds)"
        description="How often to fetch resource data from game APIs."
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
        Start minimized to system tray
      </Switch>
      <Select
        label="Log Level"
        selectedKey={config.log_level}
        onSelectionChange={(key) =>
          onChange({
            ...config,
            log_level: String(key),
          })
        }
      >
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="error">Error</SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="warn">Warning</SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="info">Info</SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="debug">Debug</SelectItem>
        {/* biome-ignore lint/correctness/useUniqueElementIds: React Aria SelectItem id is a key, not a DOM id */}
        <SelectItem id="trace">Trace</SelectItem>
      </Select>
      <Button
        variant="solid"
        color="light"
        onPress={() => invoke("open_config_folder")}
      >
        Open Config Folder
      </Button>
    </Section>
  );
};
