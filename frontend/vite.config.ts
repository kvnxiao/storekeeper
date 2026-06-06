import { paraglideVitePlugin } from "@inlang/paraglide-js";
import tailwindcss from "@tailwindcss/vite";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite-plus";

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
    viteReact(),
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

  // Oxlint: mirrors the former biome.json ruleset, plus type-aware rules
  // (tsgolint) and a layer of high-value extras with no Biome analog.
  lint: {
    // NB: react-hooks rules live under the `react` plugin; there is no
    // separate `react-hooks` plugin to enable.
    plugins: ["react", "import", "typescript", "unicorn", "jsx-a11y"],
    categories: {
      correctness: "error",
      suspicious: "error",
    },
    options: {
      // type-aware rules below + full type-check (replaces `tsc --noEmit`)
      typeAware: true,
      typeCheck: true,
    },
    rules: {
      // Type-aware (was biome `nursery`)
      "typescript/no-floating-promises": "error",
      "typescript/no-misused-promises": "error",
      "typescript/await-thenable": "error",
      "typescript/no-unnecessary-condition": "error",
      "typescript/require-array-sort-compare": "error",
      "typescript/switch-exhaustiveness-check": "error",
      "typescript/prefer-regexp-exec": "error",
      // React (was biome `useHookAtTopLevel` + react domain). Hooks rules are
      // namespaced under the `react` plugin in oxlint.
      "react/rules-of-hooks": "error",
      "react/exhaustive-deps": "warn",
      // Automatic JSX runtime (react-jsx): the classic-runtime rule is noise.
      "react/react-in-jsx-scope": "off",
      // Opinionated, not enforced under biome; flags intentional narrowing casts.
      "typescript/no-unsafe-type-assertion": "off",
      // Noisy, not enforced under biome: shadowing children/className/cn in
      // component render props is idiomatic here.
      "no-shadow": "off",
      // Polyfill / font / side-effect imports are intentional.
      "import/no-unassigned-import": "off",
      // was biome `suspicious/noImportCycles`
      "import/no-cycle": "error",
      // was biome `style/useBlockStatements`
      curly: ["error", "all"],
      // was biome `nursery/useFind`
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

  // Oxfmt: Prettier-compatible. Defaults (2-space indent, double quotes)
  // already match the former biome formatter, so only ignores are set here.
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
