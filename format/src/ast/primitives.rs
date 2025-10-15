use super::*;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::primitive))]
pub struct Primitive {
    /// todo: parse to enum for primitive
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

impl AsRef<str> for Primitive {
    fn as_ref(&self) -> &str {
        self.value.as_str()
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::ident_type_nonprimitive))]
pub struct NonPrimitive(pub IdentCapitalized);
