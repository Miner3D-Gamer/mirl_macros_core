//! A module focusing on [`CfgDefinitionValue`]

use crate::dependency::Dependency;
#[derive(Debug, Clone)]
/// A definition of a configurably derivative value
pub struct CfgDefinitionValue {
    /// The feature name
    pub feature_name: String,
    /// The derive parameter in quote! form
    pub streams: Vec<proc_macro2::TokenStream>,
    /// If the stream is read only
    /// What does this mean?
    ///
    /// Items that restrict definitions to be read only can have invalid states.
    ///
    /// For example:
    /// - `u8`: Cannot have an invalid state -> `read_only`: false
    /// - `struct GameBoard {tiles: Vec<u8>, width: usize, height: usize}`: The length of the list and the width/height could differ which would create an invalid state -> `read_only`: true
    pub is_read_only: bool,
    /// What dependencies are required
    pub dependencies: Vec<Dependency>,
    /// [`CfgDefinitionValueSettings`]
    pub config: CfgDefinitionValueSettings,
}

impl PartialEq for CfgDefinitionValue {
    fn eq(&self, other: &Self) -> bool {
        format!("{self:?}") == format!("{other:?}")
    }
}

impl Eq for CfgDefinitionValue {}

impl PartialOrd for CfgDefinitionValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CfgDefinitionValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        format!("{self:?}").cmp(&format!("{other:?}"))
    }
}

impl std::hash::Hash for CfgDefinitionValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{self:?}").hash(state);
    }
}
/// An overload trait for convenience
pub trait CfgDefinitionValueNew<D> {
    #[allow(clippy::new_ret_no_self)]
    /// Create a new flag config
    fn new<F: Into<String>, C: Into<CfgDefinitionValueSettings>>(
        feature: F,
        streams: Vec<proc_macro2::TokenStream>,
        is_read_only: bool,
        dependencies: D,
        config: C,
    ) -> CfgDefinitionValue;
}
impl From<&mut Self> for CfgDefinitionValueSettings {
    fn from(value: &mut Self) -> Self {
        *value
    }
}
impl CfgDefinitionValueNew<Vec<Dependency>> for CfgDefinitionValue {
    fn new<F: Into<String>, C: Into<CfgDefinitionValueSettings>>(
        feature: F,
        streams: Vec<proc_macro2::TokenStream>,
        is_read_only: bool,
        dependencies: Vec<Dependency>,
        config: C,
    ) -> CfgDefinitionValue {
        Self {
            feature_name: feature.into(),
            streams,
            config: config.into(),
            is_read_only,
            dependencies: dependencies.into_iter().collect(),
        }
    }
}

impl CfgDefinitionValueNew<Option<()>> for CfgDefinitionValue {
    fn new<F: Into<String>, C: Into<CfgDefinitionValueSettings>>(
        feature: F,
        streams: Vec<proc_macro2::TokenStream>,
        is_read_only: bool,
        _dependencies: Option<()>,
        config: C,
    ) -> CfgDefinitionValue {
        Self {
            feature_name: feature.into(),
            streams,
            is_read_only,
            config: config.into(),
            dependencies: Vec::default(),
        }
    }
}

/// Boolean settings for the [`CfgDefinitionValue`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CfgDefinitionValueSettings {
    /// If the field should be enabled by default
    pub default: Option<bool>,
    /// None if enums aren't supported
    ///
    /// If an enum holding a struct/enum/prim is supported
    pub supports_enums_with_value: Option<bool>,
    /// If it supports structs
    pub supports_structs: bool,
    /// Supports values with lifetimes
    pub supports_lifetime: bool,
    /// Supports values with generics
    pub supports_generic: bool,
    /// Supports values with const
    pub supports_const: bool,
    /// If the given name should also be treated as a dependency
    pub name_is_dependency: bool,
    /// If it needs a derive, else it'll be inputted raw
    // "That's what he said"
    pub derive_level: DeriveLevel,
}
/// How the given item is applied
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeriveLevel {
    /// `#[item]`
    Attribute,
    /// `#[derive(item)]`
    Derive,
    /// `#[const_derive(item)]`
    ConstDerive,
}
impl DeriveLevel {
    #[must_use]
    /// Does the item use `#[derive()]`?
    pub const fn is_derived(self) -> bool {
        match self {
            Self::Attribute => false,
            Self::ConstDerive | Self::Derive => true,
        }
    }
    /// If the item supports const
    #[must_use]
    pub const fn is_const(self) -> bool {
        matches!(self, Self::ConstDerive)
    }
    #[allow(clippy::single_match)]
    /// Downgrade the current value to not be const if it is
    pub const fn downgrade_to_non_const(&mut self) {
        match self {
            Self::ConstDerive => *self = Self::Derive,
            _ => {}
        }
    }
}
impl CfgDefinitionValueSettings {
    /// Creates a new configuration with the given derive level.
    #[must_use]
    pub const fn new(derive_level: DeriveLevel) -> Self {
        Self {
            default: None,
            supports_enums_with_value: Some(true),
            supports_structs: true,
            supports_generic: true,
            supports_lifetime: true,
            supports_const: true,
            name_is_dependency: true,
            derive_level,
        }
    }

    /// Sets the default value.
    #[must_use]
    pub fn default<D: Into<Option<bool>>>(&mut self, value: D) -> &mut Self {
        self.default = value.into();
        self
    }

    /// Sets whether enums with values are supported.
    #[must_use]
    pub fn supports_enums_with_value<D: Into<Option<bool>>>(&mut self, value: D) -> &mut Self {
        self.supports_enums_with_value = value.into();
        self
    }

    /// Sets whether structs are supported.
    #[must_use]
    pub const fn supports_structs(&mut self, value: bool) -> &mut Self {
        self.supports_structs = value;
        self
    }

    /// Sets whether generics are supported.
    #[must_use]
    pub const fn supports_generic(&mut self, value: bool) -> &mut Self {
        self.supports_generic = value;
        self
    }

    /// Sets whether lifetimes are supported.
    #[must_use]
    pub const fn supports_lifetime(&mut self, value: bool) -> &mut Self {
        self.supports_lifetime = value;
        self
    }

    /// Sets whether const parameters are supported.
    #[must_use]
    pub const fn supports_const(&mut self, value: bool) -> &mut Self {
        self.supports_const = value;
        self
    }

    /// Sets whether the name refers to a dependency.
    #[must_use]
    pub const fn name_is_dependency(&mut self, value: bool) -> &mut Self {
        self.name_is_dependency = value;
        self
    }
}
impl CfgDefinitionValue {
    #[must_use]
    /// Get feature without the info after
    pub fn get_feature(&self) -> &str {
        self.feature_name
            .split_once('#')
            .map_or(self.feature_name.as_str(), |x| x.0)
    }
}
