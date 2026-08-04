import { defineConfig } from "vite-plus";
import viteSolid from "vite-plugin-solid";

export default defineConfig({
  // hot: false — solid-refresh's virtual module breaks the test module runner
  plugins: [viteSolid({ hot: false })],
  resolve: {
    tsconfigPaths: true,
  },
  test: {
    environment: "jsdom",
  },
});
