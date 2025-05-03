use crate::*;
use data::escape_char;
use ser::util::{NewtypeSer, StrSerializer};
use serde::{
    de::Error as DeError,
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serializer,
};

#[derive(Debug, Clone, Default)]
pub struct CQSerializer {
    pub(crate) output: String,
}

impl CQSerializer {
    fn append_value_display(&mut self, dis: impl AsRef<str>) {
        self.output.push('=');
        self.output.push_str(dis.as_ref());
    }

    fn push_char(&mut self, c: char) {
        if let Some(code) = escape_char(c) {
            self.output.push('&');
            self.output.push_str(code);
            self.output.push(';');
        } else {
            self.output.push(c);
        }
    }

    pub fn finish(self) -> String {
        self.output
    }
}

#[derive(Debug)]
pub struct StructSerializer<'a> {
    pub(crate) is_text: bool,
    pub(crate) last_key: Option<String>,
    pub(crate) ser: &'a mut CQSerializer,
}

impl<'a> StructSerializer<'a> {
    pub fn new(ty: impl AsRef<str>, ser: &'a mut CQSerializer) -> Self {
        let ty = ty.as_ref();
        if ty == "text" {
            Self {
                is_text: true,
                ser,
                last_key: None,
            }
        } else {
            ser.output.push_str("[CQ:");
            ser.output.push_str(ty.as_ref());
            Self {
                is_text: false,
                last_key: None,
                ser,
            }
        }
    }

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        if !self.is_text {
            self.ser.output.push(',');
            key.serialize(&mut *self.ser)
        } else {
            let s = key.serialize(&StrSerializer)?;
            self.last_key = Some(s);
            Ok(())
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        if self.is_text {
            if let Some("text") = self.last_key.as_deref() {
                value.serialize(&mut *self.ser)
            } else {
                Ok(())
            }
        } else {
            self.ser.output.push('=');
            value.serialize(&mut *self.ser)
        }
    }

    fn end(self) -> Result<(), Error> {
        if !self.is_text {
            self.ser.output.push(']');
        }
        Ok(())
    }
}

impl SerializeStruct for StructSerializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_key(&key)?;
        self.serialize_value(value)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.end()
    }
}

impl SerializeStructVariant for StructSerializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_key(&key)?;
        self.serialize_value(value)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.end()
    }
}

pub struct MapSerializer<'a> {
    ser: &'a mut CQSerializer,
    seeds: Vec<(String, Option<String>)>,
}

impl<'a> MapSerializer<'a> {
    pub fn new(ser: &'a mut CQSerializer) -> Self {
        Self {
            ser,
            seeds: Default::default(),
        }
    }
}

impl SerializeMap for MapSerializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.seeds.push((key.serialize(&StrSerializer)?, None));
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.seeds.last_mut().unwrap().1.replace({
            let mut ser = CQSerializer::default();
            value.serialize(&mut ser)?;
            ser.output
        });
        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        let (idx, ty) = self
            .seeds
            .iter()
            .enumerate()
            .find(|r| r.1 .0 == "$type")
            .map(|r| (r.0, r.1 .1.as_deref().unwrap()))
            .ok_or_else(|| Error::missing_field("$type"))?;
        let mut st = StructSerializer::new(ty, self.ser);
        for (i, (key, value)) in self.seeds.into_iter().enumerate() {
            if i != idx {
                st.serialize_key(&key)?;
                if !st.is_text {
                    st.ser.append_value_display(value.unwrap());
                } else if key == "text" {
                    st.ser.output.push_str(&value.unwrap());
                }
            }
        }
        st.end()
    }
}

impl SerializeSeq for &mut CQSerializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTuple for &mut CQSerializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleStruct for &mut CQSerializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleVariant for &mut CQSerializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> Serializer for &'a mut CQSerializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = MapSerializer<'a>;

    type SerializeStruct = StructSerializer<'a>;

    type SerializeStructVariant = StructSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(if v { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.push_char(v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        v.chars().for_each(|c| self.push_char(c));
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!("serialize bytes")
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        StructSerializer::new(name, self).end()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        StructSerializer::new(variant, self).end()
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(NewtypeSer::new(name, self))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(NewtypeSer::new(variant, self))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer::new(self))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(StructSerializer::new(name, self))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(StructSerializer::new(variant, self))
    }
}
