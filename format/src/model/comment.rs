use crate::ast;
use crate::model::attr::Attributes;
use crate::model::duplicity::Duplicity;
use crate::model::r#type::Type;
use crate::model::{Ref, TypeRef};
use derive_builder::Builder;
use derive_getters::Getters;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Builder, Getters)]
pub struct Comment {
    text: String,
}

impl From<&ast::Comment> for Comment {
    fn from(ast: &ast::Comment) -> Self {
        Self {
            text: ast.to_string(),
        }
    }
}
