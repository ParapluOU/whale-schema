use super::*;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::splat_block))]
pub struct SplatBlock(pub Block);

impl AsRef<Block> for SplatBlock {
    fn as_ref(&self) -> &Block {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::splat_type))]
pub struct SplatType(pub TypeName);

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::splat_generic_var))]
pub struct SplatGenericVar(pub TypeVar);
