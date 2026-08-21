import ExternalLink from "lucide-solid/icons/external-link";
import FolderOpen from "lucide-solid/icons/folder-open";
import type { VoidComponent } from "solid-js";
import { openLogFolder, openLogsWindow } from "@/modules/logs/logs.query";
import { logLevelOptions } from "@/modules/logs/logs.utils";
import { SettingsCard } from "@/modules/settings/components/SettingsCard";
import type { GeneralConfig } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { Select } from "@/modules/ui/components/Select";
import * as m from "@/paraglide/messages";

export interface LogsSectionProps {
  config: GeneralConfig;
  onChange: (config: GeneralConfig) => void;
}

export const LogsSection: VoidComponent<LogsSectionProps> = (props) => {
  return (
    <SettingsCard title={m.settings_logs_title()} description={m.settings_logs_description()}>
      <Select
        label={m.settings_logs_level()}
        value={props.config.log_level}
        onChange={(value) =>
          props.onChange({
            ...props.config,
            log_level: value,
          })
        }
        options={logLevelOptions()}
      />
      <div class="flex flex-wrap gap-2">
        <Button color="light" onClick={() => openLogFolder()}>
          <FolderOpen aria-hidden="true" class="size-4" />
          {m.settings_logs_open_folder()}
        </Button>
        <Button color="light" onClick={() => openLogsWindow()}>
          <ExternalLink aria-hidden="true" class="size-4" />
          {m.settings_logs_view()}
        </Button>
      </div>
    </SettingsCard>
  );
};
