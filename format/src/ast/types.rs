use super::*;

/// type with concrete generic arguments
/// Type(String, Int)
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::type_with_generic))]
pub struct TypeWithGeneric {
    pub typename: IdentTypeNonPrimitive,
    pub args: Option<TypeArgs>,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::type_without_generic))]
pub struct TypeWithoutGeneric(pub IdentType);

impl TypeWithoutGeneric {
    pub fn is_primitive(&self) -> bool {
        matches!(self.0, IdentType::Primitive(_))
    }

    pub fn ident_nonprim(&self) -> Option<&IdentTypeNonPrimitive> {
        match &self.0 {
            IdentType::NonPrimitive(nonprim) => Some(nonprim),
            _ => None,
        }
    }

    pub fn ident(&self) -> Ident {
        self.0.clone().into()
    }
}

// impl AsRef<str> for TypeWithoutGeneric {
//     fn as_ref(&self) -> &str {
//         self.0.as_ref()
//     }
// }

impl Deref for TypeWithoutGeneric {
    type Target = IdentType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::type_with_vars))]
pub struct TypeWithVars {
    pub typename: IdentTypeNonPrimitive,
    pub args: Option<TypeDefVars>,
}

/// TypeName with optional facets
/// Examples:
/// - String (regular)
/// - String<5..20> (with facets)
/// - List(String) (generic)
/// - List(String<5..20>) (generic with faceted type arg)
#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typename))]
pub struct TypeName {
    pub base: TypeNameBase,
    pub facets: Option<Facets>,
}

#[derive(Debug, Eq, PartialEq, Clone, FromPest)]
#[pest_ast(rule(Rule::typename_base))]
pub enum TypeNameBase {
    Regular(TypeWithoutGeneric),
    Generic(TypeWithGeneric),
}

impl TypeName {
    /// when the ident cannot be a primitive and should be a complex type reference
    pub fn ident_nonprim(&self) -> Option<&IdentTypeNonPrimitive> {
        match &self.base {
            TypeNameBase::Regular(TypeWithoutGeneric(IdentType::NonPrimitive(nonprim))) => {
                Some(nonprim)
            }
            TypeNameBase::Generic(TypeWithGeneric { typename, .. }) => Some(typename),
            _ => None,
        }
    }

    /// when the ident should not be generic
    pub fn ident_regular(&self) -> Option<&IdentTypeNonPrimitive> {
        match &self.base {
            TypeNameBase::Regular(TypeWithoutGeneric(IdentType::NonPrimitive(ty))) => Some(ty),
            _ => None,
        }
    }

    /// Get the base type identifier (returns clone to avoid lifetime issues)
    pub fn base_ident(&self) -> IdentType {
        match &self.base {
            TypeNameBase::Regular(t) => t.0.clone(),
            TypeNameBase::Generic(t) => IdentType::NonPrimitive(t.typename.clone()),
        }
    }

    /// Check if this typename has facets
    pub fn has_facets(&self) -> bool {
        self.facets.is_some()
    }
}

#[derive(Debug, Eq, PartialEq, FromVariants, FromPest)]
#[pest_ast(rule(Rule::type_simple))]
pub enum TypeSimple {
    Primitive(Primitive),
    Regex(TypeRegex),
    Compound(SimpleTypingInline),
    Union(TypeUnion),
}
