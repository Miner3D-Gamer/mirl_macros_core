//! A module focusing on [`UserSetting`]

use syn::{Token, parse::Parse};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Settings for the user
pub enum UserSetting {
    /// Yes, add this item
    True,
    /// No, addn't this item
    False,
    /// Yes, add this item but not as const
    NonConst,
}
impl From<bool> for UserSetting {
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }
}
impl From<Option<()>> for UserSetting {
    fn from(_: Option<()>) -> Self {
        Self::NonConst
    }
}
impl UserSetting {
    /// If the user agrees
    #[must_use]
    pub const fn as_bool(self) -> bool {
        match self {
            Self::False => false,
            Self::True | Self::NonConst => true,
        }
    }
    /// If const is allowed
    #[must_use]
    pub const fn allow_const(self) -> bool {
        matches!(self, Self::True)
    }
}
impl Parse for UserSetting {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // `true` / `false` are keywords, so parse them as LitBool.
        if input.peek(syn::LitBool) {
            let value: syn::LitBool = input.parse()?;
            return Ok(if value.value() {
                Self::True
            } else {
                Self::False
            });
        }

        // Parse the remaining forms.
        let first: syn::Ident = input.parse()?;
        let mut value = first.to_string();

        // Support `non-const`.
        if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            let second: syn::Ident = input.parse()?;
            value.push('-');
            value.push_str(&second.to_string());
        }

        match value.to_ascii_lowercase().as_str() {
            "nonconst" | "non_const" | "non-const" => Ok(Self::NonConst),
            _ => Err(syn::Error::new(
                first.span(),
                "expected `true`, `false`, `nonconst`, `non_const`, or `non-const`",
            )),
        }
    }
}
