use super::*;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::type_regex))]
pub struct TypeRegex {
    // todo: validate regex with regex?
    #[pest_ast(outer(with(span_into_str), with(strip_delimiters)))]
    pub value: String,
}
