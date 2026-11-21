//! A module focusing on [`Dependency`]

// /// The crate dependency of a Definition
// pub struct DependencyCrate {
//     pub crate_name: String,

//     pub is_required: bool,
// }

// pub enum Dependency {
//     Crate { name: String, required: bool },

// }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// The dependency of a Definition
pub struct Dependency {
    /// [`DependencyValue`]
    pub dep_type: DependencyType,
    /// If dependency type is crate:
    ///     If the dependency is required or only exists to restrict the flag
    /// If the dependency type is feature:
    ///     If a dependency with the same name is required
    pub is_required: bool,
    /// The containing value
    pub value: String,
}
impl Dependency {
    #[must_use]
    /// Create a dependency
    pub fn new<T: Into<String>>(value: T, dep_type: DependencyType, required: bool) -> Self {
        Self {
            value: value.into(),
            dep_type,
            is_required: required,
        }
    }
    #[must_use]
    /// Transform the current dependency into a
    pub fn to_cfg(&self) -> Option<String> {
        Some(match self.dep_type {
            DependencyType::Crate => format!("CRATE_PRESENT_{}", self.value.to_uppercase()),
            DependencyType::Feature => format!("feature=\"{}\"", self.value),
            DependencyType::Custom => self.value.clone(),
            DependencyType::OtherAttribute => return None,
        })
    }
}

/// The dependency of a flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DependencyType {
    /// If the flag depends on a crate
    ///
    /// The flag will be enabled when the crate is unconditionally imported or a feature with the same name is active
    Crate,
    /// If the feature will only be activated when another flag is active
    OtherAttribute,
    /// If the flag depends on a feature
    ///
    /// The flag will only be enabled when a feature with the same name is active
    Feature,
    /// Custom, do whatever you want
    Custom,
}
