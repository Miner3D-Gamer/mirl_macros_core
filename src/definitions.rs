//! A module containing all supported crates

use crate::{
    definition_value::{CfgDefinitionValueNew, CfgDefinitionValueSettings, DeriveLevel},
    dependency::Dependency,
    *,
};
use quote::quote;

#[must_use]
/// Get all definitions
pub fn get_all_definitions() -> Vec<CfgDefinitionValue> {
    let mut all = Vec::new();
    all.extend(get_codec_definition());
    all.extend(get_core_definition());
    all.extend(get_enum_definition());
    all.extend(get_integration_definition());
    all
}

/// Get definitions for supported crates in the codec department
#[must_use]
pub fn get_codec_definition() -> Vec<CfgDefinitionValue> {
    vec![
        CfgDefinitionValue::new(
            "serde#serialize",
            vec![quote! { serde::Serialize }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        ),
        CfgDefinitionValue::new(
            "serde#deserialize",
            vec![quote! { serde::Deserialize }],
            false,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive)
                .default(true)
                .supports_generic(false),
        ),
        CfgDefinitionValue::new(
            "bitcode#decode",
            vec![quote! { bitcode::Decode }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        ),
        CfgDefinitionValue::new(
            "bitcode#encode",
            vec![quote! { bitcode::Encode }],
            false,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        ),
        // Reenable wincode when
        // ```overly complex generic constant
        // consider moving this anonymous constant into a `const` function
        // this operation may be supported in the future```
        // Isn't an issue anymore. Latest broken version: 0.5.3
        CfgDefinitionValue::new(
            "wincode",
            vec![
                // quote! { wincode::SchemaRead },
            ],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(false),
        ),
        CfgDefinitionValue::new(
            "wincode",
            vec![
                // quote! { wincode::SchemaWrite },
            ],
            false,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(false),
        ),
        CfgDefinitionValue::new(
            "compactly",
            vec![
                // Why are u buggy?
                // quote! { compactly::v1::Encode },
                // quote! { compactly::v2::Encode },
            ],
            false,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive)
                .default(true)
                .supports_enums_with_value(false),
        ),
        CfgDefinitionValue::new(
            "zerocopy#into_bytes",
            vec![quote! { zerocopy::IntoBytes }],
            true,
            vec![Dependency::new(
                "c_compatible",
                dependency::DependencyType::OtherAttribute,
                true,
            )],
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        ),
        CfgDefinitionValue::new(
            "zerocopy#try_from_bytes",
            vec![quote! { zerocopy::TryFromBytes }],
            false,
            vec![Dependency::new(
                "c_compatible",
                dependency::DependencyType::OtherAttribute,
                true,
            )],
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        ),
        // CfgDefinitionValue::new(
        //     "rkyv",
        //     vec![quote! { rkyv::Serialize }],
        //     true,
        //     vec![Dependency::new(
        //         "rkyv#archive",
        //         dependency::DependencyType::OtherAttribute,
        //         true,
        //     )],
        //     CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        // ),
        // CfgDefinitionValue::new(
        //     "rkyv",
        //     vec![quote! { rkyv::Deserialize }],
        //     false,
        //     None,
        //     CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        // ),
        CfgDefinitionValue::new(
            "parity_scale_codec",
            vec![quote! { parity_scale_codec::Encode }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(false),
        ),
        CfgDefinitionValue::new(
            "parity_scale_codec",
            vec![quote! { parity_scale_codec::Decode }],
            false,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(false),
        ),
    ]
}
/// Get definitions for supported crates
#[must_use]
pub fn get_integration_definition() -> Vec<CfgDefinitionValue> {
    vec![
        CfgDefinitionValue::new(
            "get_size2",
            vec![quote! { get_size2::GetSize }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        ),
        CfgDefinitionValue::new(
            "bevy_reflect",
            vec![quote! { bevy_reflect::Reflect }],
            false,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        ),
        // CfgDefinitionValue::new(
        //     "rkyv#archive",
        //     vec![quote! { rkyv::Archive }],
        //     false,
        //     None,
        //     CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        // ),
        CfgDefinitionValue::new(
            "rkyv",
            vec![quote! { rkyv::Portable }],
            false,
            vec![Dependency::new(
                "c_compatible",
                dependency::DependencyType::OtherAttribute,
                true,
            )],
            CfgDefinitionValueSettings::new(DeriveLevel::Derive).default(true),
        ),
    ]
}
/// Get definitions for supported crates that are applied on enums
#[must_use]
pub fn get_enum_definition() -> Vec<CfgDefinitionValue> {
    vec![
        CfgDefinitionValue::new(
            "strum",
            vec![
                // quote! { strum::EnumIter }, This can only be implemented when a type has default
                quote! { strum::EnumCount },
                quote! { strum::AsRefStr },
                quote! { strum::IntoStaticStr },
                // quote! { strum::VariantArray },
                quote! { strum::VariantNames },
            ],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive)
                .default(true)
                .supports_structs(false),
        ),
        // TODO:
        // This crate errors: "Expected Ident"
        // I think it's trying to apply its stuff to what other definitions have created rather than the enum itself
        CfgDefinitionValue::new(
            "enum_ext",
            vec![
            // quote! { enum_ext::enum_extend }
            ],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Attribute)
                .default(true)
                .supports_enums_with_value(false)
                .supports_structs(false),
        ),
        CfgDefinitionValue::new(
            "enum-ordinalize",
            vec![quote! { enum_ordinalize::Ordinalize }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive)
                .default(true)
                .supports_structs(false)
                .supports_enums_with_value(false),
        ),
    ]
}
/// Get definitions for the default std
#[must_use]
pub fn get_core_definition() -> Vec<CfgDefinitionValue> {
    vec![
        CfgDefinitionValue::new(
            "debug",
            vec![quote! { core::fmt::Debug }],
            true,
            vec![Dependency::new(
                "std",
                dependency::DependencyType::Feature,
                false,
            )],
            CfgDefinitionValueSettings::new(DeriveLevel::Derive)
                .default(true)
                .name_is_dependency(false),
        ),
        CfgDefinitionValue::new(
            "clone",
            vec![quote! { core::clone::Clone }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::ConstDerive)
                .default(true)
                .name_is_dependency(false),
        ),
        CfgDefinitionValue::new(
            "eq",
            vec![quote! { core::cmp::Eq }],
            true,
            vec![Dependency::new(
                "partial_eq",
                dependency::DependencyType::OtherAttribute,
                true,
            )],
            CfgDefinitionValueSettings::new(DeriveLevel::ConstDerive)
                .default(true)
                .name_is_dependency(false),
        ),
        CfgDefinitionValue::new(
            "partial_eq",
            vec![quote! { core::cmp::PartialEq }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::ConstDerive)
                .default(true)
                .name_is_dependency(false),
        ),
        CfgDefinitionValue::new(
            "default",
            vec![quote! { core::default::Default }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::ConstDerive)
                // .supports_structs(false) // This line creates errors in `get_definitions` where it isn't removed from `user_settings` after being used
                .name_is_dependency(false),
        ),
        CfgDefinitionValue::new(
            "hash",
            vec![quote! { std::hash::Hash }],
            true,
            vec![Dependency::new(
                "std",
                dependency::DependencyType::Feature,
                false,
            )],
            CfgDefinitionValueSettings::new(DeriveLevel::Derive)
                .default(true)
                .name_is_dependency(false),
        ),
        CfgDefinitionValue::new(
            "ord",
            vec![quote! { core::cmp::Ord }],
            true,
            vec![
                Dependency::new(
                    "partial_ord",
                    dependency::DependencyType::OtherAttribute,
                    true,
                ),
                Dependency::new("eq", dependency::DependencyType::OtherAttribute, true),
            ],
            CfgDefinitionValueSettings::new(DeriveLevel::ConstDerive)
                .default(true)
                .name_is_dependency(false),
        ),
        CfgDefinitionValue::new(
            "partial_ord",
            vec![quote! { core::cmp::PartialOrd }],
            true,
            vec![Dependency::new(
                "partial_eq",
                dependency::DependencyType::OtherAttribute,
                true,
            )],
            CfgDefinitionValueSettings::new(DeriveLevel::ConstDerive)
                .default(true)
                .name_is_dependency(false),
        ),
        CfgDefinitionValue::new(
            "copy",
            vec![quote! { core::marker::Copy }],
            true,
            None,
            CfgDefinitionValueSettings::new(DeriveLevel::Derive)
                .default(true)
                .name_is_dependency(false),
        ),
    ]
}
