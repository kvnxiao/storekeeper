import { createFileRoute } from "@tanstack/solid-router";
import { DashboardPage } from "@/modules/dashboard/components/DashboardPage";

export const Route = createFileRoute("/")({
  component: DashboardPage,
});
