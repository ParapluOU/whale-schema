use super::*;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::ident_attr))]
pub struct IdentAttr(pub IdentLowercase);

impl AsRef<str> for IdentAttr {
    fn as_ref(&self) -> &str {
        self.0.value.as_ref()
    }
}

impl fmt::Display for IdentAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.value)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::attr_assign))]
pub struct AttrAssign {
    pub ident: IdentAttr,
    pub mod_opt: Option<SymbolModOpt>,
}

/// Attribute typing can be either a union or simple/compound type
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::attr_typing))]
pub enum AttrTyping {
    /// Union type: "active" | "inactive"
    Union(TypeUnion),
    /// Simple or compound type: String or String + "-" + Int
    SimpleCompound(SimpleTypingInline),
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::attrdef))]
pub struct AttrDef {
    /// optional comments before the attr def
    pub comments: Vec<Comment>,
    /// actual assignment tokens
    pub assign: AttrAssign,
    // pub mod_opt: Option<SymbolModOpt>,
    pub typing: Option<AttrTyping>,
    // optional comment at the end of the line
    pub comment: Option<CommentLine>,
}

impl AttrDef {
    pub fn is_optional(&self) -> bool {
        self.assign.mod_opt.is_some()
    }

    pub fn is_required(&self) -> bool {
        !self.is_optional()
    }
}

#[derive(Debug, Eq, PartialEq, Default, Clone, FromPest)]
#[pest_ast(rule(Rule::attributes))]
pub struct Attributes(pub Vec<AttrDef>);

impl Deref for Attributes {
    type Target = Vec<AttrDef>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::attr_item_str))]
pub struct AttrItemStr {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}

impl ToString for AttrItemStr {
    fn to_string(&self) -> String {
        self.value.clone()
    }
}

impl Deref for AttrItemStr {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::attr_item))]
pub enum AttrItem {
    /// a primitive attribute type like String, Int, etc.
    /// or an alis
    /// todo: should generics be supported here?
    Simple(TypeName),
    /// a regex definition for the attribute type
    TypeRegex(TypeRegex),
    /// a static string definition like
    /// @my-attribute: "my-value"
    AttrItemStr(AttrItemStr),
}

impl AttrItem {
    pub fn is_independent_type(&self) -> bool {
        match self {
            AttrItem::Simple(_) | AttrItem::TypeRegex(_) => true,
            _ => false,
        }
    }
}
