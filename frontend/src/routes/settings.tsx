import { createFileRoute } from "@tanstack/solid-router";
import { SettingsPage } from "@/modules/settings/components/SettingsPage";

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});
