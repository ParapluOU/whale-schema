use super::*;
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::ident_element))]
pub struct IdentElement(pub IdentLowercase);

impl AsRef<str> for IdentElement {
    fn as_ref(&self) -> &str {
        self.0.value.as_ref()
    }
}

impl Display for IdentElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.value)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::element))]
pub struct Element {
    pub attributes: Attributes,
    pub item: ElementItem,
}

impl Element {
    pub fn name(&self) -> &str {
        self.ident().as_ref()
    }

    pub fn ident(&self) -> &IdentElement {
        &self.assignment().element
    }

    pub fn assignment(&self) -> &ElementAssign {
        match &self.item {
            ElementItem::WithType(ElementWithType { assign, .. }) => &assign,
            ElementItem::WithBlock(ElementWithBlock { assign, .. }) => &assign,
        }
    }

    pub fn duplicity(&self) -> Option<&ModDuplicity> {
        self.assignment().mod_dup.as_ref()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::element_assign))]
pub struct ElementAssign {
    pub element: IdentElement,
    pub mod_dup: Option<ModDuplicity>,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::element_item))]
pub enum ElementItem {
    WithType(ElementWithType),
    WithBlock(ElementWithBlock),
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::element_with_type))]
pub struct ElementWithType {
    pub assign: ElementAssign,
    pub typing: Typing,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::element_with_block))]
pub struct ElementWithBlock {
    pub assign: ElementAssign,
    pub block: Block,
}
