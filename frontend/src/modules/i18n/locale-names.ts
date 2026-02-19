/**
 * Static mapping of locale codes to their endonyms (native display names).
 *
 * Endonyms are NOT localized — "English" is always "English", "日本語" is
 * always "日本語", regardless of the active UI locale. They must therefore
 * live in code, not in the message JSON files.
 */
export const LOCALE_ENDONYMS: Record<string, string> = {
  en: "English",
  "zh-CN": "简体中文",
  ko: "한국어",
  ja: "日本語",
};
