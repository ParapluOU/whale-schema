use super::*;

// #[derive(Debug, FromPest)]
// #[pest_ast(rule(Rule::SOI))]
// pub struct FileStart;

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::EOI))]
pub struct FileEnd;
