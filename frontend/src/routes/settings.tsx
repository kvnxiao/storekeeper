import { createFileRoute, Link } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { useCallback } from "react";
import { Button } from "react-aria-components";

const SettingsPage: React.FC = () => {
  const handleOpenConfigFolder = useCallback(async () => {
    try {
      await invoke("open_config_folder");
    } catch (e) {
      console.error("Failed to open config folder:", e);
    }
  }, []);

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-4">
      <header className="mb-6">
        <div className="flex items-center gap-4">
          <Link
            to="/"
            className="text-blue-600 dark:text-blue-400 hover:underline"
          >
            &larr; Back
          </Link>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">
            Settings
          </h1>
        </div>
      </header>

      <main className="space-y-6">
        <section className="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
            Configuration
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            Edit your game credentials and settings in the configuration files.
          </p>
          <Button
            onPress={handleOpenConfigFolder}
            className="px-4 py-2 bg-gray-600 hover:bg-gray-700 pressed:bg-gray-800 text-white rounded-lg text-sm font-medium transition-colors cursor-pointer"
          >
            Open Config Folder
          </Button>
        </section>

        <section className="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-4">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
            About
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-400">
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
