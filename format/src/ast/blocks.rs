use super::*;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::block))]
pub struct Block {
    /// modifiers, like whether the block should allow mixed content,
    /// and whether it should act as a <xs:sequence> or a <xs:choice> or <xs:all>
    pub mods: BlockMods,
    /// all sub-items of the block
    pub items: Vec<BlockItem>,
}

impl Deref for Block {
    type Target = BlockMods;

    fn deref(&self) -> &Self::Target {
        &self.mods
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::block_item))]
pub enum BlockItem {
    /// this block item is a nested element
    Element(Element),
    /// another block definition is flattened into this definition
    SplatBlock(SplatBlock),
    SplatType(SplatType),
    /// a generic variable is splat into the block
    SplatGenericArg(SplatGenericVar),
    /// comment node
    Comment(Comment),
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::block_mods))]
pub struct BlockMods {
    /// whether the type is abstract (cannot be instantiated)
    pub abstract_mod: Option<BlockModAbstract>,
    /// whether the block allows mixed content
    pub mixed_prefix: Option<BlockModMixed>,
    /// whether the block is a <xs:sequence>, <xs:all> or <xs:choice>
    /// defaults to sequence
    pub occurrence: Option<BlockModOccurrence>,
    /// whether the block allows mixed content
    pub mixed_postfix: Option<BlockModMixed>,
}

impl BlockMods {
    pub fn is_abstract(&self) -> bool {
        self.abstract_mod.is_some()
    }

    pub fn is_mixed_content(&self) -> bool {
        self.mixed_prefix.is_some() || self.mixed_postfix.is_some()
    }

    pub fn is_sequence(&self) -> bool {
        self.occurrence.is_none()
    }

    pub fn is_all(&self) -> bool {
        self.occurrence
            .as_ref()
            .map(|m| m.is_must())
            .unwrap_or_default()
    }

    pub fn is_choice(&self) -> bool {
        self.occurrence
            .as_ref()
            .map(|m| m.is_opt())
            .unwrap_or_default()
    }
}

/// whether the block is a xs:sequence (default, no mods)
/// or a xs:choice (=Opt) OR MUST have all elements in any order
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::mod_occurrence))]
pub enum BlockModOccurrence {
    /// choice
    Opt(BlockModOpt),
    /// all
    Must(BlockModMust),
}

impl BlockModOccurrence {
    pub fn is_opt(&self) -> bool {
        matches!(self, BlockModOccurrence::Opt(_))
    }

    pub fn is_must(&self) -> bool {
        matches!(self, BlockModOccurrence::Must(_))
    }
}

/// block modifier indicating xs:choice
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::sym_mod_opt))]
pub struct BlockModOpt {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}

/// block modifier indicating xs:all
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::sym_mod_must))]
pub struct BlockModMust {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}

/// block modifier indicating @mixed=true
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::mod_mixed))]
pub struct BlockModMixed {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}

/// block modifier indicating abstract type
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::mod_abstract))]
pub struct BlockModAbstract {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}
