import * as m from "@/paraglide/messages";

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
export type GenshinResourceType = (typeof GenshinResource)[keyof typeof GenshinResource];
export type HsrResourceType = (typeof HsrResource)[keyof typeof HsrResource];
export type ZzzResourceType = (typeof ZzzResource)[keyof typeof ZzzResource];
export type WuwaResourceType = (typeof WuwaResource)[keyof typeof WuwaResource];

export type ResourceType =
  | GenshinResourceType
  | HsrResourceType
  | ZzzResourceType
  | WuwaResourceType;

/** Resource types that are stamina-based (support value-threshold notifications) */
export const STAMINA_RESOURCE_TYPES: ReadonlySet<ResourceType> = new Set([
  GenshinResource.Resin,
  GenshinResource.RealmCurrency,
  HsrResource.TrailblazePower,
  ZzzResource.Battery,
  WuwaResource.Waveplates,
]);

const RESOURCE_ICON_PATHS: Record<ResourceType, string> = {
  [GenshinResource.Resin]: "/icons/game/genshin/Item_Original_Resin.webp",
  [GenshinResource.ParametricTransformer]: "/icons/game/genshin/Item_Parametric_Transformer.webp",
  [GenshinResource.RealmCurrency]: "/icons/game/genshin/Item_Realm_Currency.webp",
  [GenshinResource.Expeditions]: "/icons/game/genshin/Expeditions.webp",
  [HsrResource.TrailblazePower]: "/icons/game/hsr/Item_Trailblaze_Power.webp",
  [ZzzResource.Battery]: "/icons/game/zzz/Item_Battery_Charge.webp",
  [WuwaResource.Waveplates]: "/icons/game/wuwa/Item_Waveplate.webp",
};

/** Returns the icon asset path for a resource type. */
export function getResourceIconPath(type: ResourceType): string {
  return RESOURCE_ICON_PATHS[type];
}

const RESOURCE_DISPLAY_NAMES: Record<ResourceType, () => string> = {
  [GenshinResource.Resin]: m.resource_resin,
  [GenshinResource.ParametricTransformer]: m.resource_parametric_transformer,
  [GenshinResource.RealmCurrency]: m.resource_realm_currency,
  [GenshinResource.Expeditions]: m.resource_expeditions,
  [HsrResource.TrailblazePower]: m.resource_trailblaze_power,
  [ZzzResource.Battery]: m.resource_battery,
  [WuwaResource.Waveplates]: m.resource_waveplates,
};

/** Returns the localized display name for a resource type, evaluated at call time */
export function getResourceDisplayName(type: ResourceType): string {
  return RESOURCE_DISPLAY_NAMES[type]();
}
