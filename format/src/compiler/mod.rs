mod result;

use crate::ast::{
    AttrItem, BlockItem, ElementItem, IdentType, SimpleTypingInline, TypeDef, TypeDefInlineTyping,
    TypeName, TypeWithoutGeneric, Typing,
};
use crate::model::{GroupBuilder, Ref, SchemaObjId, SimpleType};
use crate::model::{Schema, TypeRef};
use crate::sourced::SourcedSchemaFile;
use crate::tools::default;
use crate::{ast, model, tools};
use anyhow::anyhow;
use from_pest::log::info;
use itertools::Itertools;
use log::debug;
use result::CompileResult;
use std::convert::identity;
use std::ops::Deref;

pub fn compile(source: &SourcedSchemaFile) -> anyhow::Result<model::Schema> {
    // the target schema we are building
    let mut schema = model::Schema::default();

    // define all types using an ID so they can be recursively resolved
    compile_type_definitions(source, &mut schema)?;

    // finally, define all elements
    compile_elements(source, &mut schema)?;

    Ok(schema)
}

pub fn compile_type_definitions(
    source: &SourcedSchemaFile,
    schema: &mut model::Schema,
) -> anyhow::Result<()> {
    info!("compiling type definitions...");

    // define all types using an ID so they can be recursively resolved
    for typedef in source.types().iter().sorted() {
        // schema.register_type_definition_name(&typedef)?;
        compile_type_definition(source, schema, typedef)?;
    }

    Ok(())
}

pub fn compile_type_definition(
    source: &SourcedSchemaFile,
    schema: &mut Schema,
    typedef: &ast::TypeDef,
) -> anyhow::Result<model::TypeRef> {
    info!("compiling type definition {}...", typedef.ident());

    // if type is already defined with this name, short-circuit and return known ref
    if let Some(existing) = schema.preliminary_ref_for_typename(&typedef, source) {
        return Ok(existing.get_ref());
    }

    debug!(
        "no preliminary type reference found for '{}'. resolving new",
        typedef.ident()
    );

    let new_id = SchemaObjId::new();

    // register name with an ID that will have no type info attached yet
    schema.register_type_definition_name(&new_id, typedef)?;

    assert!(
        schema
            .preliminary_ref_for_typename(&typedef, source)
            .is_some(),
        "it should now be possible to retrieve a priliminary type reference because we just regstered the type"
    );

    // helper to panic on nth occurrence of recursion without having resolved the type name
    // which should have totally happened by now
    // tools::panic_nth(&typedef.ident().to_string(), 2);

    let target_ty = match typedef {
        ast::TypeDef::Inline(ty_inline) => compile_inline_type(source, ty_inline, schema)?,
        ast::TypeDef::Block(blockdef) => {
            compile_block_definition(source, &blockdef, schema)?.into()
        }
    };

    // resolve unnamed part of the type definition and
    let type_ref = schema.register_preliminary_id_type(&new_id, target_ty)?;

    Ok(type_ref)
}

/// Compile inheritance clause and validate circular dependencies
pub fn compile_inheritance(
    source: &SourcedSchemaFile,
    blockdef: &ast::TypeDefBlock,
    inheritance: &ast::Inheritance,
    schema: &mut Schema,
) -> anyhow::Result<Ref<model::Group>> {
    info!("compiling inheritance from {:?}...", inheritance.base_type);

    // Resolve the base type name to a type definition
    let base_type_name = inheritance.base_type.ident_nonprim()
        .ok_or(anyhow!("Base type must be a non-primitive type"))?;

    let base_typedef = source.find_type(base_type_name)
        .ok_or(anyhow!("Base type '{}' not found", base_type_name))?;

    // Base type must be a block definition (complex type), not an inline simple type
    let base_block = match base_typedef {
        ast::TypeDef::Block(block) => block,
        ast::TypeDef::Inline(_) => {
            return Err(anyhow!(
                "Cannot inherit from simple type '{}'. Only complex types (blocks) support inheritance.",
                base_type_name
            ));
        }
    };

    // Detect circular inheritance
    validate_no_circular_inheritance(source, blockdef, base_block)?;

    // Compile the base type
    compile_block_definition(source, base_block, schema)
}

/// Validate that there are no circular inheritance chains
fn validate_no_circular_inheritance(
    source: &SourcedSchemaFile,
    current: &ast::TypeDefBlock,
    base: &ast::TypeDefBlock,
) -> anyhow::Result<()> {
    let mut visited = std::collections::HashSet::new();
    visited.insert(current.typename.as_ref().to_string());

    let mut current_base = base;
    loop {
        let base_name = current_base.typename.as_ref().to_string();

        if visited.contains(&base_name) {
            return Err(anyhow!(
                "Circular inheritance detected: {} forms a cycle",
                base_name
            ));
        }

        visited.insert(base_name.clone());

        // Check if this base has its own base
        if let Some(inheritance) = &current_base.inheritance {
            if let Some(next_base_name) = inheritance.base_type.ident_nonprim() {
                if let Some(next_base_typedef) = source.find_type(next_base_name) {
                    if let ast::TypeDef::Block(next_base_block) = next_base_typedef {
                        current_base = next_base_block;
                        continue;
                    }
                }
            }
        }

        // No more inheritance chain
        break;
    }

    Ok(())
}

pub fn compile_block_definition(
    source: &SourcedSchemaFile,
    blockdef: &ast::TypeDefBlock,
    schema: &mut Schema,
) -> anyhow::Result<Ref<model::Group>> {
    info!(
        "compiling Block definition '{}'...",
        blockdef.typename.to_string()
    );

    let attrs = compile_attributes(source, &blockdef.attributes, schema)?;

    if !attrs.is_empty() {
        info!("attributes: {:#?}", attrs.keys().collect_vec());
    }

    // Check for abstract modifier
    let is_abstract = blockdef.block.is_abstract();

    // Process inheritance
    let base_type = if let Some(inheritance) = &blockdef.inheritance {
        Some(compile_inheritance(source, blockdef, inheritance, schema)?)
    } else {
        None
    };

    compile_block(source, &blockdef.block, Some(attrs), is_abstract, base_type, schema)
}

pub fn compile_elements(
    source: &SourcedSchemaFile,
    schema: &mut model::Schema,
) -> anyhow::Result<()> {
    // iterate element definitions in the AST
    for element_ast in source.elements_top_level() {
        // now build the element
        let res = compile_element(source, element_ast, schema)?;
    }

    Ok(())
}

/// given a top-level element definition, compile it into a model Element
/// and return all supporting type definitions
pub fn compile_element(
    // complete source AST to query from
    source: &SourcedSchemaFile,
    // the ast element definition to compile to our models
    element_ast: &ast::Element,
    // the schema to register types in
    schema: &mut Schema,
) -> anyhow::Result<Ref<model::Element>> {
    info!("compiling element '{}'...", element_ast.name());

    if !element_ast.attributes.0.is_empty() {
        info!("attributes: {:#?}", &element_ast.attributes.0);
    }

    // create a builder for the element and
    let mut element_builder = model::ElementBuilder::default();

    // prepopulate with stuff that we can easily pull out
    element_builder
        .name(element_ast.name().to_string())
        // don tmerge attributes here already, since we can still merge and resolve from the model itself
        // .attributes(compile_attributes(source, element_ast, schema)?.unwrap())
        .attributes(compile_attributes(source, &element_ast.attributes, schema)?)
        .duplicity(element_ast.duplicity().map(Into::into).unwrap_or_default())
        .typing(match &element_ast.item {
            // element is defined as SimpleType or as type alias
            ElementItem::WithType(ast::ElementWithType { typing, .. }) => {
                compile_typing(source, typing, schema)?
            }
            // nested element definition
            ElementItem::WithBlock(ast::ElementWithBlock { block, .. }) => {
                compile_block(source, block, None, false, None, schema)?.into()
            }
        });

    schema.register_element(element_builder.build()?)
}

pub fn compile_typing_generic(
    source: &SourcedSchemaFile,
    element_ast: &ast::TypeWithGeneric,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    todo!("generics not impl yet")
}

pub fn compile_inline_type(
    source: &SourcedSchemaFile,
    element_ast: &ast::TypeDefInline,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    match &element_ast.typing {
        // Union type: Int | String | "literal"
        TypeDefInlineTyping::Union(union_ast) => {
            compile_type_union(source, union_ast, schema)
        }
        TypeDefInlineTyping::Typename(typename) => {
            compile_typename(source, typename, schema)
        },
        // the element is typed like an attribute
        TypeDefInlineTyping::SimpleType(inlinetype) => {
            parse_type_from_inline(source, inlinetype, schema)
        }

        // todo: a variable name is the identifier for a type.
        // to determine what the actual type is, we would have to
        // pass down all variables we encounter in the AST down to
        // the level where they are used, like here
        TypeDefInlineTyping::Var(var) => compile_typing_var(source, var, schema),
    }
}

// given a regular typename, resolve it to the final type definition,
// be it a block or a simple type. Any aliases will be resolved
pub fn compile_typing_regular(
    source: &SourcedSchemaFile,
    element_ast: &ast::TypeWithoutGeneric,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    match &element_ast.0 {
        // endpoint
        IdentType::Primitive(prim) => Ok(schema.register_primitive_type(prim.into())?.into()),

        // alias to other type
        IdentType::NonPrimitive(alias) => {
            let referred_typedef = source.find_type(&alias).ok_or(anyhow!(
                "Type definition not found for NonPrimitive '{}'",
                &alias
            ))?;

            // if type is already defined with this name, short-circuit and return known ref
            if let Some(existing) = schema.preliminary_ref_for_typename(referred_typedef, source) {
                return Ok(existing.get_ref());
            }

            compile_type_definition(source, schema, referred_typedef)
        }
    }
}

pub fn compile_typing(
    source: &SourcedSchemaFile,
    element_ast: &ast::Typing,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    match element_ast {
        // Union type: Int | String | "literal"
        Typing::Union(union) => compile_type_union(source, union, schema),
        Typing::Typename(typename) => compile_typename(source, typename, schema),
        // the contents of an
        Typing::Regex(regexty) => Ok(schema
            .register_simple_type(model::SimpleType::from_regex(regexty, schema))?
            .into()),
        // todo: a variable name is the identifier for a type.
        // to determine what the actual type is, we would have to
        // pass down all variables we encounter in the AST down to
        // the level where they are used, like here
        Typing::Var(var) => compile_typing_var(source, var, schema),
    }
}

/// Compile a typename with optional facets
pub fn compile_typename(
    source: &SourcedSchemaFile,
    typename: &ast::TypeName,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    // First compile the base type
    let base_type = match &typename.base {
        ast::TypeNameBase::Regular(regulartype) => compile_typing_regular(source, regulartype, schema)?,
        ast::TypeNameBase::Generic(generic_ty) => compile_typing_generic(source, generic_ty, schema)?,
    };

    // Apply facets if present
    if let Some(facets) = &typename.facets {
        // Get base primitive type to determine facet interpretation
        if let TypeRef::Simple(simple_ref) = &base_type {
            let simple_type = simple_ref.resolve(schema);
            let base_primitive = match simple_type {
                SimpleType::Builtin { name } => name.clone(),
                SimpleType::Derived { base, .. } => {
                    // Get the ultimate base primitive
                    let mut curr_base = base.resolve(schema);
                    loop {
                        match curr_base {
                            SimpleType::Builtin { name } => break name.clone(),
                            SimpleType::Derived { base, .. } => {
                                curr_base = base.resolve(schema);
                            }
                            _ => return Err(anyhow!("Facets can only be applied to primitive or derived types"))
                        }
                    }
                }
                _ => return Err(anyhow!("Facets can only be applied to simple types"))
            };

            let restrictions = compile_facets(facets, &base_primitive)?;

            // Create a derived type with the facets
            let faceted_type = SimpleType::Derived {
                base: simple_ref.clone(),
                restrictions,
                abstract_type: false,
            };

            Ok(schema.register_simple_type(faceted_type)?.into())
        } else {
            Err(anyhow!("Facets can only be applied to simple types, not complex types"))
        }
    } else {
        Ok(base_type)
    }
}

// todo: a variable name is the identifier for a type.
// to determine what the actual type is, we would have to
// pass down all variables we encounter in the AST down to
// the level where they are used, like here
pub fn compile_typing_var(
    source: &SourcedSchemaFile,
    element_ast: &ast::TypeVar,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    todo!("variable subtitution for type definitions")
}

/// Compile a union type (Type1 | Type2 | "literal" | 0)
pub fn compile_type_union(
    source: &SourcedSchemaFile,
    union_ast: &ast::TypeUnion,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    info!("compiling union type with {} members...", union_ast.members.len());

    let mut member_types = Vec::new();

    for member in &union_ast.members {
        let type_ref = match member {
            ast::UnionMember::TypeName(typename) => {
                // Compile typename with potential facets
                compile_typename(source, typename, schema)?
            }
            ast::UnionMember::Regex(regex) => {
                // Register regex as simple type
                schema.register_simple_type(SimpleType::from_regex(regex, schema))?.into()
            }
            ast::UnionMember::Literal(lit) => {
                // Register literal string as enumeration
                schema.register_simple_type(SimpleType::static_string(lit, schema))?.into()
            }
            ast::UnionMember::Var(_) => {
                return Err(anyhow!(
                    "Union members with type variables are not yet supported"
                ))
            }
            ast::UnionMember::Number(num) => {
                // Register numeric literal as enumeration
                schema.register_simple_type(SimpleType::static_number(num, schema))?.into()
            }
        };

        // Validate that the member is a simple type, not a complex type
        match type_ref {
            TypeRef::Simple(simple_ref) => {
                member_types.push(simple_ref);
            }
            TypeRef::Group(_) => {
                return Err(anyhow!(
                    "Union members must be simple types. Complex types (blocks) are not allowed in unions. \
                    This is an XSD limitation - unions can only contain simple types like Int, String, or type aliases to simple types."
                ))
            }
        }
    }

    // Register the union type
    Ok(schema.register_simple_type(SimpleType::Union { member_types })?.into())
}

pub fn resolve_block_def<'a>(
    ast: &'a ast::SchemaFile,
    typedef: &'a ast::TypeDef,
) -> Option<&'a ast::TypeDefBlock> {
    match typedef {
        TypeDef::Block(block) => Some(block),

        TypeDef::Inline(inlinedef) => {
            if inlinedef.is_generic() {
                todo!() // resolve
            }

            match &inlinedef.typing {
                ast::TypeDefInlineTyping::Union(_) => {
                    // unions are simple types, not block definitions
                    return None;
                }
                ast::TypeDefInlineTyping::Typename(ty) => match &ty.base {
                    ast::TypeNameBase::Regular(reg) => {
                        let name = reg.ident_nonprim().unwrap();
                        let typedef = ast.find_type(name).unwrap();
                        resolve_block_def(ast, typedef)
                    }
                    ast::TypeNameBase::Generic(_) => {
                        todo!("generics still unimpl")
                    }
                },
                ast::TypeDefInlineTyping::Var(_) => {
                    todo!("generics still unimpl")
                }
                // simpletypes are not block definitions
                ast::TypeDefInlineTyping::SimpleType(_) => return None,
            }
        }
    }
}

pub fn compile_block(
    source: &SourcedSchemaFile,
    block_ast: &ast::Block,
    attributes: Option<model::Attributes>,
    is_abstract: bool,
    base_type: Option<Ref<model::Group>>,
    schema: &mut Schema,
) -> anyhow::Result<Ref<model::Group>> {
    info!("compiling block definition...");

    // initialize a builder for the group
    // this definition goes inside the model::Type,
    // which is wrapped in a CompileResult
    let mut builder = GroupBuilder::default();

    // call builder setters
    builder
        .ty((&block_ast.mods))
        .mixed(block_ast.is_mixed_content())
        .abstract_type(is_abstract)
        .base_type(base_type)
        .attributes(attributes.unwrap_or_default())
        .items(
            block_ast
                .items
                .iter()
                .filter_map(|item| {
                    Some(match item {
                        BlockItem::Element(element_item) => {
                            compile_element(source, element_item, schema).map(Into::into)
                        }
                        BlockItem::SplatBlock(block) => {
                            compile_block(source, block.as_ref(), None, false, None, schema).map(Into::into)
                        }
                        BlockItem::SplatType(ast::SplatType(ty)) => ty
                            .ident_regular()
                            .ok_or(anyhow!(
                                "expected splatted type reference to not be generic!"
                            ))
                            .and_then(|res| {
                                source.find_type(res).ok_or(anyhow!(
                                    "type definition not found for IdentTypeNonPrimitive '{}'",
                                    &res
                                ))
                            })
                            .and_then(|res| {
                                resolve_block_def(source, res).ok_or(anyhow!(
                                    "expected resolved type definition to be a block definition"
                                ))
                            })
                            .and_then(|res| {
                                compile_block(source, &res.block, None, false, None, schema).map(Into::into)
                            }),
                        BlockItem::SplatGenericArg(_) => todo!("splat generic arg not impl yet"),
                        BlockItem::Comment(txt) => {
                            schema.push_comment(model::Comment::from(txt));
                            return None;
                        }
                    })
                })
                .collect::<anyhow::Result<_>>()?,
        );

    Ok(schema.register_group(builder.build()?)?)
}

// todo: throw out CompileResult struct

/// elements' attributes are defined both on element definitions and on Block Type definitions.
/// so, to know what the final element attributes are going to be, we need to know whether
/// the element Typing refers to a block definition with attributes
pub fn compile_element_attributes(
    source: &SourcedSchemaFile,
    element: &ast::Element,
    schema: &mut Schema,
) -> anyhow::Result<CompileResult<model::Attributes>> {
    // default to return when there is no Block Type definition
    let attrs = compile_attributes(source, &element.attributes, schema)?;

    // match on the actual element content type
    match &element.item {
        // element name is a definition with a reference to a defined Type, like
        //      #element: MyType
        //      #element: /this|that/
        ElementItem::WithType(typed) => match &typed.typing {
            // if the typing is a custom type reference
            Typing::Typename(ty) => {
                // try retrieve a non-primitive type identifier.
                // if it would ba a primitive, it wouldnt support any attribute definitions
                // and thus can be ignored
                if let Some(name) = ty.ident_nonprim() {
                    // lookup the type definition in the schema and retrieve attributes
                    let ast_attrs = &source
                        .find_type(name)
                        .ok_or(anyhow!(
                            "Type definition not found for IdentTypeNonPrimitive '{}'",
                            &name
                        ))?
                        .attributes();

                    // parse attributes and merge so that the element attributes override the nested type attributes
                    return Ok(compile_attributes(source, ast_attrs, schema)?
                        .merge(attrs)
                        .into());
                }
            }

            // we are doing top-level element parsing where no variables can be in the type
            Typing::Var(var) => {
                let typedef = compile_typing_var(source, var, schema)?;

                todo!("make sure found type is a SimpleType fit for attributes");

                // typedef.attributes() ...
            }

            // Union types and TypeRegex have no attributes (simple types)
            Typing::Union(_) | Typing::Regex(_) => {}
        },

        // alternatively the element could have an attached inline block
        // which doesnt allow inline attributes so we can ignore the case
        ElementItem::WithBlock(_) => {}
    }

    Ok(attrs.into())
}

/// compile AST attributes into model Attributes
pub fn compile_attributes(
    source: &SourcedSchemaFile,
    attrs: &ast::Attributes,
    schema: &mut Schema,
) -> anyhow::Result<model::Attributes> {
    Ok(model::Attributes::new(
        attrs
            .iter()
            .map(|attr| parse_attribute(source, attr, schema))
            .collect::<anyhow::Result<_>>()?,
        schema,
    ))
}

pub fn parse_attribute_type_from_primitive_or_alias(
    source: &SourcedSchemaFile,
    typing: &TypeWithoutGeneric,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    match &typing.0 {
        // coerce primtive type defininition into SimpleType
        IdentType::Primitive(prim) => Ok(schema
            .register_primitive_type(model::PrimitiveType::from(prim))?
            .into()),
        // type is alias and refers to definition elsewhere
        IdentType::NonPrimitive(alias) => {
            let referenced_typedef = source.find_type(alias).ok_or(anyhow!(
                "type definition not found in AST for Attribute type: '{}'",
                alias
            ))?;

            match referenced_typedef {
                TypeDef::Inline(inlinedef) => {
                    if inlinedef.is_generic() {
                        panic!("generic attribute type definitions not supported yet");
                    }

                    match &inlinedef.typing {
                        TypeDefInlineTyping::Union(union_ast) => {
                            compile_type_union(source, union_ast, schema)
                        }
                        TypeDefInlineTyping::Typename(typename) => {
                            compile_typename(source, typename, schema)
                        },
                        TypeDefInlineTyping::SimpleType(simpletype) => {
                            parse_type_from_inline(source, simpletype, schema)
                        }
                        TypeDefInlineTyping::Var(var) => {
                            Ok(compile_typing_var(source, var, schema)?)
                        }
                    }
                }
                // for now it is an error to have attribute types reference a block definition
                // but in the future we may use a block definition to have more space for complex attr type definitions
                TypeDef::Block(_) => Err(anyhow!(
                    // todo: reference position in source code
                    "block type definitions for attributes not allowed (in {:?})",
                    typing
                ))?,
            }
        }
    }
}

pub fn parse_type_from_inline(
    source: &SourcedSchemaFile,
    typing: &SimpleTypingInline,
    schema: &mut Schema,
) -> anyhow::Result<model::TypeRef> {
    // todo: support inline Typing like rust traits by considering the whole array.
    // it would look like: String + "--" + Int + /this|that/
    if typing.is_compound() {
        todo!("compound attribute definition items not supported yet");
    }
    // we dont support generics yet
    else if typing.is_generic() {
        todo!("generic attribute definition items not supported yet")
    }
    // its a single type that we can resolve. Could be a primitive, alias or reference to custom type
    else {
        match typing.first_item() {
            // type definition reference
            AttrItem::Simple(typename) => {
                compile_typename(source, typename, schema)
            }
            // regex definition
            AttrItem::TypeRegex(regexdef) => Ok(schema
                .register_simple_type(SimpleType::from_regex(regexdef, schema))?
                .into()),
            // static string definition
            AttrItem::AttrItemStr(strval) => Ok(schema
                .register_simple_type(SimpleType::static_string(strval, schema))?
                .into()),
            _ => unreachable!("typename should not be generic"),
        }
    }
}

/// compile AST attributes into model Attributes
pub fn parse_attribute(
    source: &SourcedSchemaFile,
    attr: &ast::AttrDef,
    schema: &mut Schema,
) -> anyhow::Result<Ref<model::Attribute>> {
    let mut builder = model::AttributeBuilder::default();

    builder
        .name(attr.assign.ident.as_ref().to_string())
        .required(attr.is_required())
        .typing(match &attr.typing {
            None => schema.register_simple_type(default())?, // String by default
            Some(typing) => {
                let type_ref = match typing {
                    ast::AttrTyping::Union(union) => compile_type_union(source, union, schema)?,
                    ast::AttrTyping::SimpleCompound(simple) => parse_type_from_inline(source, simple, schema)?,
                };
                match type_ref {
                    TypeRef::Simple(simpletype) => simpletype,
                    TypeRef::Group(_) => Err(anyhow!("group Type not supported for Attribute"))?,
                }
            },
        });

    schema.register_attribute(builder.build()?)
}

/// independent types are Type definitions that do not need further resolving in the AST
pub fn get_independent_types(source: &SourcedSchemaFile) -> Vec<&ast::TypeDef> {
    source
        .types()
        .into_iter()
        .filter(|ty| is_independent_type_def(ty))
        .collect::<Vec<_>>()
}

/// whether a Type is not a reference/alias to something else
pub fn is_independent_type_def(def: &ast::TypeDef) -> bool {
    match def {
        ast::TypeDef::Inline(ty_inline) => {
            !ty_inline.is_generic()
                && match &ty_inline.typing {
                    TypeDefInlineTyping::Union(_) => true, // unions are independent
                    TypeDefInlineTyping::Typename(name) => is_independent_type_name(name),
                    TypeDefInlineTyping::SimpleType(simple) => simple.is_independent_type(),
                    TypeDefInlineTyping::Var(_) => false,
                }
        }
        ast::TypeDef::Block(block) => is_independent_type(block),
    }
}

pub fn is_independent_type(ty: &ast::TypeDefBlock) -> bool {
    !ty.is_generic() && is_independent_block(&ty.block)
}

pub fn is_independent_type_name(ty: &ast::TypeName) -> bool {
    match &ty.base {
        ast::TypeNameBase::Regular(ast::TypeWithoutGeneric(IdentType::Primitive(_))) => true,
        // generics
        _ => false,
    }
}

pub fn is_independent_block(ty: &ast::Block) -> bool {
    ty.items.iter().map(is_independent_block_item).all(identity)
}

pub fn is_independent_block_item(item: &ast::BlockItem) -> bool {
    match item {
        BlockItem::Element(element) => {
            match &element.item {
                ElementItem::WithBlock(ast::ElementWithBlock { block, .. }) => {
                    is_independent_block(block)
                }
                ElementItem::WithType(ast::ElementWithType { typing, .. }) => {
                    match typing {
                        Typing::Typename(typename) => match &typename.base {
                            ast::TypeNameBase::Regular(ast::TypeWithoutGeneric(
                                IdentType::Primitive(_),
                            )) => true,
                            _ => false,
                        },
                        Typing::Regex(_) => true,
                        // if theres generics
                        _ => false,
                    }
                }
            }
        }
        BlockItem::SplatBlock(ast::SplatBlock(block)) => is_independent_block(block),
        BlockItem::SplatType(ast::SplatType(typename)) => match &typename.base {
            ast::TypeNameBase::Regular(ast::TypeWithoutGeneric(IdentType::Primitive(_))) => true,
            _ => false,
        },
        BlockItem::SplatGenericArg(_) => false,
        BlockItem::Comment(_) => true,
        // any of the specific branches that were unmatched
        _ => false,
    }
}

/// Compile facets from AST into SimpleTypeRestriction
pub fn compile_facets(
    facets: &ast::Facets,
    base_primitive: &model::PrimitiveType,
) -> anyhow::Result<model::restriction::SimpleTypeRestriction> {
    use model::restriction::SimpleTypeRestriction;

    let mut restriction = SimpleTypeRestriction::default();

    if let Some(facet_list) = &facets.items {
        for facet_item in &facet_list.items {
            match facet_item {
                ast::FacetItem::Shorthand(shorthand) => {
                    let range = shorthand.parse_range();

                    // Apply shorthand based on base type
                    match base_primitive {
                        // String types: shorthand = length constraints
                        model::PrimitiveType::String => {
                            if let Some(min) = &range.min {
                                if let Some(max) = &range.max {
                                    if min == max {
                                        // Exact length
                                        restriction.length = Some(min.parse()?);
                                    } else {
                                        // Range
                                        restriction.min_length = Some(min.parse()?);
                                        restriction.max_length = Some(max.parse()?);
                                    }
                                } else {
                                    // Min only
                                    restriction.min_length = Some(min.parse()?);
                                }
                            } else if let Some(max) = &range.max {
                                // Max only
                                restriction.max_length = Some(max.parse()?);
                            }
                        }
                        // Numeric types: shorthand = value range constraints
                        model::PrimitiveType::Int
                        | model::PrimitiveType::Short
                        | model::PrimitiveType::Float
                        | model::PrimitiveType::Double
                        | model::PrimitiveType::Decimal => {
                            if let Some(min) = &range.min {
                                restriction.min_inclusive = Some(min.to_string());
                            }
                            if let Some(max) = &range.max {
                                restriction.max_inclusive = Some(max.to_string());
                            }
                        }
                        _ => {
                            return Err(anyhow!(
                                "Shorthand range syntax not supported for type {:?}",
                                base_primitive
                            ))
                        }
                    }
                }
                ast::FacetItem::Named(named) => {
                    let name = named.name.as_str();
                    let value = named.value.as_string();

                    match name {
                        // Length facets
                        "length" => restriction.length = Some(value.parse()?),
                        "minLength" => restriction.min_length = Some(value.parse()?),
                        "maxLength" => restriction.max_length = Some(value.parse()?),

                        // Numeric range facets
                        "minInclusive" => restriction.min_inclusive = Some(value),
                        "maxInclusive" => restriction.max_inclusive = Some(value),
                        "minExclusive" => restriction.min_exclusive = Some(value),
                        "maxExclusive" => restriction.max_exclusive = Some(value),

                        // Precision facets
                        "totalDigits" => restriction.total_digits = Some(value.parse()?),
                        "fractionDigits" => restriction.fraction_digits = Some(value.parse()?),

                        // Whitespace facet
                        "whiteSpace" => {
                            restriction.white_space = Some(match value.as_str() {
                                "preserve" => model::restriction::WhiteSpaceHandling::Preserve,
                                "replace" => model::restriction::WhiteSpaceHandling::Replace,
                                "collapse" => model::restriction::WhiteSpaceHandling::Collapse,
                                _ => return Err(anyhow!(
                                    "Invalid whiteSpace value: '{}'. Must be 'preserve', 'replace', or 'collapse'",
                                    value
                                )),
                            });
                        }

                        // Pattern facet (from regex value)
                        "pattern" => restriction.pattern = Some(value),

                        _ => {
                            return Err(anyhow!("Unknown facet name: '{}'", name));
                        }
                    }
                }
            }
        }
    }

    Ok(restriction)
}
