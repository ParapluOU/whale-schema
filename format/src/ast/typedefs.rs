use super::*;

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typedef))]
pub enum TypeDef {
    Inline(TypeDefInline),
    Block(TypeDefBlock),
}

impl TypeDef {
    pub fn ident_nonprim(&self) -> &IdentTypeNonPrimitive {
        match self {
            TypeDef::Inline(item) => &item.typename,
            TypeDef::Block(item) => &item.typename,
        }
    }

    pub fn ident(&self) -> Ident {
        Ident::Type(IdentType::from(self.ident_nonprim().clone()))
    }

    pub fn is_named(&self, name: &IdentTypeNonPrimitive) -> bool {
        name == self.ident_nonprim()
    }

    pub fn has_name(&self, name: impl AsRef<str>) -> bool {
        self.ident_nonprim().as_ref() == name.as_ref()
    }

    pub fn attributes(&self) -> Attributes {
        match self {
            TypeDef::Inline(_) => default(), // no attributes support
            TypeDef::Block(block) => block.attributes.clone(),
        }
    }

    pub fn type_variant(&self, ast: &ast::SchemaFile) -> anyhow::Result<model::TypeVariant> {
        Ok(match self.simple_type(ast)? {
            Some(simple) => model::TypeVariant::Simple,
            None => model::TypeVariant::Group,
        })
    }

    // todo: instead of passing around the reference path everywhere,
    // it shoujld be part of the schema struct that we are passing.
    // but because that is currently an AST node, it cannot support that
    // so we have to make a wrapper managed by a schema manager,
    // but that requires refactoring the compiler
    pub fn simple_type(&self, schema: &ast::SchemaFile) -> anyhow::Result<Option<TypeSimple>> {
        match self {
            TypeDef::Inline(TypeDefInline { typing, .. }) => {
                // resolve typename.  return true if at the end the type does not refer to a block
                match typing {
                    TypeDefInlineTyping::Var(_) => {
                        todo!("how do we know whether a typevar is a simpletype or not?")
                    }
                    TypeDefInlineTyping::SimpleType(compound) => {
                        return Ok(Some((*compound).clone().into()));
                    }
                    TypeDefInlineTyping::Typename(ty) => {
                        match ty {
                            TypeName::Regular(TypeWithoutGeneric(IdentType::Primitive(prim))) => {
                                return Ok(Some((*prim).clone().into()));
                            }
                            // if its a custom type reference, we have to look up _that_ type now
                            TypeName::Regular(TypeWithoutGeneric(IdentType::NonPrimitive(
                                nonprim,
                            ))) => {
                                println!("resolving subtype {:?}...\n", ty);
                                return schema
                                    .find_type(nonprim)
                                    .ok_or(anyhow!("could not find Type declaration for '{}' when resolving type {:#?}", nonprim, typing))?
                                    .simple_type(schema);
                            }
                            TypeName::Generic(_generic_ty) => {
                                todo!()
                            }
                        }
                    }
                }
            }
            // blocks and unmatched cases. noop
            _ => {
                // println!("is not TypeDefInline!\n");
                // noop
            }
        }

        Ok(None)
    }

    pub fn is_simple_type(&self, schema: &SchemaFile) -> anyhow::Result<bool> {
        self.simple_type(schema).map(|v| v.is_some())
    }

    pub fn is_group(&self) -> bool {
        match self {
            TypeDef::Inline(_) => false,
            TypeDef::Block(_) => true,
        }
    }
}

impl PartialOrd<Self> for TypeDef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TypeDef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ident_nonprim().cmp(&other.ident_nonprim())
    }
}

impl AsRef<IdentTypeNonPrimitive> for TypeDef {
    fn as_ref(&self) -> &IdentTypeNonPrimitive {
        self.ident_nonprim()
    }
}

#[derive(Debug, Eq, Clone, PartialEq, FromPest)]
#[pest_ast(rule(Rule::typedef_inline))]
pub struct TypeDefInline {
    pub typename: IdentTypeNonPrimitive,
    pub vars: Option<TypeDefVars>,
    pub typing: TypeDefInlineTyping,
}

impl TypeDefInline {
    pub fn is_generic(&self) -> bool {
        if let Some(vars) = self.vars.as_ref() && !vars.0.is_empty() {
            return true;
        }
        false
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typedef_inline_typing))]
pub enum TypeDefInlineTyping {
    // type reference, still unknown if its for attribute or block
    Typename(TypeName),
    // type varibale
    Var(TypeVar),
    // compound simpletype
    SimpleType(SimpleTypingInline),
}

/// Inheritance clause: < BaseType or < BaseType(Arg)
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::inheritance))]
pub struct Inheritance {
    pub base_type: TypeName,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typedef_block))]
pub struct TypeDefBlock {
    pub attributes: Attributes,
    pub typename: IdentTypeNonPrimitive,
    pub vars: Option<TypeDefVars>,
    pub inheritance: Option<Inheritance>,
    pub block: Block,
}

impl TypeDefBlock {
    pub fn is_generic(&self) -> bool {
        if let Some(vars) = self.vars.as_ref() && !vars.0.is_empty() {
            return true;
        }
        false
    }
}
