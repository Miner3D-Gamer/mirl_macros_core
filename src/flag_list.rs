//! A module focusing on [`FlagList`]

use syn::{Token, parse::Parse};

use crate::user_settings::UserSetting;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// The flag list structured: `Vec<(flag, setting)>`
pub struct FlagList(pub Vec<(String, UserSetting)>);

impl Parse for FlagList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: UserSetting = input.parse()?;

            items.push((key.to_string().replace('-', "_"), value));

            let _ = input.parse::<Token![,]>();
        }

        Ok(Self(items))
    }
}
impl FlagList {
    /// Add a given flag to the a vec
    pub fn add_flag_to_vec<T: Into<UserSetting>>(&mut self, flag: &str, bool: T) {
        if !self.vec_contains_flag(flag.as_ref()) {
            self.0.push((flag.to_string(), bool.into()));
        }
    }
    /// Remove a given flag to the a vec
    pub fn remove_flag_from_vec(&mut self, flag: &str) {
        self.0.retain(|x| x.0.ne(flag));
    }
    /// Get the flag value inside the vec
    #[must_use]
    pub fn vec_get_flag(&mut self, flag: &str) -> Option<UserSetting> {
        self.0.iter().find(|x| x.0.eq(flag)).map(|x| x.1)
    }
    /// Check if vec contains a flag
    #[must_use]
    fn vec_contains_flag(&self, flag: &str) -> bool {
        self.0.iter().any(|x| x.0.eq(flag))
    }
}
