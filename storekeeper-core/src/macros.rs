//! Macros for reducing boilerplate in game resource definitions.

/// Macro for defining game resource enums with consistent structure.
///
/// This macro generates:
/// - An enum with the specified variants
/// - Serde serialization with `{ type, data }` tagged format
/// - Implementation of `DisplayableResource` trait
/// - Standard derives (Debug, Clone, Serialize, Deserialize)
///
/// Optionally, with `/ TypeName` syntax, also generates a companion type-only enum.
///
/// # Example
///
/// ```rust,ignore
/// use storekeeper_core::{game_resource_enum, StaminaResource, CooldownResource};
///
/// game_resource_enum! {
///     /// Genshin Impact resource types.
///     pub enum GenshinResource {
///         /// Original Resin.
///         Resin(StaminaResource) => ("Original Resin", "resin"),
///         /// Parametric Transformer cooldown.
///         ParametricTransformer(CooldownResource) => ("Parametric Transformer", "transformer"),
///     }
/// }
/// ```
#[macro_export]
macro_rules! game_resource_enum {
    // Extended form: generates both data enum and type-only enum
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident / $type_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident($inner:ty) => ($display_name:literal, $icon:literal)
            ),* $(,)?
        }
    ) => {
        $crate::game_resource_enum! {
            $(#[$meta])*
            $vis enum $name {
                $(
                    $(#[$variant_meta])*
                    $variant($inner) => ($display_name, $icon)
                ),*
            }
        }

        #[doc = concat!("Resource type identifiers for [`", stringify!($name), "`].")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ::serde::Serialize, ::serde::Deserialize, ::strum::AsRefStr)]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        $vis enum $type_name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $type_name {
            /// Returns a static slice of all variants.
            #[must_use]
            pub const fn all() -> &'static [Self] {
                &[$(Self::$variant,)*]
            }
        }
    };

    // Original form: generates only the data enum
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident($inner:ty) => ($display_name:literal, $icon:literal)
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(rename_all = "snake_case", tag = "type", content = "data")]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant($inner),
            )*
        }

        impl $crate::DisplayableResource for $name {
            fn display_name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant(_) => $display_name,
                    )*
                }
            }

            fn icon(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant(_) => $icon,
                    )*
                }
            }
        }
    };
}
