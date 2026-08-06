import { paraglideVitePlugin } from "@inlang/paraglide-js";
import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/solid-start/plugin/vite";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite-plus";
import viteSolid from "vite-plugin-solid";

type TanStackStartInputConfig = NonNullable<Parameters<typeof tanstackStart>[0]>;
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

export default defineConfig({
  plugins: [
    paraglideVitePlugin({
      project: "./project.inlang",
      outdir: "./src/paraglide",
      strategy: ["localStorage", "baseLocale"],
    }),
    devtools(),
    nitro(),
    tailwindcss(),
    tanstackStart({
      spa: spaWithPrerenderOptions,
    }),
    viteSolid({ ssr: true }),
  ],

  clearScreen: false,
  resolve: {
    tsconfigPaths: true,
  },
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

  lint: {
    plugins: ["import", "typescript", "unicorn", "jsx-a11y"],
    categories: {
      correctness: "error",
      suspicious: "error",
    },
    options: {
      // typeCheck covers what a separate `tsc --noEmit` pass would.
      typeAware: true,
      typeCheck: true,
    },
    rules: {
      "typescript/no-floating-promises": "error",
      "typescript/no-misused-promises": "error",
      "typescript/await-thenable": "error",
      "typescript/no-unnecessary-condition": "error",
      "typescript/require-array-sort-compare": "error",
      "typescript/switch-exhaustiveness-check": "error",
      "typescript/prefer-regexp-exec": "error",
      // Fires on intentional narrowing casts.
      "typescript/no-unsafe-type-assertion": "off",
      // Shadowing children/className/cn in component render props is idiomatic
      // here.
      "no-shadow": "off",
      // Polyfill / font / side-effect imports are intentional.
      "import/no-unassigned-import": "off",
      "import/no-cycle": "error",
      curly: ["error", "all"],
      "unicorn/prefer-array-find": "error",
    },
    ignorePatterns: [
      "src/routeTree.gen.ts",
      "src/paraglide/**",
      "dist/**",
      ".output/**",
      ".tanstack/**",
    ],
  },

  // Oxfmt defaults (2-space indent, double quotes) are what this repo wants,
  // so only ignores are set here.
  fmt: {
    ignorePatterns: [
      "src/routeTree.gen.ts",
      "src/paraglide/**",
      "dist/**",
      ".output/**",
      ".tanstack/**",
    ],
  },
});
