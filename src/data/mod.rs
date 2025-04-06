pub mod model;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum Number {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum CodeValue {
    Bool(bool),
    String(String),
    Number(Number),
    CQCode(CQCode),
}

#[derive(Debug, thiserror::Error)]
pub enum NumberError {
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
}

impl Number {
    pub fn parse(s: impl AsRef<str>) -> Result<Self, NumberError> {
        let s = s.as_ref();
        if s.contains('.') {
            s.parse::<f64>()
                .map(Number::Float)
                .map_err(NumberError::from)
        } else {
            s.parse::<i64>().map(Number::Int).map_err(NumberError::from)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CQCode {
    pub r#type: String,
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

impl CQCode {
    pub fn new(ty: impl Into<String>) -> Self {
        CQCode {
            r#type: ty.into(),
            data: HashMap::default(),
        }
    }

    pub fn data_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.data
    }

    pub fn data(&self) -> &HashMap<String, String> {
        &self.data
    }
}

impl<T, V, VS> From<(T, V)> for CQCode
where
    V: IntoIterator<Item = (VS, VS)>,
    T: Into<String>,
    VS: Into<String>,
{
    fn from(value: (T, V)) -> Self {
        let (ty, data) = value;
        let mut cq_code = CQCode::new(ty.into());
        for (key, value) in data {
            cq_code.data.insert(key.into(), value.into());
        }
        cq_code
    }
}

pub fn parse_code(code: &str) -> Option<char> {
    match code {
        "amp" => Some('&'),
        "#91" => Some('['),
        "#93" => Some(']'),
        "#44" => Some(','),
        _ => None,
    }
}

pub const fn escape_char(code: char) -> Option<&'static str> {
    match code {
        '&' => Some("amp"),
        '[' => Some("#91"),
        ']' => Some("#93"),
        ',' => Some("#44"),
        _ => None,
    }
}
