//! The path module provides a trait for accessing values from a JSON-like
//! structure.
use std::borrow::Cow;

use crate::jsonlike::JsonLike;

///
/// The PathString trait provides a method for accessing values from a JSON-like
/// structure. The returned value is encoded as a plain string.
/// This is typically used in evaluating mustache templates.
pub trait PathString {
    fn path_string<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<Cow<'a, str>>;
}

/// PathValue trait provides a method for accessing values from JSON-like
/// structure, the returned value is wrapped with RawValue enum, delegating
/// encoding to the client of this method.
pub trait PathValue {
    fn raw_value<'a, T: AsRef<str>>(&'a self, path: &[T]) -> Option<ValueString<'a>>;
}

///
/// The PathGraphql trait provides a method for accessing values from a
/// JSON-like structure. The returned value is encoded as a GraphQL Value.
pub trait PathGraphql {
    fn path_graphql<T: AsRef<str>>(&self, path: &[T]) -> Option<String>;
}

impl PathString for serde_json::Value {
    fn path_string<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<Cow<'a, str>> {
        self.get_path(path).map(move |a| match a {
            serde_json::Value::String(s) => Cow::Borrowed(s.as_str()),
            _ => Cow::Owned(a.to_string()),
        })
    }
}

///
/// An optimized version of async_graphql::Value that handles strings in a more
/// efficient manner.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueString<'a> {
    Value(Cow<'a, async_graphql::Value>),
    String(Cow<'a, str>),
}
