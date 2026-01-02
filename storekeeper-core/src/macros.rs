//! Macros for reducing boilerplate in game resource definitions.

/// Macro for defining game resource enums with consistent structure.
///
/// This macro generates:
/// - An enum with the specified variants
/// - Serde serialization with `{ type, data }` tagged format
/// - Implementation of `DisplayableResource` trait
/// - Standard derives (Debug, Clone, Serialize, Deserialize)
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
