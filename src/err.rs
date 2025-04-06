use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("unsupported type: {0}")]
    UnsupportedType(String),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error("expected cq code")]
    ExpectedCQCode,
    #[error("mismatched value: expected {1}, found {0}")]
    MismatchedValueType(String, String),
    #[error("missing cq code type")]
    MissingCodeType,
    #[error("missing value")]
    MissingValue,
    #[error("expected delimiter")]
    ExpectedDelimiter,
    #[error("expected cq code type")]
    ExpectedCodeType,
    #[error("expected code comma")]
    ExpectedCodeComma,
    #[error("expected code colon")]
    ExpectedCodeColon,
    #[error("expected code start")]
    ExpectedCodeStart,
    #[error("expected code end")]
    ExpectedCodeEnd,
    #[error("unknown escaping")]
    UnknownEscaping(String),
    #[error("expected value type {1}, found value {0}")]
    UnknownValue(String, String),
    #[error("end of input")]
    Eof,
    #[error("{0}")]
    Custom(String),
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}
