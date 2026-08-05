// vite-plus-test 0.1.24 (the aliased vitest) bundles @voidzero-dev/vite-plus-core
// 0.1.24, whose napi loader resolves its native binding via require("vite-plus/binding")
// without enforcing a version match. vite-plus >= 0.2.6 bindings drop enum variants
// that core 0.1.24 still constructs (builtin:vite-wasm-fallback), so vitest crashes
// at startup with InvalidArg. Nesting a binding-compatible vite-plus under that core
// lets the rest of the tree stay on the latest delegate. Remove once vite-plus-test
// ships a release paired with the 0.2.x core line.
module.exports = {
  hooks: {
    readPackage(pkg) {
      if (pkg.name === "@voidzero-dev/vite-plus-core" && pkg.version === "0.1.24") {
        pkg.dependencies = { ...pkg.dependencies, "vite-plus": "0.2.1" };
      }
      return pkg;
    },
  },
};
