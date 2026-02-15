import { atomWithQuery } from "jotai-tanstack-query";
import { configQueryOptions } from "@/modules/settings/settings.query";

// =============================================================================
// Config Query Atom (app-wide state)
// =============================================================================

/** Fetch config from backend - used by CoreAtoms and SettingsAtoms */
export const configQueryAtom = atomWithQuery(() => configQueryOptions());
