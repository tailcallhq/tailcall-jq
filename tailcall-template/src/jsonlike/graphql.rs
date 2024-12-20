use std::borrow::Cow;
use std::collections::HashMap;

use async_graphql::Name;
use async_graphql_value::{ConstValue, Number};
use indexmap::IndexMap;

use super::*;

impl<'obj, Value: JsonLike<'obj> + Clone> JsonObjectLike<'obj> for IndexMap<Name, Value> {
    type Value = Value;

    fn new() -> Self {
        IndexMap::new()
    }

    fn with_capacity(n: usize) -> Self {
        IndexMap::with_capacity(n)
    }

    fn get_key(&self, key: &str) -> Option<&Self::Value> {
        self.get(key)
    }

    fn insert_key(&mut self, key: &'obj str, value: Self::Value) {
        self.insert(Name::new(key), value);
    }

    fn remove_key(&mut self, key: &'obj str) -> Option<Self::Value> {
        self.swap_remove(key)
    }

    fn iter(&'obj self) -> impl Iterator<Item = (&'obj str, &'obj Self::Value)> {
        self.iter().map(|(k, v)| (k.as_str(), v))
    }
}

impl<'json> JsonLike<'json> for ConstValue {
    type JsonObject = IndexMap<Name, ConstValue>;

    fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn into_array(self) -> Option<Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            ConstValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            ConstValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            ConstValue::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            ConstValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            ConstValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, ConstValue::Null)
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self> {
        let mut val = self;
        for token in path {
            val = match val {
                ConstValue::List(seq) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    seq.get(index)?
                }
                ConstValue::Object(map) => map.get(token.as_ref())?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn get_key(&self, path: &str) -> Option<&Self> {
        match self {
            ConstValue::Object(map) => map.get(&async_graphql::Name::new(path)),
            _ => None,
        }
    }

    fn group_by(&self, path: &[String]) -> HashMap<String, Vec<&Self>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }

    fn null() -> Self {
        Default::default()
    }

    fn as_object(&self) -> Option<&Self::JsonObject> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn as_object_mut(&mut self) -> Option<&mut Self::JsonObject> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn into_object(self) -> Option<Self::JsonObject> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn object(obj: Self::JsonObject) -> Self {
        ConstValue::Object(obj)
    }

    fn obj(pairs: Vec<(&'json str, Self)>) -> Self {
        let mut map = IndexMap::new();
        for (k, v) in pairs {
            map.insert(async_graphql::Name::new(k), v);
        }
        ConstValue::Object(map)
    }

    fn array(arr: Vec<Self>) -> Self {
        ConstValue::List(arr)
    }

    fn string(s: Cow<'json, str>) -> Self {
        ConstValue::String(s.to_string())
    }

    fn number_f64(n: f64) -> Self {
        ConstValue::Number(Number::from_f64(n).unwrap())
    }
}
