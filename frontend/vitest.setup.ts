import { cleanup } from "@solidjs/testing-library";
import { afterEach } from "vite-plus/test";

// @solidjs/testing-library only self-registers cleanup when `afterEach` is a
// global, which it is not without `globals: true`. Without this every render
// stays mounted for the rest of the file and `screen` queries the leftovers.
afterEach(cleanup);
