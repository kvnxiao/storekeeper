import * as m from "@/paraglide/messages";

/** Resource types that are stamina-based (support value-threshold notifications) */
export const STAMINA_RESOURCE_TYPES: ReadonlySet<string> = new Set([
  "resin",
  "realm_currency",
  "trailblaze_power",
  "battery",
  "waveplates",
]);

/** Returns the localized display name for a resource type, evaluated at call time */
export function getResourceDisplayName(type: string): string {
  const names: Record<string, () => string> = {
    resin: m.resource_resin,
    parametric_transformer: m.resource_parametric_transformer,
    realm_currency: m.resource_realm_currency,
    expeditions: m.resource_expeditions,
    trailblaze_power: m.resource_trailblaze_power,
    battery: m.resource_battery,
    waveplates: m.resource_waveplates,
  };
  return names[type]?.() ?? type;
}
