use std::{borrow::Cow, collections::VecDeque};

use crate::{data::*, Result, *};
use model::*;
use serde::{forward_to_deserialize_any, Deserialize};

use super::access::*;

pub struct CQDeserializer<'de> {
    pub(crate) input: &'de str,
}

impl<'de> CQDeserializer<'de> {
    /// Creates a new `Deserializer` with the given input string.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice that holds the input to be deserialized.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Deserializer`.
    pub fn new(input: &'de str) -> Self {
        Self { input }
    }

    /// Peeks at the next character in the input without consuming it.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the next character if successful, or an `Error::Eof`
    /// if the input is empty.
    pub fn peek_char(&self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    /// Consumes and returns the next character in the input.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the next character if successful, or an `Error::Eof`
    /// if the input is empty.
    pub fn next_char(&mut self) -> Result<char> {
        if let Some((i, ch)) = self.input.char_indices().next() {
            self.input = &self.input[i + ch.len_utf8()..];
            Ok(ch)
        } else {
            Err(Error::Eof)
        }
    }

    pub fn peek_str_until(&self, until: char) -> Result<&str> {
        if let Some(pos) = self.input.find(until) {
            Ok(&self.input[..pos])
        } else {
            Err(Error::ExpectedDelimiter)
        }
    }

    /// Consumes and returns the input string until a specified delimiter is found.
    ///
    /// # Arguments
    ///
    /// * `until` - The character delimiter to search for.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the substring up to the delimiter if successful, or an
    /// `Error::ExpectedCodeComma` if the delimiter is not found.
    pub fn next_str_until(&mut self, until: char) -> Result<&str> {
        if let Some(pos) = self.input.find(until) {
            let result = &self.input[..pos];
            self.input = &self.input[pos..];
            Ok(result)
        } else {
            Err(Error::ExpectedCodeComma)
        }
    }

    /// Peeks at a substring of the input with a specified maximum length, without consuming it.
    ///
    /// # Arguments
    ///
    /// * `max_len` - The maximum length of the substring to peek.
    ///
    /// # Returns
    ///
    /// Returns a substring of the input with a length up to `max_len`.
    pub fn peek_str(&self, max_len: usize) -> &str {
        let len = if max_len > self.input.len() {
            self.input.len()
        } else {
            max_len
        };
        &self.input[..len]
    }

    /// Returns the next substring from the input with a maximum length.
    ///
    /// This function extracts a substring from the input with a length up to `max_len`.
    /// If `max_len` is greater than the remaining input length, it returns the rest of the input.
    ///
    /// # Arguments
    ///
    /// * `max_len` - The maximum length of the substring to extract.
    ///
    /// # Returns
    ///
    /// Returns the extracted substring.
    pub fn next_str(&mut self, max_len: usize) -> &str {
        let len = if max_len > self.input.len() {
            self.input.len()
        } else {
            max_len
        };
        let result = &self.input[..len];
        self.input = &self.input[len..];
        result
    }

    pub fn peek_escaping(&self) -> Result<char> {
        let code = self.peek_str_until(';')?;
        let ch = if let Some(ch) = parse_code(code) {
            ch
        } else {
            return Err(Error::UnknownEscaping(code.to_owned()));
        };

        Ok(ch)
    }

    pub fn peek_escaped_char(&self) -> Result<char> {
        match self.peek_char()? {
            '&' => self.peek_escaping(),
            ch => Ok(ch),
        }
    }

    pub fn next_escaping(&mut self) -> Result<char> {
        let code = self.next_str_until(';')?;
        let ch = if let Some(ch) = parse_code(code) {
            ch
        } else {
            return Err(Error::UnknownEscaping(code.to_owned()));
        };

        self.next_char()?;
        Ok(ch)
    }

    pub fn next_escaped_char(&mut self) -> Result<char> {
        match self.next_char()? {
            '&' => self.next_escaping(),
            ch => Ok(ch),
        }
    }

    pub fn peek_delimiter(&self) -> Option<char> {
        for ch in self.input.chars() {
            if let '[' | ']' | ',' | '=' = ch {
                return Some(ch);
            }
        }

        None
    }

    pub fn next_raw_str(&mut self) -> &str {
        let end_pos = self.input.find([']', ',', '=']).unwrap_or(self.input.len());
        let result = &self.input[..end_pos];
        self.input = &self.input[end_pos..];
        result
    }

    /// Parses the next string from the input until a delimiter is encountered.
    ///
    /// This function reads characters from the input and constructs a string
    /// until it encounters one of the delimiters: ']', ',', or '='. If an '&'
    /// character is encountered, it attempts to parse an escape sequence.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed string if successful, or an
    /// `Error` if the end of the input is reached without encountering a
    /// delimiter.
    pub fn next_escaped_string(&mut self) -> Result<String> {
        let mut str = String::new();

        while let Ok(ch) = self.peek_char() {
            match ch {
                ']' | ',' | '=' => break,
                '&' => {
                    self.next_char()?;
                    str.push(self.next_escaped_char()?);
                }
                _ => {
                    str.push(self.next_char()?);
                }
            }
        }

        Ok(str)
    }

    /// Parses a text message from the input until a '[' character is encountered.
    ///
    /// This function constructs a `CQCode` with the type "text" and the accumulated
    /// message data. It handles escape sequences starting with '&' by calling the
    /// `escaping` method. If the end of the input is reached without encountering
    /// a '[', it returns an `Error::Eof`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `CQCode` if successful, or an `Error::Eof`
    /// if the end of the input is reached without encountering a '['.
    pub fn parse_text_msg(&mut self) -> Result<CQCodeModel<'de>> {
        let mut msg = String::new();

        while let Ok(ch) = self.peek_char() {
            match ch {
                '[' => {
                    break;
                }
                '&' => {
                    self.next_char()?;
                    msg.push(self.next_escaping()?);
                }
                _ => {
                    msg.push(self.next_char()?);
                }
            }
        }

        Ok(CQCodeModel {
            r#type: Cow::Borrowed("text"),
            data: VecDeque::from([("text", ModelValue::String(Cow::Owned(msg)))]),
        })
    }
}

impl<'de> serde::de::Deserializer<'de> for &mut CQDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.peek_char()? == '[' {
            self.next_char()?;
            let map = CodeRaw::create(&mut *self)?;
            let result = visitor.visit_map(map)?;
            if self.peek_char()? != ']' {
                Err(Error::ExpectedCodeEnd)
            } else {
                Ok(result)
            }
        } else if let Some('[') | None = self.peek_delimiter() {
            visitor.visit_map(self.parse_text_msg()?.into_access())
        } else {
            self.deserialize_str(visitor)
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.peek_char()? == '[' {
            self.next_char()?;
            let map = CodeRaw::create(&mut *self)?;
            let result = visitor.visit_map(map)?;
            if self.next_char()? != ']' {
                Err(Error::ExpectedCodeEnd)
            } else {
                Ok(result)
            }
        } else {
            visitor.visit_map(self.parse_text_msg()?.into_access())
        }
    }

    forward_to_deserialize_any! {
        struct
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.next_raw_str() {
            "1" | "yes" | "true" => visitor.visit_bool(true),
            "0" | "no" | "false" => visitor.visit_bool(false),
            v => Err(Error::UnknownValue(v.into(), "bool".into())),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.next_escaped_string()
            .and_then(|r| visitor.visit_string(r))
    }

    fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value: i64 = self.next_raw_str().parse()?;
        visitor.visit_i64(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.next_raw_str();
        visitor.visit_f64(value.parse()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.next_escaped_string()?;
        let mut chars = value.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            visitor.visit_char(ch)
        } else {
            Err(Error::MismatchedValueType(value, "char".into()))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.next_escaped_string()?;
        visitor.visit_string(value)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.next_raw_str();
        visitor.visit_bytes(value.as_bytes())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.next_raw_str();
        visitor.visit_byte_buf(value.as_bytes().to_vec())
    }

    fn deserialize_option<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::UnsupportedType("option".into()))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::UnsupportedType("unit".into()))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(CodeSeq::new(self))
    }

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(CQCodeModel::deserialize(self)?.into_access())
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}
