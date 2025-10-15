use super::*;

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::keyword))]
pub struct Keyword {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub token: String,
}
