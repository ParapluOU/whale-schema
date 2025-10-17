use super::*;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::simple_compound_inline))]
pub struct SimpleTypingInline(pub Vec<AttrItem>);

impl SimpleTypingInline {
    /// whether the type does not need further resolving in the AST
    pub fn is_independent_type(&self) -> bool {
        self.0.iter().all(|v| v.is_independent_type())
    }

    /// whether the type is a compound type, meaning it adds multiple restraints
    /// like "-" + String + "000"
    pub fn is_compound(&self) -> bool {
        self.0.len() > 1
    }

    /// whether any parts of the compound are generic
    pub fn is_generic(&self) -> bool {
        self.0.iter().any(|v| match v {
            AttrItem::Simple(typename) => matches!(&typename.base, ast::TypeNameBase::Generic(_)),
            _ => false,
        })
    }

    /// get the "first" item, which will be the only item if the definition is not a compound,
    /// or get the first item of the compound
    pub fn first_item(&self) -> &AttrItem {
        self.0.first().expect("no items in SimpleTypingInline")
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typing))]
pub enum Typing {
    /// union of multiple types (Int | String | "literal")
    Union(TypeUnion),
    /// type is a real type with possible generics and stuff
    Typename(TypeName),
    /// text-only element value
    Regex(TypeRegex),
    /// type variable, probably denoting a block
    Var(TypeVar),
}
