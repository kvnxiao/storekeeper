import * as m from "@/paraglide/messages";

// =============================================================================
// Resource type constants per game
// =============================================================================

export const GenshinResource = {
  Resin: "resin",
  ParametricTransformer: "parametric_transformer",
  RealmCurrency: "realm_currency",
  Expeditions: "expeditions",
} as const;

export const HsrResource = {
  TrailblazePower: "trailblaze_power",
} as const;

export const ZzzResource = {
  Battery: "battery",
} as const;

export const WuwaResource = {
  Waveplates: "waveplates",
} as const;

/** Extracted resource type unions per game */
export type GenshinResourceType =
  (typeof GenshinResource)[keyof typeof GenshinResource];
export type HsrResourceType = (typeof HsrResource)[keyof typeof HsrResource];
export type ZzzResourceType = (typeof ZzzResource)[keyof typeof ZzzResource];
export type WuwaResourceType = (typeof WuwaResource)[keyof typeof WuwaResource];

/** Resource types that are stamina-based (support value-threshold notifications) */
export const STAMINA_RESOURCE_TYPES: ReadonlySet<string> = new Set([
  GenshinResource.Resin,
  GenshinResource.RealmCurrency,
  HsrResource.TrailblazePower,
  ZzzResource.Battery,
  WuwaResource.Waveplates,
]);

/** Returns the localized display name for a resource type, evaluated at call time */
export function getResourceDisplayName(type: string): string {
  const names: Record<string, () => string> = {
    [GenshinResource.Resin]: m.resource_resin,
    [GenshinResource.ParametricTransformer]: m.resource_parametric_transformer,
    [GenshinResource.RealmCurrency]: m.resource_realm_currency,
    [GenshinResource.Expeditions]: m.resource_expeditions,
    [HsrResource.TrailblazePower]: m.resource_trailblaze_power,
    [ZzzResource.Battery]: m.resource_battery,
    [WuwaResource.Waveplates]: m.resource_waveplates,
  };
  return names[type]?.() ?? type;
}
