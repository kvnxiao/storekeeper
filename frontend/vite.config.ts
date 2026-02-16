import { paraglideVitePlugin } from "@inlang/paraglide-js";
import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";
import viteTsConfigPaths from "vite-tsconfig-paths";

type TanStackStartInputConfig = NonNullable<
  Parameters<typeof tanstackStart>[0]
>;
type SpaOptions = NonNullable<TanStackStartInputConfig["spa"]>;

const host = process.env.TAURI_DEV_HOST;

// SPA prerender options for Tauri desktop app
const spaWithPrerenderOptions: SpaOptions = {
  prerender: {
    enabled: true,
    autoSubfolderIndex: true,
    outputPath: "/index.html",
    crawlLinks: false,
    retryCount: 0,
  },
};

export default defineConfig(async () => ({
  plugins: [
    paraglideVitePlugin({
      project: "./project.inlang",
      outdir: "./src/paraglide",
      strategy: ["localStorage", "baseLocale"],
    }),
    devtools(),
    nitro(),
    viteTsConfigPaths({
      projects: ["./tsconfig.json"],
    }),
    tailwindcss(),
    tanstackStart({
      spa: spaWithPrerenderOptions,
    }),
    viteReact(),
  ],

  clearScreen: false,
  server: {
    port: 3000,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 3001,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
