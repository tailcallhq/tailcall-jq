use std::{borrow::Cow, ops::Deref};

use jaq_core::ValR;

use crate::jsonlike::{JsonLike, JsonObjectLike};

#[derive(Debug, Clone, PartialEq)]
pub struct JsonLikeHelper<'json,
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
>(pub Cow<'json, A>);

impl<'json, A> Deref for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<'json, A> From<A> for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    fn from(value: A) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'json, A> jaq_core::ValT for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    fn from_num(n: &str) -> ValR<Self> {
        match n.parse::<f64>() {
            Ok(num) => ValR::Ok(JsonLikeHelper(Cow::Owned(A::number_f64(num)))),
            Err(err) => ValR::Err(jaq_core::Error::str(format!(
                "Invalid number format: {}",
                err.to_string()
            ))),
        }
    }

    fn from_map<I: IntoIterator<Item = (Self, Self)>>(iter: I) -> ValR<Self> {
        iter.into_iter().fold(
            ValR::Ok(Self(Cow::Owned(JsonLike::object(JsonObjectLike::new())))),
            |acc, (key, value)| {
                match acc {
                    Ok(mut acc) => {
                        let key_str = key.0.as_str().ok_or_else(||
                            jaq_core::Error::str("Key cannot be converted to String"))?;
                        let acc_mut = JsonLike::as_object_mut(acc.0.to_mut()).unwrap();
                        acc_mut.insert_key(key_str, value.0.into_owned());
                        ValR::Ok(acc)
                    }
                    Err(err) => ValR::Err(err),
                }
            },
        )
    }

    fn values(self) -> Box<dyn Iterator<Item = ValR<Self>>> {
        if let Some(arr) = self.0.as_ref().as_array() {
            let owned_array: Vec<_> = arr.iter().cloned().collect();
            Box::new(owned_array.into_iter().map(|a| Ok(JsonLikeHelper(Cow::Owned(a)))))
        } else if let Some(obj) = self.0.as_ref().as_object() {
            let owned_array: Vec<_> = obj.iter().map(|(_k, v)| v.clone()).collect();
            Box::new(owned_array.into_iter().map(|a| Ok(JsonLikeHelper(Cow::Owned(a)))))
        } else {
            Box::new(core::iter::once(ValR::Err(jaq_core::Error::str(
                "Value is not object or array",
            ))))
        }
    }

    fn index(self, index: &Self) -> ValR<Self> {
        if let Some(obj) = self.0.as_ref().as_object() {
            let Some(key) = index.0.as_ref().as_str() else {
                return ValR::Err(jaq_core::Error::str("Key cannot be converted to String"));
            };

            match obj.get_key(key) {
                Some(item) => ValR::Ok(JsonLikeHelper(Cow::Owned(item.clone()))),
                None => ValR::Ok(JsonLikeHelper(Cow::Owned(JsonLike::null()))),
            }
        } else if let Some(arr) = self.0.as_ref().as_array() {
            let Some(index) = index.0.as_ref().as_u64() else {
                return ValR::Err(jaq_core::Error::str("Index cannot be converted to u64"));
            };

            match arr.get(index as usize) {
                Some(item) => ValR::Ok(JsonLikeHelper(Cow::Owned(item.clone()))),
                None => ValR::Ok(JsonLikeHelper(Cow::Owned(JsonLike::null()))),
            }
        } else {
            ValR::Err(jaq_core::Error::str("Value is not object or array"))
        }
    }

    fn range(self, range: jaq_core::val::Range<&Self>) -> ValR<Self> {
        let (from, upto) = (range.start, range.end);
        if let Some(a) = self.0.into_owned().into_array() {
            let len = a.len();

            let from = from
                .as_ref()
                .map(|i| i.as_i64())
                .flatten()
                .ok_or_else(|| jaq_core::Error::str("From is not a Number"));
            let upto = upto
                .as_ref()
                .map(|i| i.as_i64())
                .flatten()
                .ok_or_else(|| jaq_core::Error::str("Upto is not a Number"));

            let (from, upto) = from
                .and_then(|from| Ok((from, upto?)))
                .map(|(from, upto)| {
                    let from: Result<isize, _> = from
                        .try_into()
                        .map_err(|_| jaq_core::Error::str("From cannot be converted to isize"));
                    let upto: Result<isize, _> = upto
                        .try_into()
                        .map_err(|_| jaq_core::Error::str("Upto cannot be converted to isize"));
                    (from, upto)
                })?;

            from.and_then(|from| Ok((from, upto?))).map(|(from, upto)| {
                let from = abs_bound(Some(from), len, 0);
                let upto = abs_bound(Some(upto), len, len);
                let (skip, take) = skip_take(from, upto);
                a.iter()
                    .skip(skip)
                    .take(take)
                    .cloned()
                    .map(|v| JsonLikeHelper(Cow::Owned(v)))
                    .collect()
            })
        } else if let Some(s) = self.0.as_ref().as_str() {
            let len = s.chars().count();

            let from = from
                .as_ref()
                .map(|i| i.as_i64())
                .flatten()
                .ok_or_else(|| jaq_core::Error::str("From is not a Number"));
            let upto = upto
                .as_ref()
                .map(|i| i.as_i64())
                .flatten()
                .ok_or_else(|| jaq_core::Error::str("Upto is not a Number"));

            let (from, upto) = from
                .and_then(|from| Ok((from, upto?)))
                .map(|(from, upto)| {
                    let from: Result<isize, _> = from
                        .try_into()
                        .map_err(|_| jaq_core::Error::str("From cannot be converted to isize"));
                    let upto: Result<isize, _> = upto
                        .try_into()
                        .map_err(|_| jaq_core::Error::str("Upto cannot be converted to isize"));
                    (from, upto)
                })?;

            from.and_then(|from| Ok((from, upto?))).map(|(from, upto)| {
                let from = abs_bound(Some(from), len, 0);
                let upto = abs_bound(Some(upto), len, len);
                let (skip, take) = skip_take(from, upto);
                JsonLikeHelper(Cow::Owned(JsonLike::string(s.chars().skip(skip).take(take).collect())))
            })
        } else {
            Err(jaq_core::Error::str("Value is not object or array"))
        }
    }

    fn map_values<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        if let Some(arr) = self.0.as_ref().as_array() {
            let iter = arr.iter().map(|a| JsonLikeHelper(Cow::Owned(a.clone()))).flat_map(f);
            Ok(iter.collect::<Result<_, _>>()?)
        } else if let Some(obj) = self.0.clone().as_ref().as_object() {
            let iter = obj
                .iter()
                .filter_map(|(k, v)| f(JsonLikeHelper(Cow::Owned(v.clone()))).next().map(|v| Ok((k, v?.0.into_owned()))));
            let obj = A::obj(iter.collect::<Result<Vec<_>, jaq_core::Exn<_>>>()?);
            Ok(JsonLikeHelper(Cow::Owned(obj)))
        } else {
            return opt.fail(self, |_v| {
                jaq_core::Exn::from(jaq_core::Error::str("Value is not object or array"))
            });
        }
    }

    fn map_index<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        mut self,
        index: &Self,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        if let Some(obj) = self.0.to_mut().as_object_mut() {
            let Some(key) = index.0.as_ref().as_str() else {
                return opt.fail(self, |_v| {
                    jaq_core::Exn::from(jaq_core::Error::str("Key cannot be converted to String"))
                });
            };

            match obj.get_key(key) {
                Some(e) => match f(JsonLikeHelper(Cow::Owned(e.clone()))).next().transpose()? {
                    Some(value) => obj.insert_key(key, value.0.into_owned()),
                    None => {
                        obj.remove_key(key);
                    }
                },
                None => {
                    if let Some(value) = f(JsonLikeHelper(Cow::Owned(JsonLike::null()))).next().transpose()? {
                        obj.insert_key(key, value.0.into_owned());
                    }
                }
            }
            Ok(self)
        } else if let Some(arr) = self.0.to_mut().as_array_mut() {
            let Some(index) = index.0.as_ref().as_u64() else {
                return opt.fail(self, |_v| {
                    jaq_core::Exn::from(jaq_core::Error::str("Index cannot be converted to u64"))
                });
            };
            let abs_or = |i| {
                abs_index(i, arr.len())
                    .ok_or(jaq_core::Error::str(format!("index {i} out of bounds")))
            };
            // TODO: perform error handling
            let index = match abs_or(index.try_into().unwrap()) {
                Ok(index) => index,
                Err(e) => return opt.fail(self, |_v| jaq_core::Exn::from(e)),
            };

            let item = arr[index].clone();
            if let Some(value) = f(JsonLikeHelper(Cow::Owned(item))).next().transpose()? {
                arr[index] = value.0.into_owned();
            } else {
                arr.remove(index);
            }
            Ok(self)
        } else {
            return opt.fail(self, |_v| {
                jaq_core::Exn::from(jaq_core::Error::str("Value is not object or array"))
            });
        }
    }

    fn map_range<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        mut self,
        range: jaq_core::val::Range<&Self>,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        if let Some(arr) = self.0.to_mut().as_array_mut() {
            let len = arr.len();
            let from: Result<Option<isize>, jaq_core::Error<JsonLikeHelper<A>>> = range
                .start
                .as_ref()
                .as_ref()
                .map(|i| i.as_i64())
                .flatten()
                .map(|v| {
                    v.try_into()
                        .map_err(|_| jaq_core::Error::str("From cannot be converted to isize"))
                })
                .transpose();
            let upto: Result<Option<isize>, jaq_core::Error<JsonLikeHelper<A>>> = range
                .end
                .as_ref()
                .as_ref()
                .map(|i| i.as_i64())
                .flatten()
                .map(|v| {
                    v.try_into()
                        .map_err(|_| jaq_core::Error::str("From cannot be converted to isize"))
                })
                .transpose();

            let (Ok(from), Ok(upto)) = (from, upto) else {
                return opt.fail(self, |_v| {
                    jaq_core::Exn::from(jaq_core::Error::str("Failed to parse range"))
                });
            };

            let from = abs_bound(from, len, 0);
            let upto = abs_bound(upto, len, len);

            let (skip, take) = skip_take(from, upto);

            let arr_slice = arr
                .iter_mut()
                .skip(skip)
                .take(take)
                .map(|a| a.clone())
                .collect::<Vec<_>>();

            let new_values =
                f(JsonLikeHelper(Cow::Owned(JsonLike::array(arr_slice)))).collect::<Result<Vec<_>, _>>()?;

            arr.splice(skip..skip + take, new_values.into_iter().map(|a| a.0.into_owned()));
            Ok(self)
        } else {
            opt.fail(self, |_v| {
                jaq_core::Exn::from(jaq_core::Error::str("Value is not array"))
            })
        }
    }

    fn as_bool(&self) -> bool {
        if let Some(b) = self.0.as_ref().as_bool() {
            b
        } else if self.0.as_ref().is_null() {
            false
        } else {
            true
        }
    }

    fn as_str(&self) -> Option<&str> {
        if let Some(s) = self.0.as_ref().as_str() {
            Some(s)
        } else if let Some(b) = self.0.as_ref().as_bool() {
            if b {
                Some("true")
            } else {
                Some("false")
            }
        } else {
            // TODO: fill the rest cases if possible
            None
        }
    }
}

impl<'json, A> PartialOrd for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // TODO: compare properly
        self.0.to_string().partial_cmp(&other.0.to_string())
    }
}

impl<'json, A> std::fmt::Display for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'json, A> From<bool> for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    fn from(value: bool) -> Self {
        todo!()
    }
}

impl<'json, A> From<isize> for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    fn from(value: isize) -> Self {
        todo!()
    }
}

impl<'json, A> From<String> for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    fn from(value: String) -> Self {
        JsonLikeHelper(Cow::Owned(JsonLike::string(Cow::Owned(value))))
    }
}

impl<'json, A> FromIterator<Self> for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        todo!()
    }
}
impl<'json, A> std::ops::Add for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    type Output = ValR<JsonLikeHelper<'json, A>>;
    fn add(mut self, rhs: Self) -> Self::Output {
        if self.0.as_ref().is_null() && rhs.0.as_ref().is_null() {
            return Ok(self);
        } else if let (Some(l), Some(r)) = (self.0.as_ref().as_f64(), rhs.0.as_ref().as_f64()) {
            return Ok(JsonLikeHelper(Cow::Owned(A::number_f64(l + r))));
        } else if let (Some(l), Some(r)) = (self.0.as_ref().as_str(), rhs.0.as_ref().as_str()) {
            let mut result = String::from(l);
            result.push_str(r);
            return Ok(JsonLikeHelper(Cow::Owned(A::string(result.into()))));
        } else if let (Some(l), Some(r)) = (self.0.to_mut().as_array_mut(), rhs.0.as_ref().as_array()) {
            l.extend(r.iter().cloned());
            return Ok(self);
        } else if let Some(obj_mut) = self.0.to_mut().as_object_mut() {
            if let Some(rhs_obj) = rhs.0.as_ref().as_object() {
                for (k, v) in rhs_obj.iter() {
                    obj_mut.insert_key(k, v.clone());
                }
                return Ok(self);
            }
        }
        Err(jaq_core::Error::str("Cannot add values of different types"))
    }
}

impl<'json, A> std::ops::Sub for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    type Output = ValR<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<'json, A> std::ops::Mul for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    type Output = ValR<Self>;
    fn mul(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<'json, A> std::ops::Div for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    type Output = ValR<Self>;
    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<'json, A> std::ops::Rem for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    type Output = ValR<Self>;
    fn rem(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<'json, A> std::ops::Neg for JsonLikeHelper<'json, A>
where
    A: JsonLike<'json> + std::fmt::Display + std::clone::Clone + std::cmp::PartialEq,
{
    type Output = ValR<Self>;
    fn neg(self) -> Self::Output {
        todo!()
    }
}

fn skip_take(from: usize, until: usize) -> (usize, usize) {
    (from, if until > from { until - from } else { 0 })
}

/// If a range bound is given, absolutise and clip it between 0 and `len`,
/// else return `default`.
fn abs_bound(i: Option<isize>, len: usize, default: usize) -> usize {
    i.map_or(default, |i| core::cmp::min(wrap(i, len).unwrap_or(0), len))
}

/// Absolutise an index and return result if it is inside [0, len).
fn abs_index(i: isize, len: usize) -> Option<usize> {
    wrap(i, len).filter(|i| *i < len)
}

fn wrap(i: isize, len: usize) -> Option<usize> {
    if i >= 0 {
        Some(i as usize)
    } else if len < -i as usize {
        None
    } else {
        Some(len - (-i as usize))
    }
}
