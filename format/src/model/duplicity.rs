use crate::ast;
use crate::ast::ModDuplicity;
use std::ops::Range;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Duplicity {
    Optional,
    Single,
    Any,
    Min1,
    Custom(Range<usize>),
}

impl From<&ast::ModDuplicity> for Duplicity {
    fn from(ast: &ModDuplicity) -> Self {
        match ast {
            ModDuplicity::Opt(_) => Self::Optional,
            ModDuplicity::Any(_) => Self::Any,
            ModDuplicity::Min(_) => Self::Min1,
            ModDuplicity::Range(range) => Self::Custom(range.into()),
        }
    }
}

impl Default for Duplicity {
    fn default() -> Self {
        Self::Single
    }
}

impl Duplicity {
    pub fn min_occurs(&self) -> usize {
        match self {
            Duplicity::Optional | Duplicity::Any => 0,
            Duplicity::Single | Duplicity::Min1 => 1,
            Duplicity::Custom(range) => range.start,
        }
    }

    pub fn max_occurs(&self) -> Option<usize> {
        match self {
            Duplicity::Optional | Duplicity::Single => Some(1),
            Duplicity::Any | Duplicity::Min1 => None,
            Duplicity::Custom(range) => Some(range.end),
        }
    }
}
