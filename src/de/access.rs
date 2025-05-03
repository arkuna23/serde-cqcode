use data::model::*;
use serde::de::{EnumAccess, Error as DeError, IntoDeserializer, SeqAccess, VariantAccess};

use super::core::CQDeserializer;
use serde::de::MapAccess;

use crate::*;

pub struct CQCodeAccess<'a> {
    code: CQCodeModel<'a>,
    current: Option<ModelValue<'a>>,
    first: bool,
}

impl<'a> CQCodeAccess<'a> {
    pub fn from_model(code: CQCodeModel<'a>) -> Self {
        CQCodeAccess {
            code,
            current: None,
            first: true,
        }
    }
}

impl<'de> MapAccess<'de> for CQCodeAccess<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.first {
            self.first = false;
            seed.deserialize("$type".into_deserializer()).map(Some)
        } else {
            let Some((key, value)) = self.code.data.pop_front() else {
                return Ok(None);
            };
            self.current = Some(value);
            seed.deserialize(key.into_deserializer()).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        if let Some(value) = self.current.take() {
            seed.deserialize(value)
        } else {
            seed.deserialize(self.code.cq_type.clone().into_deserializer())
        }
    }
}

pub(crate) struct CodeRaw<'a, 'de: 'a> {
    de: &'a mut CQDeserializer<'de>,
    first: bool,
}

impl<'a, 'de: 'a> CodeRaw<'a, 'de> {
    pub fn create(de: &'a mut CQDeserializer<'de>) -> Result<Self> {
        if de.peek_str(3) == "CQ:" {
            de.next_str(2);
            Ok(Self { de, first: true })
        } else {
            Err(Error::ExpectedCodeType)
        }
    }
}

impl<'a, 'de: 'a> MapAccess<'de> for CodeRaw<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.de.peek_char()? {
            ']' => Ok(None),
            ',' => {
                self.de.next_char()?;
                seed.deserialize(&mut *self.de).map(Some)
            }
            _ => {
                if self.first {
                    self.first = false;
                    seed.deserialize("$type".into_deserializer()).map(Some)
                } else {
                    Err(Error::ExpectedCodeComma)
                }
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.de.next_char()? {
            '=' | ':' => seed.deserialize(&mut *self.de),
            _ => Err(Error::ExpectedCodeColon),
        }
    }
}

impl<'de> EnumAccess<'de> for CodeRaw<'_, 'de> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(
        mut self,
        seed: V,
    ) -> std::result::Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        Ok((
            self.next_value_seed(seed)?,
            self,
        ))
    }
}

impl<'de> VariantAccess<'de> for CodeRaw<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> std::result::Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::custom("not supported"))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }
}

impl<'de> VariantAccess<'de> for CQCodeAccess<'de> {
    type Error = Error;

    fn unit_variant(self) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> std::result::Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.code)
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::UnsupportedType("tuple_variant".into()))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }
}

impl<'de> EnumAccess<'de> for CQCodeAccess<'de> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> std::result::Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let r = seed.deserialize(IntoDeserializer::<'de, Error>::into_deserializer(
            self.code.cq_type.clone(),
        ))?;
        Ok((r, self))
    }
}

pub struct CodeSeq<'a, 'de: 'a> {
    de: &'a mut CQDeserializer<'de>,
}

impl<'a, 'de: 'a> CodeSeq<'a, 'de> {
    pub fn new(de: &'a mut CQDeserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'a, 'de: 'a> SeqAccess<'de> for CodeSeq<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if !self.de.input.is_empty() && self.de.peek_char()? != ']' {
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<'a, 'de: 'a> VariantAccess<'de> for CodeSeq<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> std::result::Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        Err(Error::UnsupportedType("newtype_variant".into()))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::UnsupportedType("struct_variant".into()))
    }
}
