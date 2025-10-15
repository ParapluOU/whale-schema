use crate::formats::fonto;
use crate::model;
use crate::model::PrimitiveType;
use serde::{Deserialize, Serialize};

// see reference:
// fonto/platform/fontoxml-schema/src/simple-types/builtins/builtinModels.js

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Primitive {
    String,
    #[serde(rename = "anyURI")]
    URI,
    AnySimpleType,
    Date,
    DateTime,
    DateTimeStamp,
    Time,
    Duration,
    Boolean,
    Integer,
    Float,
    Double,
    Short,
    Decimal,
    #[serde(rename = "ID")]
    ID,
    #[serde(rename = "IDREF")]
    IDRef,
    #[serde(rename = "IDREFS")]
    IDRefs,
    Language,
    #[serde(rename = "Name")]
    Name,
    #[serde(rename = "NCName")]
    NoColName,
    /// negative integer
    NegativeInteger,
    /// integer than can be 0
    NonNegativeInteger,
    /// integer that is > 0
    PositiveInteger,
    UnsignedLong,
    Base64Binary,
    Token,
    #[serde(rename = "NMTOKEN")]
    NameToken,
    #[serde(rename = "NMTOKENS")]
    NameTokens,
}

impl Default for Primitive {
    fn default() -> Self {
        Self::String
    }
}

impl From<&model::PrimitiveType> for fonto::Primitive {
    fn from(value: &model::PrimitiveType) -> Self {
        match value {
            PrimitiveType::String => Primitive::String,
            PrimitiveType::URI => Primitive::URI,
            PrimitiveType::AnySimpleType => Primitive::AnySimpleType,
            PrimitiveType::Date => Primitive::Date,
            PrimitiveType::DateTime => Primitive::DateTime,
            PrimitiveType::DateTimestamp => Primitive::DateTimeStamp,
            PrimitiveType::Time => Primitive::Time,
            PrimitiveType::Duration => Primitive::Duration,
            PrimitiveType::Bool => Primitive::Boolean,
            PrimitiveType::Int => Primitive::Integer,
            PrimitiveType::Float => Primitive::Float,
            PrimitiveType::Double => Primitive::Double,
            PrimitiveType::Short => Primitive::Short,
            PrimitiveType::Decimal => Primitive::Decimal,
            PrimitiveType::ID => Primitive::ID,
            PrimitiveType::IDRef => Primitive::IDRef,
            PrimitiveType::IDRefs => Primitive::IDRefs,
            PrimitiveType::Lang => Primitive::Language,
            PrimitiveType::Name => Primitive::Name,
            PrimitiveType::NoColName => Primitive::NoColName,
            PrimitiveType::IntNeg => Primitive::NegativeInteger,
            PrimitiveType::IntNonNeg => Primitive::NonNegativeInteger,
            PrimitiveType::IntPos => Primitive::PositiveInteger,
            PrimitiveType::UnsignedLong => Primitive::UnsignedLong,
            PrimitiveType::Base64Binary => Primitive::Base64Binary,
            PrimitiveType::Token => Primitive::Token,
            PrimitiveType::NameToken => Primitive::NameToken,
            PrimitiveType::NameTokens => Primitive::NameTokens,
        }
    }
}
