use super::*;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typevar))]
pub struct TypeVar(pub IdentLowercase);

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typedef_vars))]
pub struct TypeDefVars(pub Vec<TypeVar>);

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typearg))]
pub struct TypeArg(pub Box<TypeName>);

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::type_args))]
pub struct TypeArgs(pub Vec<TypeArg>);
