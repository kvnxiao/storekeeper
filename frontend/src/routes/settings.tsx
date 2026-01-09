import { createFileRoute, Link } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { useCallback } from "react";
import { Button } from "@/components/ui";

const SettingsPage: React.FC = () => {
  const handleOpenConfigFolder = useCallback(async () => {
    try {
      await invoke("open_config_folder");
    } catch (e) {
      console.error("Failed to open config folder:", e);
    }
  }, []);

  return (
    <div className="min-h-screen bg-background p-4">
      <header className="mb-6">
        <div className="flex items-center gap-4">
          <Link to="/" className="text-primary hover:underline">
            &larr; Back
          </Link>
          <h1 className="text-xl font-bold text-foreground">Settings</h1>
        </div>
      </header>

      <main className="space-y-6">
        <section className="rounded-lg border border-border bg-card p-4 shadow-sm">
          <h2 className="mb-3 text-lg font-semibold text-card-foreground">
            Configuration
          </h2>
          <p className="mb-4 text-sm text-muted-foreground">
            Edit your game credentials and settings in the configuration files.
          </p>
          <Button onPress={handleOpenConfigFolder}>Open Config Folder</Button>
        </section>

        <section className="rounded-lg border border-border bg-card p-4 shadow-sm">
          <h2 className="mb-3 text-lg font-semibold text-card-foreground">
            About
          </h2>
          <p className="text-sm text-muted-foreground">
            Storekeeper tracks stamina resources for gacha games including
            Genshin Impact, Honkai: Star Rail, Zenless Zone Zero, and Wuthering
            Waves.
          </p>
        </section>
      </main>
    </div>
  );
};

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});
