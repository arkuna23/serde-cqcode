use std::{borrow::Cow, collections::VecDeque};

use serde::{forward_to_deserialize_any, Deserialize, Serialize};

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

type CodeData<'a> = VecDeque<(&'a str, ModelValue<'a>)>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct CQCodeModel<'a> {
    #[serde(borrow)]
    pub r#type: Cow<'a, str>,
    #[serde(flatten)]
    pub data: CodeData<'a>,
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
