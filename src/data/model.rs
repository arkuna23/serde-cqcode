use std::{borrow::Cow, collections::VecDeque};

use serde::{de::Visitor, forward_to_deserialize_any, Deserialize, Serialize};

use crate::{de::access::CQCodeAccess, Error};

use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum ModelValue<'a> {
    Bool(bool),
    #[serde(borrow)]
    String(Cow<'a, str>),
    Number(Number),
    CQCode(CQCodeModel<'a>),
}

impl<'de> ModelValue<'de> {
    pub fn to_unexp(&'de self) -> serde::de::Unexpected<'de> {
        use serde::de::Unexpected;
        match *self {
            ModelValue::Bool(b) => Unexpected::Bool(b),
            ModelValue::String(ref cow) => Unexpected::Str(cow),
            ModelValue::Number(number) => match number {
                Number::Int(i) => Unexpected::Signed(i),
                Number::Float(f) => Unexpected::Float(f),
            },
            ModelValue::CQCode(_) => Unexpected::Map,
        }
    }
}

impl<'de> serde::de::Deserializer<'de> for ModelValue<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            ModelValue::Bool(b) => visitor.visit_bool(b),
            ModelValue::String(s) => match s {
                Cow::Owned(s) => visitor.visit_string(s),
                Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            },
            ModelValue::Number(n) => match n {
                Number::Int(i) => visitor.visit_i64(i),
                Number::Float(f) => visitor.visit_f64(f),
            },
            ModelValue::CQCode(cq) => visitor.visit_map(cq.into_access()),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if let ModelValue::String(s) = self {
            let mut chars = s.chars();
            if let (Some(c), None) = (chars.next(), chars.next()) {
                visitor.visit_char(c)
            } else {
                Err(Error::MismatchedValueType(s.to_string(), "char".into()))
            }
        } else {
            Err(Error::ExpectedCodeType)
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub(crate) struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = ModelValue<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a valid ModelValue")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ModelValue::Bool(value))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ModelValue::String(Cow::Borrowed(v)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ModelValue::Number(Number::Int(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ModelValue::Number(Number::Float(value)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut cq_type = None;
        let mut data = VecDeque::new();

        while let Some((key, value)) = map.next_entry::<Cow<'de, str>, ModelValue<'de>>()? {
            if key == "$type" {
                cq_type = if let ModelValue::String(ty) = value {
                    Some(ty)
                } else {
                    return Err(serde::de::Error::invalid_type(
                        value.to_unexp(),
                        &"a string for cq code type",
                    ));
                };
            } else {
                data.push_back((key, value));
            }
        }

        let cq_type = cq_type.ok_or_else(|| serde::de::Error::missing_field("$type"))?;
        Ok(ModelValue::CQCode(CQCodeModel { cq_type, data }))
    }
}

type CodeData<'a> = VecDeque<(Cow<'a, str>, ModelValue<'a>)>;

#[derive(Debug, Serialize, Clone, PartialEq, PartialOrd)]
pub struct CQCodeModel<'a> {
    #[serde(borrow)]
    pub cq_type: Cow<'a, str>,
    pub data: CodeData<'a>,
}

impl<'de> Deserialize<'de> for CQCodeModel<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ModelValue::CQCode(map) = deserializer.deserialize_map(ValueVisitor)? else {
            panic!("expected map")
        };
        Ok(map)
    }
}

impl<'de> CQCodeModel<'de> {
    pub fn into_access(self) -> CQCodeAccess<'de> {
        CQCodeAccess::from_model(self)
    }
}

impl<'de> serde::Deserializer<'de> for CQCodeModel<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(self.into_access())
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
