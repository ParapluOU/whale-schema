use super::*;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::ident_lowercase))]
pub struct IdentLowercase {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

#[derive(Debug, Eq, PartialEq, FromPest, Ord, Clone, PartialOrd)]
#[pest_ast(rule(Rule::ident_capitalized))]
pub struct IdentCapitalized {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

#[derive(Debug, Eq, PartialEq, Clone, FromVariants, FromPest)]
#[pest_ast(rule(Rule::ident_type))]
pub enum IdentType {
    Primitive(Primitive),
    NonPrimitive(IdentTypeNonPrimitive),
}

impl AsRef<str> for IdentType {
    fn as_ref(&self) -> &str {
        match self {
            IdentType::Primitive(p) => p.as_ref(),
            IdentType::NonPrimitive(np) => np.as_ref(),
        }
    }
}

impl fmt::Display for IdentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentType::Primitive(p) => p.fmt(f),
            IdentType::NonPrimitive(np) => np.fmt(f),
        }
    }
}

#[derive(Debug, Eq, PartialEq, FromPest, Ord, Clone, PartialOrd)]
#[pest_ast(rule(Rule::ident_type_nonprimitive))]
pub struct IdentTypeNonPrimitive(pub IdentCapitalized);

impl std::fmt::Display for IdentTypeNonPrimitive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.value)
    }
}

impl AsRef<str> for IdentTypeNonPrimitive {
    fn as_ref(&self) -> &str {
        self.0.value.as_ref()
    }
}

impl Into<Ident> for IdentTypeNonPrimitive {
    fn into(self) -> Ident {
        Ident::Type(IdentType::NonPrimitive(self))
    }
}

impl Into<Ident> for &IdentTypeNonPrimitive {
    fn into(self) -> Ident {
        Ident::Type(IdentType::NonPrimitive(self.clone()))
    }
}

// abstract
#[derive(Debug, Eq, PartialEq, FromVariants, FromPest)]
#[pest_ast(rule(Rule::ident))]
pub enum Ident {
    Element(IdentElement),
    Attr(IdentAttr),
    Type(IdentType),
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ident::Element(e) => e.fmt(f),
            Ident::Attr(a) => a.fmt(f),
            Ident::Type(t) => t.fmt(f),
        }
    }
}
