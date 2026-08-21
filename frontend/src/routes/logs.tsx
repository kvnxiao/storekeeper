import { createFileRoute } from "@tanstack/solid-router";
import { LogsPage } from "@/modules/logs/components/LogsPage";

export const Route = createFileRoute("/logs")({
  component: LogsPage,
});
