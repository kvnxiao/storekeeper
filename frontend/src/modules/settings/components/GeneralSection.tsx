import type { VoidComponent } from "solid-js";
import { LOCALE_ENDONYMS } from "@/modules/i18n/i18n.constants";
import { Section } from "@/modules/settings/components/Section";
import { openConfigFolder } from "@/modules/settings/settings.query";
import type { GeneralConfig } from "@/modules/settings/settings.types";
import { Button } from "@/modules/ui/components/Button";
import { NumberField } from "@/modules/ui/components/NumberField";
import { Select } from "@/modules/ui/components/Select";
import { Switch } from "@/modules/ui/components/Switch";
import * as m from "@/paraglide/messages";

export interface GeneralSectionProps {
  config: GeneralConfig;
  onChange: (config: GeneralConfig) => void;
}

// Evaluated at call time so labels follow the active locale
const languageOptions = () => [
  { id: "auto", label: m.settings_general_language_system_default() },
  ...Object.entries(LOCALE_ENDONYMS).map(([code, name]) => ({ id: code, label: name })),
];

const logLevelOptions = () => [
  { id: "error", label: m.settings_general_log_error() },
  { id: "warn", label: m.settings_general_log_warning() },
  { id: "info", label: m.settings_general_log_info() },
  { id: "debug", label: m.settings_general_log_debug() },
  { id: "trace", label: m.settings_general_log_trace() },
];

export const GeneralSection: VoidComponent<GeneralSectionProps> = (props) => {
  return (
    <Section title={m.settings_general_title()} description={m.settings_general_description()}>
      <NumberField
        label={m.settings_general_poll_interval()}
        description={m.settings_general_poll_description()}
        value={props.config.poll_interval_secs}
        onChange={(value) =>
          props.onChange({
            ...props.config,
            poll_interval_secs: value,
          })
        }
        minValue={60}
        maxValue={3600}
        step={60}
      />
      <Switch
        checked={props.config.start_minimized}
        onChange={(checked) =>
          props.onChange({
            ...props.config,
            start_minimized: checked,
          })
        }
      >
        {m.settings_general_start_minimized()}
      </Switch>
      <Switch
        checked={props.config.autostart}
        onChange={(checked) =>
          props.onChange({
            ...props.config,
            autostart: checked,
          })
        }
      >
        {m.settings_general_autostart()}
      </Switch>
      <Select
        label={m.settings_general_language()}
        value={props.config.language ?? "auto"}
        onChange={(value) =>
          props.onChange({
            ...props.config,
            language: value === "auto" ? null : value,
          })
        }
        options={languageOptions()}
      />
      <Select
        label={m.settings_general_log_level()}
        value={props.config.log_level}
        onChange={(value) =>
          props.onChange({
            ...props.config,
            log_level: value,
          })
        }
        options={logLevelOptions()}
      />
      <Button color="light" onClick={() => openConfigFolder()}>
        {m.settings_general_open_config()}
      </Button>
    </Section>
  );
};
