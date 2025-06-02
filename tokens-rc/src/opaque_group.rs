//! [OpaqueGroup] impl.

use ::std::{
    cell::{Cell, OnceCell},
    fmt::Debug,
    ops::Deref,
};

use ::proc_macro2::{Span, TokenStream};
use ::quote::ToTokens;

/// Wrapper for [Group] with lazy conversion from [::proc_macro2::Group].
pub struct OpaqueGroup {
    /// Wrapped group.
    group: OnceCell<crate::Group>,
    /// Group to lazily convert from.
    backing: Cell<Option<::proc_macro2::Group>>,
}

impl OpaqueGroup {
    /// Get a reference to wrapped group.
    pub fn as_group(&self) -> &crate::Group {
        let Self { group, backing } = self;

        group.get_or_init(|| crate::Group::from(backing.take().unwrap_or_else(|| unreachable!())))
    }

    /// Get span of opaque group.
    pub fn span(&self) -> Span {
        if let Some(group) = self.backing.take() {
            let span = group.span();
            self.backing.set(Some(group));
            span
        } else {
            self.as_group().span
        }
    }

    /// Get tokens contained by group as a [TokenStream].
    pub fn stream(&self) -> TokenStream {
        if let Some(group) = self.backing.take() {
            let stream = group.stream();
            self.backing.set(Some(group));
            stream
        } else {
            self.stream.to_token_stream()
        }
    }
}

impl Debug for OpaqueGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let backing = self.backing.take();
        let res = f
            .debug_struct("OpaqueGroup")
            .field("group", &self.group)
            .field("backing", &backing)
            .finish();
        self.backing.set(backing);
        res
    }
}

impl Clone for OpaqueGroup {
    fn clone(&self) -> Self {
        Self {
            group: OnceCell::from(self.as_group().clone()),
            backing: Cell::new(None),
        }
    }
}

impl Deref for OpaqueGroup {
    type Target = crate::Group;

    fn deref(&self) -> &Self::Target {
        self.as_group()
    }
}

impl From<::proc_macro2::Group> for OpaqueGroup {
    fn from(value: ::proc_macro2::Group) -> Self {
        Self {
            group: OnceCell::new(),
            backing: Cell::new(Some(value)),
        }
    }
}

impl From<crate::Group> for OpaqueGroup {
    fn from(value: crate::Group) -> Self {
        Self {
            group: OnceCell::from(value),
            backing: Cell::new(None),
        }
    }
}

impl From<OpaqueGroup> for crate::Group {
    fn from(value: OpaqueGroup) -> Self {
        let OpaqueGroup { group, backing } = value;

        if let Some(group) = backing.into_inner() {
            crate::Group::from(group)
        } else {
            group.into_inner().unwrap_or_else(|| unreachable!())
        }
    }
}

impl ToTokens for OpaqueGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(group) = self.backing.take() {
            group.to_tokens(tokens);
            self.backing.set(Some(group));
        } else {
            self.as_group().to_tokens(tokens);
        }
    }
}
