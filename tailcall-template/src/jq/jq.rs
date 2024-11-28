use jaq_core::ValR;

use crate::jsonlike::{JsonLike, JsonObjectLike};

#[derive(Clone, PartialEq, PartialOrd)]
pub struct JsonLikeHelper<A: for<'a> JsonLike<'a>>(pub A);

impl<A> jaq_core::ValT for JsonLikeHelper<A>
where
    A: for<'a> JsonLike<'a> + Clone + PartialEq + PartialOrd,
{
    fn from_num(n: &str) -> ValR<Self> {
        match n.parse::<f64>() {
            Ok(num) => ValR::Ok(JsonLikeHelper(A::number_f64(num))),
            Err(err) => ValR::Err(jaq_core::Error::str(format!("Invalid number format: {}", err.to_string()))),
        }
    }

    fn from_map<I: IntoIterator<Item = (Self, Self)>>(iter: I) -> ValR<Self> {
        iter.into_iter().fold(ValR::Ok(Self(JsonLike::object(JsonObjectLike::new()))), |acc, (key, value)| {
            let key = match JsonLike::as_str(&key.0) {
                Some(key) => key,
                None => return ValR::Err(jaq_core::Error::str("The value cannot be converted to String")),
            };

            match acc {
                Ok(mut acc) => {
                    let acc_mut = JsonLike::as_object_mut(&mut acc.0).unwrap();
                    acc_mut.insert_key(key, value.0);
                    ValR::Ok(acc)
                },
                Err(err) => ValR::Err(err),
            }
        })
    }

    fn values(self) -> Box<dyn Iterator<Item = ValR<Self>>> {
        todo!()
    }

    fn index(self, index: &Self) -> ValR<Self> {
        todo!()
    }

    fn range(self, range: jaq_core::val::Range<&Self>) -> ValR<Self> {
        todo!()
    }

    fn map_values<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        todo!()
    }

    fn map_index<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        index: &Self,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        todo!()
    }

    fn map_range<'a, I: Iterator<Item = jaq_core::ValX<'a, Self>>>(
        self,
        range: jaq_core::val::Range<&Self>,
        opt: jaq_core::path::Opt,
        f: impl Fn(Self) -> I,
    ) -> jaq_core::ValX<'a, Self> {
        todo!()
    }

    fn as_bool(&self) -> bool {
        todo!()
    }

    fn as_str(&self) -> Option<&str> {
        todo!()
    }
}

impl<A> std::fmt::Display for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<A> From<bool> for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    fn from(value: bool) -> Self {
        todo!()
    }
}

impl<A> From<isize> for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    fn from(value: isize) -> Self {
        todo!()
    }
}

impl<A> From<String> for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    fn from(value: String) -> Self {
        todo!()
    }
}

impl<A> FromIterator<Self> for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        todo!()
    }
}

impl<A> std::ops::Add for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    type Output = ValR<Self>;
    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<A> std::ops::Sub for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    type Output = ValR<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<A> std::ops::Mul for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    type Output = ValR<Self>;
    fn mul(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<A> std::ops::Div for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    type Output = ValR<Self>;
    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<A> std::ops::Rem for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    type Output = ValR<Self>;
    fn rem(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<A> std::ops::Neg for JsonLikeHelper<A> where A: for<'a> JsonLike<'a> {
    type Output = ValR<Self>;
    fn neg(self) -> Self::Output {
        todo!()
    }
}
