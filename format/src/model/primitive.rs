use crate::ast;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(
    Debug,
    Hash,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::EnumIter,
)]
pub enum PrimitiveType {
    String,
    URI,
    DateTimestamp,
    DateTime,
    Date,
    Time,
    Duration,
    Bool,
    Int,
    Float,
    Double,
    Short,
    Decimal,
    IDRefs,
    IDRef,
    ID,
    Lang,
    NoColName,
    /// negative int
    IntNeg,
    /// int that is >= 0
    IntNonNeg,
    /// int that is > 0
    IntPos,
    Token,
    NameTokens,
    NameToken,
    Name,
    Base64Binary,
    UnsignedLong,
    AnySimpleType,
}

impl Default for PrimitiveType {
    fn default() -> Self {
        Self::String
    }
}

impl PrimitiveType {
    pub fn parse(ast: &ast::Primitive) -> anyhow::Result<Self> {
        // Ok(match ast.value.as_str() {
        //     "String" => Self::String,
        //     _ => todo!(),
        // })
        Ok(Self::from_str(ast.value.as_str())?)
    }
}

impl From<&ast::Primitive> for PrimitiveType {
    fn from(ast: &ast::Primitive) -> Self {
        let ast_str = ast.value.as_str();
        let err = format!("could not parse {} into Primitive", ast_str);

        // if the ast_str starts with, and ends with square brackets,
        // then take the string within it and add an 's' to the end
        if ast_str.starts_with('[') && ast_str.ends_with(']') {
            let inner = &ast_str[1..ast_str.len() - 1];
            let mut inner = inner.to_string();
            inner.push('s');
            return Self::from_str(inner.as_str()).expect(err.as_str());
        }

        // todo: support alias attribute on the Primitive enum
        if ast_str == "+Int" {
            return Self::IntPos;
        }

        // todo: support alias attribute on the Primitive enum
        if ast_str == "-Int" {
            return Self::IntNeg;
        }

        // todo: support alias attribute on the Primitive enum
        if ast_str == "Boolean" {
            return Self::Bool;
        }

        // todo: support alias attribute on the Primitive enum
        if ast_str == "Integer" {
            return Self::Int;
        }

        // todo: support alias attribute on the Primitive enum
        // if ast_str == "Integer" {
        //     return Self::DateTimestamp;
        // }

        Self::from_str(ast_str).expect(err.as_str())
    }
}
