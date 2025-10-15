use super::*;

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::sym_attr))]
pub struct SymbolAttr {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::sym_mod_opt))]
pub struct SymbolModOpt {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::sym_mod_any))]
pub struct SymbolModAny {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::sym_mod_min1))]
pub struct SymbolModMin1 {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::sym_mod_range))]
pub struct SymbolModRange {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, FromPest)]
#[pest_ast(rule(Rule::mod_range))]
pub enum ModRange {
    Span(ModRangeSpan),
    Static(Uint),
}

impl Into<Range<usize>> for &ModRange {
    fn into(self) -> Range<usize> {
        match self {
            ModRange::Span(ModRangeSpan { from, to }) => (from.value..to.value),
            ModRange::Static(num) => (num.value..num.value),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, FromPest)]
#[pest_ast(rule(Rule::mod_range_span))]
pub struct ModRangeSpan {
    pub from: Uint,
    pub to: Uint,
}

#[derive(Debug, Eq, Clone, Copy, PartialEq, FromPest)]
#[pest_ast(rule(Rule::uint))]
pub struct Uint {
    #[pest_ast(outer(with(span_into_str), with(str::parse), with(Result::unwrap)))]
    pub value: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::mod_duplicity))]
pub enum ModDuplicity {
    Opt(SymbolModOpt),
    Any(SymbolModAny),
    Min(SymbolModMin1),
    Range(ModRange),
}
