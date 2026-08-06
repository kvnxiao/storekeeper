import { cleanup, configure } from "@solidjs/testing-library";
import { afterEach } from "vite-plus/test";

// @solidjs/testing-library only self-registers cleanup when `afterEach` is a
// global, which it is not without `globals: true`. Without this every render
// stays mounted for the rest of the file and `screen` queries the leftovers.
afterEach(cleanup);

// The 1s default expires on a loaded machine once the whole suite runs in
// parallel, which fails a test that would have passed. Stay below the 5s test
// timeout so a genuine hang still reports the query that never matched.
// `vi.waitFor` reads none of this and takes the same budget per call site.
configure({ asyncUtilTimeout: 3_000 });
