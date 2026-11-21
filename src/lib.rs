//! Derive values for structs and enums
// TODO: Add a proc macro that can be put at the top of a crate that captures the README.md and also enables nightly features if nightly

use std::hash::Hash;

use syn::DeriveInput;

pub mod definition_value;
pub mod definitions;
pub use definition_value::CfgDefinitionValue;

pub mod user_settings;

pub mod flag_list;

macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(false)]
        println!($($arg)*);
    };
}

use crate::{
    definitions::get_all_definitions,
    dependency::{Dependency, DependencyType},
    flag_list::FlagList,
    user_settings::UserSetting,
};
pub mod dependency;
#[allow(clippy::too_many_arguments)] // Oh shut it, I've been working at this for several days now
/// Given settings for what items a derive should be added, return a token stream with the added definitions
#[must_use]
pub fn apply_derive(
    item: &mut DeriveInput,
    flags: &FlagList,
    definitions: Vec<CfgDefinitionValue>,
    config: ItemAttributes,
    allow_unused_flag: bool,
    is_nightly: Option<bool>,
    crate_dependencies: Option<&[(String, bool)]>,
    feature_flags: Option<&[String]>,
) -> Option<proc_macro2::TokenStream> {
    // let mut flags = flags;
    let definitions = definitions;
    // Do we know it's stable rust? If so we can skip a bunch of nightly only stuff
    let is_stable = !is_nightly.unwrap_or(true);
    let is_nightly = is_nightly.unwrap_or_default();
    let names = definitions
        .iter()
        .map(|x| x.feature_name.clone()) // Does this need to use `.get_feature`()?
        .collect::<Vec<String>>();
    let defined_flags = flags.clone();
    debug_println!("// Before Definitions");
    let valid_definitions = filter_definitions(
        definitions,
        flags,
        &config,
        crate_dependencies,
        feature_flags,
    );

    debug_println!("// After Definitions");

    if !flags.0.is_empty() {
        if allow_unused_flag {
        } else {
            if flags.0.is_empty() {
                return Some(
                    syn::Error::new_spanned(
                        &item,
                        format!(
                            "Unknown macro input{}: {}. \nAll expected fields have been defined. Consider removing it?",
                            if flags.0.len() == 1{""}else{"s"},
                            flags.0
                                .iter()
                                .map(|x| format!("`{}` ({:?})", x.0, x.1))
                                .collect::<Vec<String>>()
                                .join(", "),
                        ),
                    )
                    .to_compile_error(),
                );
            }
            // This should be removed when the todo in the get_definition function is implemented (User setting removal after no definition depends on it)
            let is_useless = flags
                .0
                .iter()
                .filter(|x| !defined_flags.0.contains(x))
                .count()
                == 0;
            if !is_useless {
                return Some(
                syn::Error::new_spanned(
                    &item,
                    format!(
                        "Unknown macro input{}: {}.\nExpected one of: {}\nDefined fields: {}\nThis error is useless: {}",
                        if flags.0.len() == 1 { "" } else { "s" },
                        flags
                            .0
                            .iter()
                            .map(|x| format!("`{}` ({:?})", x.0, x.1))
                            .collect::<Vec<String>>()
                            .join(", "),
                        names
                            .iter()
                            .filter(|x| !valid_definitions.iter().any(|y| y.feature_name.eq(*x)))
                            .map(|x| format!("`{x}`"))
                            .collect::<Vec<String>>()
                            .join(", "),
                        defined_flags
                            .0
                            .into_iter()
                            .map(|x| x.0)
                            .collect::<Vec<String>>()
                            .join(", "),is_useless
                    ),
                )
                .to_compile_error(),
            );
            }
        }
    }

    for info in valid_definitions {
        let const_derive = info.config.derive_level.is_const();

        // Add all derives as separate #[derive(...)] attributes
        for derive in &info.streams {
            add_derive(
                item,
                &name_to_valid_flag(info.get_feature()),
                &info.dependencies,
                info.config.name_is_dependency,
                const_derive && !is_stable,
                derive,
                is_nightly,
                info.config.derive_level.is_derived(),
            );
        }
    }

    // panic!();

    // if !allow_unused_config && !codec_derives.is_empty() {
    //     Some(
    //         syn::Error::new_spanned(
    //             &item,
    //             format!(
    //                 "Not all fields have been defined in the `mirl_macros` macro, missing fields: {}",
    //                 codec_derives
    //                     .iter()
    //                     .map(|x| format!("`{}`", x.feature))
    //                     .collect::<Vec<String>>()
    //                     .join(", ")
    //             ),
    //         )
    //         .to_compile_error(),
    //     )
    // } else {
    //     None
    // }
    None
}

#[must_use]
/// Convert a possibly invalid crate name defined in Cargo.toml to a valid flag
pub fn name_to_valid_flag(name: &str) -> String {
    name.replace([' ', '-'], "_")
}
#[allow(clippy::too_many_arguments)] // Shut it! I've been at this fuck ass crate for a month now
/// Add a single configuration to the target item
///
/// Expects `const_derive` to be false if user is known to use stable rust
///
/// # Panics
/// When `proc_macro2` could not process the input
pub fn add_derive(
    item: &mut DeriveInput,
    name: &str,
    dependencies: &[Dependency],
    name_is_dependency: bool,
    const_derive: bool,
    derive: &proc_macro2::TokenStream,
    is_known_nightly: bool,
    is_derived: bool,
) {
    let mut features = if name_is_dependency {
        vec![Dependency::new(
            name.split_once('#').map_or(name, |x| x.0),
            dependency::DependencyType::Feature, // Instead of this, we should check if the crate exists. If it doesn't then skip this function call. Otherwise add the optional cfg.
            true,
        )]
    } else {
        Vec::new()
    };

    features.extend(dependencies.to_vec());

    let needs_checking = const_derive && !is_known_nightly;

    let requirements_list: Vec<String> = features
        .iter()
        .filter_map(dependency::Dependency::to_cfg)
        .collect::<Vec<String>>();
    let requirements = requirements_list.join(", ");

    let (feature, const_feature) = if needs_checking {
        if requirements_list.is_empty() {
            (
                "not(IS_NIGHTLY)".to_string(),
                Some("IS_NIGHTLY".to_string()),
            )
        } else {
            (
                format!("all(not(IS_NIGHTLY), {requirements})"),
                Some(format!("all(IS_NIGHTLY, {requirements})")),
            )
        }
    } else {
        if requirements_list.is_empty() {
            ("all(true)".to_string(), None)
        } else {
            (format!("all({requirements})"), None)
        }
    };

    let Ok(stream): Result<proc_macro2::TokenStream, proc_macro2::LexError> = feature.parse()
    else {
        panic!(
            "Error at {}:{}:{} \nSome feature was not defined properly: {}",
            file!(),
            line!(),
            column!(),
            feature
        )
    };
    // item.attrs.push(syn::parse_quote! {
    //     #[allow(unexpected_cfgs)]
    // });
    if is_derived {
        if let Some(const_feature) = const_feature {
            let Ok(const_stream): Result<proc_macro2::TokenStream, proc_macro2::LexError> =
                const_feature.parse()
            else {
                panic!(
                    "Error at {}:{}:{} \nSome feature was not defined properly: {}",
                    file!(),
                    line!(),
                    column!(),
                    feature
                )
            };
            // panic!("### #[cfg_attr({}, derive({}))]", const_feature, derive.to_string());
            item.attrs.push(syn::parse_quote! {
                #[cfg_attr(#const_stream, derive_const(#derive))]
            });
        }
        if const_derive {
            item.attrs.push(syn::parse_quote! {
                #[cfg_attr(#stream, derive_const(#derive))]
            });
        } else {
            item.attrs.push(syn::parse_quote! {
                #[cfg_attr(#stream, derive(#derive))]
            });
        }
    } else {
        item.attrs.push(syn::parse_quote! {
            #[cfg_attr(#stream, #derive)]
        });
    }
    // println!("{:?}", item.attrs.last().unwrap());
    debug_println!(
        "// #> {} -> Adding {} as {} {} with {}",
        item.ident,
        name,
        if needs_checking { "checking" } else { "set" },
        if is_known_nightly {
            "const"
        } else {
            "nonconst"
        },
        if requirements.is_empty() {
            "no dependencies".to_string()
        } else {
            format!(
                "the dep{} of {}",
                if requirements.len() == 1 { "" } else { "s" },
                requirements
            )
        }
    );
}
/// Convert a string from `CamelCasing` to `snake_casing`
#[must_use]
pub fn camel_to_snake_casing(camel: &str) -> String {
    let chars = camel.chars();
    let mut output: Vec<char> =
        Vec::with_capacity(camel.len() + camel.chars().filter(|x| x.is_uppercase()).count());

    let mut first = true;
    for char in chars {
        if char.is_uppercase() {
            if !first {
                output.push('_');
            }
            output.push(char.to_ascii_lowercase());
        } else {
            output.push(char);
        }
        first = false;
    }
    // let o: String = output.iter().collect();
    // panic!("### {}", o);

    output.iter().collect()
}

// /// Attribute macro to conditionally derive codec traits.
// ///
// /// Applies serialization/deserialization derives based on enabled features.
// /// Supports: `serde`, `bitcode`, `wincode`, `compactly`
// /// Optionally: `strum`, `enum_ext`
// ///
// /// # Example
// /// ```ignore
// /// #[derive_codec(serde = true, bitcode = false, wincode = true, compactly = true)]
// /// pub struct MyData {
// ///     value: i32,
// /// }
// /// ```
// // #[proc_macro_attribute]
// fn derive_possible_configured(args: TokenStream, input: TokenStream) -> TokenStream {
//     let mut item = parse_macro_input!(input as DeriveInput);
//     let FlagList(mut flags) = parse_macro_input!(args as FlagList);
//     if let Some(err) = apply_derive(
//         &mut item,
//         &mut flags,
//         get_codec_definition(),
//         false,
//         true,
//         false,
//     ) {
//         return err.into();
//     }
//     if let syn::Data::Enum(_) = item.data
//         && let Some(err) = apply_derive(
//             &mut item,
//             &mut flags,
//             get_enum_definition(),
//             false,
//             false,
//             false,
//         )
//     {
//         return err.into();
//     }
//     quote! { #item }.into()
// }

// /// Attribute macro to conditionally derive codec traits.
// ///
// /// Applies serialization/deserialization derives based on enabled features.
// /// Supports: serde, bitcode, wincode, compactly
// ///
// /// # Example
// /// ```ignore
// /// #[derive_codec(serde = true, bitcode = false, wincode = true, compactly = true)]
// /// pub struct MyData {
// ///     value: i32,
// /// }
// /// ```
// // #[proc_macro_attribute]
// fn derive_codec(args: TokenStream, input: TokenStream) -> TokenStream {
//     let mut item = parse_macro_input!(input as DeriveInput);
//     let FlagList(mut flags) = parse_macro_input!(args as FlagList);
//     if let Some(err) = apply_derive(
//         &mut item,
//         &mut flags,
//         get_codec_definition(),
//         false,
//         false,
//         false,
//     ) {
//         return err.into();
//     }
//     quote! { #item }.into()
// }

// /// Attribute macro for better enum deriving.
// ///
// /// Applies strum and optional `enum_ext` derives to enums.
// /// Supports: serde, strum, `enum_ext`
// ///
// /// # Example
// /// ```ignore
// /// #[derive_better_enum(serde = true, strum = true, enum_ext = false)]
// /// pub enum Color {
// ///     Red,
// ///     Green,
// ///     Blue,
// /// }
// /// ```
// // #[proc_macro_attribute]
// fn derive_better_enum(args: TokenStream, input: TokenStream) -> TokenStream {
//     let mut item = parse_macro_input!(input as DeriveInput);
//     let FlagList(mut flags) = parse_macro_input!(args as FlagList);
//     if let Some(err) = apply_derive(
//         &mut item,
//         &mut flags,
//         get_enum_definition(),
//         false,
//         false,
//         false,
//     ) {
//         return err.into();
//     }
//     quote! { #item }.into()
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Handles the complexity of configurations
pub struct ItemAttributes {
    /// What kind of item we're working with
    pub item_type: ItemType,
    /// If the item has a lifetime
    pub has_lifetime: bool,
    /// If the item has a generic
    pub has_generic: bool,
    /// If the item has a const value
    pub has_const: bool,
    /// If only read definitions are allowed
    pub read_only: bool,
}
#[must_use]
#[allow(clippy::let_and_return)]
/// Filter given definitions based on all things that may cull or limit definition existence
pub fn filter_definitions(
    definitions: Vec<CfgDefinitionValue>,
    user_settings: &FlagList,
    item_attributes: &ItemAttributes,
    dependencies: Option<&[(String, bool)]>,
    feature_flags: Option<&[String]>,
) -> Vec<CfgDefinitionValue> {
    debug_println!("// Inside filter definition");
    // debug_println!("// Inside filter definition");
    let definitions = filter_definition_based_on_user_settings(definitions, user_settings);
    let definitions = item_attributes.filter_definitions(definitions);
    let definitions = filter_definitions_on_dependencies(definitions, dependencies);

    let definitions = if let Some(feature_flags) = feature_flags {
        filter_definitions_on_features(definitions, feature_flags)
    } else {
        definitions
    };
    let definitions = handle_definition_dependencies_on_other_dependencies(definitions);
    let definitions = remove_useless_definition_dependencies(definitions, dependencies);
    definitions
}
#[must_use]
/// Remove definition feature dependencies if their respective crate is always enabled
pub fn remove_useless_definition_dependencies(
    definitions: Vec<CfgDefinitionValue>,
    dependencies: Option<&[(String, bool)]>,
) -> Vec<CfgDefinitionValue> {
    fn is_dep_optional(name: &str, deps: &[(String, bool)]) -> Option<bool> {
        Some(deps.iter().find(|x| x.0.eq(name))?.1)
    }
    let mut definitions = definitions;
    if let Some(dependencies) = dependencies {
        for definition in &mut definitions {
            definition
                .dependencies
                .retain(|x| is_dep_optional(&x.value, dependencies).unwrap_or(true));
        }
    } else {
        for definition in &mut definitions {
            definition.dependencies.retain(|x| {
                debug_println!("Retaining: {}={}", definition.feature_name, x.is_required);
                x.is_required
            });
        }
    }
    definitions
}

#[must_use]
/// Filter the definitions based on the dependencies
///
/// Filter order idx: 1
pub fn filter_definitions_on_features(
    definitions: Vec<CfgDefinitionValue>,
    crate_feature_flags: &[String],
) -> Vec<CfgDefinitionValue> {
    // println!("Given features: {:?}", crate_feature_flags);
    definitions
        .into_iter()
        .filter_map(|definition| {
            let mut definition = definition;
            let mut remove_definition = false;
            // println!("Definition {} requires: {:?}",definition.feature_name, definition.dependencies);
            definition.dependencies = definition
                .dependencies
                .into_iter()
                .filter_map(|dependency| {
                    if !matches!(dependency.dep_type, DependencyType::Feature) {
                        return Some(dependency);
                    }
                    if !dependency.is_required {
                        // TODO: Is this right?
                        if !crate_feature_flags.iter().any(|x| dependency.value.eq(x)) {
                            return None;
                        }

                        return Some(dependency);
                    }
                    // If the crate the definition is depending on does not exist, we remove it
                    if !crate_feature_flags.iter().any(|x| dependency.value.eq(x)) {
                        remove_definition = true;
                    }
                    None
                })
                .collect();
            if remove_definition {
                None
            } else {
                Some(definition)
            }
        })
        .collect()
}

#[must_use]
/// Filter the definitions based on the dependencies
///
/// Filter order idx: 0
pub fn filter_definitions_on_dependencies(
    definitions: Vec<CfgDefinitionValue>,
    crate_dependencies: Option<&[(String, bool)]>,
) -> Vec<CfgDefinitionValue> {
    // debug_println!(
    //     "Dependencies are as followed: [\n\t{}\n]",
    //     crate_dependencies
    //         .map_or("None".to_string(), |m| m
    //             .iter()
    //             .map(|(name, optional)| format!("{name}: Optional: {optional}"))
    //             .collect::<Vec<String>>()
    //             .join(",\n\t"))
    // );
    definitions
        .into_iter()
        .filter_map(|definition| {
            let mut definition = definition;
            let mut remove_definition = false;
            let mut added_dependencies = Vec::new();
            // println!("Processing here: {} with deps: {:?}", definition.feature_name, definition.dependencies);
            definition.dependencies = definition
                .dependencies
                .into_iter()
                .filter_map(|dependency| {
                    if !matches!(dependency.dep_type, DependencyType::Crate) {
                        return Some(dependency);
                    }
                    // let d = definition.feature_name == "debug";
                    // if d{
                    //     println!("Debug here");
                    // }
                    if !dependency.is_required {
                        if let Some(crates) = crate_dependencies {
                            //     if d{
                            //     println!("Debug and dependencies found");
                            // }
                            // TODO: Is this right?
                            if !crates.iter().any(|x| x.0 == dependency.value) {
                                // if d{
                                //     println!("Debug here skipped {}", dependency.value);
                                // }
                                return None;
                            }
                        }
                        // if d{
                        //     println!("Debug here not skipping {}", dependency.value);
                        // }
                        return Some(dependency);
                    }
                    if let Some(dependencies) = crate_dependencies {
                        // If the crate the definition is depending on does not exist, we remove it
                        if let Some(crate_is_optional) = dependencies.iter().find_map(|x| {
                            if x.0 == dependency.value {
                                Some(x.1)
                            } else {
                                None
                            }
                        }) {
                            added_dependencies.push(Dependency {
                                dep_type: DependencyType::Feature,
                                is_required: !crate_is_optional,
                                value: dependency.value,
                            });
                        } else {
                            remove_definition = true;
                        }
                    } else {
                        // All dependencies are supported
                        added_dependencies.push(Dependency {
                            dep_type: DependencyType::Feature,
                            is_required: true,
                            value: dependency.value,
                        });
                    }

                    None
                })
                .collect();
            definition.dependencies.extend(added_dependencies);
            if remove_definition {
                debug_println!(
                    "Definition removed (dependency): {}",
                    definition.feature_name
                );
                None
            } else {
                Some(definition)
            }
        })
        .collect()
}
#[must_use]
/// Handle the scenario where a definition is dependant on another definition
///
/// # Panics
/// When it loops indefinitely which should never happen
pub fn handle_definition_dependencies_on_other_dependencies(
    definitions: Vec<CfgDefinitionValue>,
) -> Vec<CfgDefinitionValue> {
    let mut definitions = definitions;
    // This will loop for as long as the dependencies change in any way.
    // Never let me write code again.
    let mut loop_max: u32 = 1000;
    loop {
        loop_max -= 1;
        // I am the greatest programmer there is and no one can stop me.
        assert!(loop_max != 0, "Stuck trying to do dependency resolving");

        // Snapshot only what's needed (assumes Feature: Clone + Hash + Eq, DeriveLevel: Copy)
        let levels: std::collections::HashMap<_, _> = definitions
            .iter()
            .map(|d| (d.feature_name.clone(), d.config.derive_level))
            .collect();

        let len_before = definitions.len();
        let mut downgraded = false;

        definitions.retain_mut(|x| {
            let mut remove_definition = false;
            x.dependencies.retain(|dep| {
                if !matches!(dep.dep_type, dependency::DependencyType::OtherAttribute) {
                    return true;
                }

                let Some(&dl) = levels.get(&dep.value) else {
                    remove_definition = true;
                    return true; // missing dependency -> drop
                };
                if x.config.derive_level.is_const() && dl.is_derived() && !dl.is_const() {
                    x.config.derive_level.downgrade_to_non_const();
                    downgraded = true;
                }
                true
            });
            !remove_definition
        });

        if definitions.len() == len_before && !downgraded {
            break;
        }
    }
    definitions
}
#[must_use]
/// Remove or nonconst downcast definitions based on what the user defined
pub fn filter_definition_based_on_user_settings(
    definitions: Vec<CfgDefinitionValue>,
    user_settings: &FlagList,
) -> Vec<CfgDefinitionValue> {
    let mut definitions = definitions;
    definitions.retain_mut(|cfg| {
        let user_setting_pos = {
            let default = user_settings
                .0
                .iter()
                .position(|x| cfg.get_feature() == x.0);
            user_settings
                .0
                .iter()
                .position(|x| cfg.feature_name == x.0)
                .map_or(default, Some)
        };

        // debug_println!("// For {}: {:?}", cfg.feature, setting_pos);

        let active = if let Some(user_setting_pos) = user_setting_pos {
            let setting = user_settings.0[user_setting_pos].1;
            // println!("// HII: Found settings for `{}`", user_settings.0[user_setting_pos].0);
            // user_settings.0.remove(setting_pos); // We should remove this entry once no dependency is depending on it anymore
            let activate = setting.as_bool();

            // debug_println!(
            //     "// User setting for {} was set: {:?}",
            //     cfg.feature_name,
            //     activate
            // );
            if activate && !setting.allow_const() {
                cfg.config.derive_level.downgrade_to_non_const();
            }
            // here.push((cfg.feature.clone(), return_value));
            activate
        } else {
            // debug_println!(
            //     "// User setting for {} wasn't set, using default: {:?}",
            //     cfg.feature,
            //     cfg.config
            //         .default
            //         .unwrap_or(UNDEFINED_DEFAULT_ENABLED_BY_DEFAULT)
            // );
            cfg.config
                .default
                .unwrap_or(UNDEFINED_DEFAULT_ENABLED_BY_DEFAULT)
        };
        if !active {
            debug_println!("Definition removed (user definition): {}", cfg.feature_name);
        }
        // debug_println!("// Definition {} is active? {:?}", cfg.feature_name, active);
        active
    });
    definitions
}

impl ItemAttributes {
    #[must_use]
    /// Remove all definitions that don't follow the given requirements
    ///
    pub fn filter_definitions(
        &self,
        definitions: Vec<CfgDefinitionValue>,
    ) -> Vec<CfgDefinitionValue> {
        let mut definitions = definitions;
        definitions.retain_mut(|cfg| {
            if self.read_only && !cfg.is_read_only {
                debug_println!("Definition removed (read only): {}", cfg.feature_name);
                return false;
            }
            if self.has_generic && !cfg.config.supports_generic {
                debug_println!("Definition removed (has generic): {}", cfg.feature_name);
                return false;
            }
            if self.has_lifetime && !cfg.config.supports_lifetime {
                debug_println!("Definition removed (has lifetime): {}", cfg.feature_name);
                return false;
            }
            if self.has_const && !cfg.config.supports_const {
                debug_println!("Definition removed (has const): {}", cfg.feature_name);
                return false;
            }
            if !match self.item_type {
                ItemType::PureEnum => cfg.config.supports_enums_with_value.is_some(),
                ItemType::Struct => cfg.config.supports_structs,
                ItemType::StorageEnum => cfg.config.supports_enums_with_value.unwrap_or_default(),
            } {
                debug_println!(
                    "Definition removed (unsupported type): {}",
                    cfg.feature_name
                );
                // user_settings.remove_flag_from_vec(&cfg.feature_name);
                return false;
            }
            true
        });
        definitions
    }
}
/// If definitions that don't have a defined default should be enabled if state isn't defined from the user
pub const UNDEFINED_DEFAULT_ENABLED_BY_DEFAULT: bool = false;

/// The kind of item we're working with
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemType {
    /// Struct
    Struct,
    /// A pure enum meaning it doesn't hold any values
    PureEnum,
    /// An impure enum meaning it can hold values
    StorageEnum,
}
#[allow(clippy::type_complexity)]
/// Extract attributes from the flag list.
///
/// Returns: `(read_only, dependencies [name, not optional], features [name])`
pub fn get_crate_and_feature_and_read_only_from_flag_list(
    flags: &mut FlagList,
) -> (bool, Option<Vec<(String, bool)>>, Option<Vec<String>>) {
    let mut dependencies: Vec<(String, bool)> = Vec::new();
    let mut features: Vec<String> = Vec::new();

    let read_only = flags
        .vec_get_flag("read_only")
        .unwrap_or(UserSetting::False)
        .as_bool();
    flags.remove_flag_from_vec("read_only");

    let supports_all = flags
        .vec_get_flag("supports_all")
        .unwrap_or(UserSetting::False)
        .as_bool();
    flags.remove_flag_from_vec("supports_all");
    if supports_all {
        return (read_only, None, None);
    }

    let feature_prefix = "FEATURE_";
    let dependency_prefix = "DEPENDENCY_";
    flags.0.retain(|(name, value)| {
        if name.starts_with(feature_prefix) {
            if UserSetting::True == *value {
                features.push(name.chars().skip(feature_prefix.len()).collect());
            }
            return false;
        }
        if name.starts_with(dependency_prefix) {
            dependencies.push((
                name.chars()
                    .skip(dependency_prefix.len())
                    .collect::<String>()
                    .to_lowercase(),
                match *value {
                    UserSetting::False => false,
                    UserSetting::True => true,
                    UserSetting::NonConst => return false,
                },
            ));

            return false;
        }

        true
    });

    (read_only, Some(dependencies), Some(features))
}

/// Handle the derive info
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn handle_derive_all(
    item: DeriveInput,
    flags: FlagList,
    read_only: bool,
    dependencies: Option<&[(String, bool)]>,
    feature_flags: Option<&[String]>,
) -> proc_macro2::TokenStream {
    let mut flags = flags;
    let mut item = item;

    let definitions = get_all_definitions();

    // let c_compatible = flags
    //     .iter()
    //     .find(|x| x.0.eq("c_compatible"))
    //     .is_none_or(|x| x.1.as_bool());

    // let vals = ["serde", "bitcode", "zerocopy"];
    // for val in vals {
    //     add_flag_to_vec(&mut flags, val, UserSetting::True);
    // }
    // add_flag_to_vec(&mut flags, "wincode", item.generics.lt_token.is_none());

    // let core_values: [(&str, bool, &[_]); _] = [
    //     ("debug", true, &[]),
    //     ("clone", true, &[]),
    //     ("default", false, &[]),
    //     ("copy", true, &[]),
    //     ("partial_eq", true, &[]),
    //     ("eq", true, &["partial_eq"]),
    //     ("partial_ord", true, &["partial_eq"]),
    //     ("ord", true, &["partial_ord", "eq"]),
    //     ("hash", true, &[]),
    // ];

    let be_nice_to_derive = !flags
        .vec_get_flag("ignore_derive")
        .unwrap_or(UserSetting::True)
        .as_bool();
    flags.remove_flag_from_vec("ignore_derive");

    let already_derived = if be_nice_to_derive {
        let mut already_derived = Vec::new();
        for attr in &item.attrs {
            if attr.path().is_ident("derive") || attr.path().is_ident("derive_const") {
                let Ok(derives) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) else {
                    panic!("Error parsing already derived values.")
                };

                for derived in derives {
                    if let Some(ident) = derived.get_ident() {
                        // panic!("### '{}'", ident.to_string());
                        already_derived.push(camel_to_snake_casing(&ident.to_string()));
                    }
                }
            }
        }
        already_derived
    } else {
        Vec::new()
    };

    for i in already_derived {
        flags.add_flag_to_vec(&i, false);
    }
    // for (val, activate, _) in core_values {
    //     if already_derived.iter().any(|x| x.eq(val)) {
    //         add_flag_to_vec(&mut flags, val, false);
    //         continue;
    //     }
    //     add_flag_to_vec(&mut flags, val, activate);
    // }

    // for (flag, _, dependencies) in core_values {
    //     for dep in dependencies {
    //         match vec_get_flag(&flags, dep).unwrap() {
    //             UserSetting::False => {
    //                 remove_flag_from_vec(&mut flags, flag);
    //                 add_flag_to_vec(&mut flags, flag, false);
    //                 break;
    //             }
    //             UserSetting::NonConst => {
    //                 if vec_get_flag(&flags, flag).is_none_or(|x| matches!(x, UserSetting::False)) {
    //                     continue;
    //                 }
    //                 remove_flag_from_vec(&mut flags, flag);
    //                 add_flag_to_vec(&mut flags, flag, UserSetting::NonConst);
    //             }
    //             UserSetting::True => {}
    //         }
    //     }
    // }

    // if !flags.iter().any(|x| x.0.eq("wincode")) {
    //     if item.generics.lt_token.is_None {
    //         flags.push(("wincode".to_string(), true));
    //     } else {
    //         flags.push(("wincode".to_string(), false));
    //     }
    // }

    let has_lifetime = item
        .generics
        .params
        .iter()
        .any(|x| matches!(x, syn::GenericParam::Lifetime(_)));

    let has_generic = item
        .generics
        .params
        .iter()
        .any(|x| matches!(x, syn::GenericParam::Type(_)));
    let has_const = item
        .generics
        .params
        .iter()
        .any(|x| matches!(x, syn::GenericParam::Const(_)));

    let (is_enum, is_pure) = if let syn::Data::Enum(data) = &item.data {
        // debug_println!("// {:?}", flags);
        let mut pure = true;
        for variant in &data.variants {
            #[allow(clippy::equatable_if_let)] // `==` cannot be used here
            if !variant.fields.is_empty() {
                pure = false;
                break;
            }
        }
        (true, pure)
    } else {
        (false, false)
    };

    let config = ItemAttributes {
        item_type: if is_enum {
            if is_pure {
                ItemType::PureEnum
            } else {
                ItemType::StorageEnum
            }
        } else {
            ItemType::Struct
        },
        has_lifetime,
        has_generic,
        has_const,
        read_only,
    };

    debug_println!(
        "// Processing {} which is a {:?}",
        item.ident,
        config.item_type
    );

    // debug_println!(
    //     "----\n{:#?} vs {:?}",
    //     flags,
    //     cfg.iter().map(|x| x.feature).collect::<Vec<&str>>()
    // );

    if let Some(err) = apply_derive(
        &mut item,
        &flags,
        definitions,
        config,
        false,
        None,
        dependencies,
        feature_flags,
    ) {
        return err;
    }

    quote::quote! { #item }
}
