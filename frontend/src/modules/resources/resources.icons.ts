/**
 * Maps resource type strings to their icon paths.
 * Type strings come from the Rust backend (snake_case JSON serialization).
 * `null` means no icon is available - use placeholder instead.
 */
export const RESOURCE_ICONS: Record<string, string | null> = {
  // Genshin Impact
  resin: "/icons/game/genshin/Item_Original_Resin.webp",
  parametric_transformer:
    "/icons/game/genshin/Item_Parametric_Transformer.webp",
  realm_currency: "/icons/game/genshin/Item_Realm_Currency.webp",
  expeditions: "/icons/game/genshin/Expeditions.webp",

  // Honkai: Star Rail
  trailblaze_power: "/icons/game/hsr/Item_Trailblaze_Power.webp",

  // Zenless Zone Zero
  battery: "/icons/game/zzz/Item_Battery_Charge.webp",

  // Wuthering Waves
  waveplates: "/icons/game/wuwa/Item_Waveplate.webp",
};
