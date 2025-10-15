use crate::model::restriction::SimpleTypeRestriction;
use crate::model::{GroupItem, SimpleType};
use crate::model::{PrimitiveType, SchemaObjId};
use crate::sourced::SourcedSchemaFile;
use crate::tests::get_test_schema_ast;
use crate::tools::init_logger;
use crate::Rule::schema;
use crate::{ast, compiler, model};
use itertools::Itertools;
use log::debug;
use std::path::Path;
use std::{env, path};
use strum::IntoEnumIterator;
use tap::Tap;

/// independent types are Type definitions that do not need further resolving in the AST
#[test]
fn get_independent_types() {
    let sch = get_test_schema_ast();
    let ty = compiler::get_independent_types(&sch);

    let ty_idents = ty
        .iter()
        .map(|t| t.ident_nonprim().to_string())
        .sorted()
        .collect::<Vec<_>>();

    assert_eq!(
        ty_idents.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        vec!(
            "AssignedPersonIds",
            "Person",
            "Price",
            "SectionHeader",
            "TimeUnit",
        )
        .into_iter()
        .sorted()
        .collect::<Vec<_>>()
    );
}

#[test]
fn test_types_alphabet_sortable() {
    let sch = &get_test_schema_ast();
    let ty = compiler::get_independent_types(sch)
        .into_iter()
        .sorted()
        .collect_vec();

    let typenames = ty
        .iter()
        .map(|t| t.ident_nonprim().to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        typenames,
        vec!(
            "AssignedPersonIds",
            "Person",
            "Price",
            "SectionHeader",
            "TimeUnit",
        )
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
    );
}

#[test]
fn test_primitive_aliasing() {
    let sch = model::Schema::from_file("src/tests/schemas/aliasing.whas").unwrap();
}

#[test]
fn test_simpletypes() {
    let sch = ast::SchemaFile::parse(
        r#"
        #simple-type: SimpleType
        
        SimpleType: WrapperType
        
        // String has no type definition because it is builtin
        WrapperType: String
        
        // has no type definition because it is builtin
        #pos-int: +Int
        
        // has no type definition because it is builtins
        #neg-int: -Int
    "#,
    )
    .unwrap();
    let attr_types = sch.types_simple().expect("type resolve issue");

    dbg!(&sch.types_simple());

    assert_eq!(2, attr_types.len());

    let sch = &get_test_schema_ast();

    dbg!("{}", sch);

    let attr_types = sch.types_simple().expect("type resolve issue");
    assert_eq!(2, attr_types.len());
}

#[test]
fn test_block_attributes() {
    let sch = ast::SchemaFile::parse(
        r#"
        @title?: String // optional title for in Outline when there is no title element or sth in the doc
        HierarchyLeafNode {
            // empty
        }
    "#,
    )
        .unwrap();

    let group = &sch.types_group()[0];

    assert!(group.attributes().0[0].assign.mod_opt.is_some());

    dbg!(&group);
}

#[test]
fn test_schema_object_id_gen() {
    let first = SchemaObjId::new().value();
    let second = SchemaObjId::new().value();
    let third = SchemaObjId::new().value();

    assert!(second > first);
    assert!(third > second);
}

#[test]
fn test_schema_type_preloading() {
    let sch = model::Schema::default();

    assert_eq!(
        sch.all_type_names()
            .into_iter()
            .cloned()
            .sorted()
            .collect::<Vec<_>>(),
        PrimitiveType::iter()
            .map(|t| t.to_string())
            .sorted()
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_compile_element() -> anyhow::Result<()> {
    let sch = ast::SchemaFile::parse(
        r#"
        #element: String
    "#,
    )?;

    println!("{:#?}", &sch);

    let res = compiler::compile(&sch.into())?;

    println!("{:#?}", &res);

    let els = res.get_elements_by_name("element");

    assert_eq!(1, els.len());
    assert_eq!(
        els[0].typing().simpletype(&res),
        Some(SimpleType::from(PrimitiveType::String)).as_ref()
    );

    Ok(())
}

#[test]
fn test_compile_primitives() -> anyhow::Result<()> {
    // dbg!(env::current_dir());
    // dbg!(file!());
    // let path = Path::new(&file!())
    //     .parent()
    //     .unwrap()
    //     .join("schemas/primitives.whas")
    //     .canonicalize()?;

    // let abspath = path::absolute(path)?;

    let sch = model::Schema::from_file("src/tests/schemas/primitives.whas")?;

    dbg!(&sch.types_simple());

    assert_eq!(0, sch.types_group().len());

    for var in model::PrimitiveType::iter() {
        let ty = sch.get_simpletype_by_name(var.to_string().as_str());
        assert!(
            sch.get_simpletype_by_name(var.to_string().as_str())
                .is_some(),
            "primitive type {} not found",
            var
        );
    }

    Ok(())
}

#[test]
fn test_recursion_2() -> anyhow::Result<()> {
    let ast = ast::SchemaFile::parse(
        r#"
List {
    #item+: ListItem
}

// a list item is a mixed content definition
// that allows optional sublists
ListItem x{
    // ... mixed content here
    #list?: List
}
        "#,
    )
    .unwrap();

    let mut sch = model::Schema::default();
    compiler::compile_type_definitions(&ast.into(), &mut sch).unwrap();

    sch.assert_type_name("List")?
        .assert_type_name("ListItem")?
        .assert_element_name("list")?
        .assert_element_name("item")?;

    Ok(())
}

#[test]
fn test_recursion_3() -> anyhow::Result<()> {
    init_logger();

    let ast = ast::SchemaFile::parse(
        r#"
// type definition for textual contents that can span
// multiple paragraphs and supports lists
Text x{
    // ... mixed content
    #list1*: List
}


List {
    #item+: ListItem
}


// a list item is a mixed content definition
// that allows optional sublists
ListItem x{
    // ... mixed content here
    #list2?: List
}
        "#,
    )
    .unwrap();

    let mut sch = model::Schema::default();
    compiler::compile_type_definitions(&ast.into(), &mut sch).unwrap();

    sch.assert_type_name("Text")?
        .assert_type_name("List")?
        .assert_type_name("ListItem")?
        .assert_element_name("list1")?
        .assert_element_name("list2")?
        .assert_element_name("item")?;

    Ok(())
}

#[test]
fn test_block_type_splat() -> anyhow::Result<()> {
    let sch = ast::SchemaFile::parse(
        r#"
        #element: {
            ...Def
        }
        
        Def {
            #block-contents: String
        }
    "#,
    )?;

    println!("{:#?}", &sch);

    let res = compiler::compile(&sch.into())?;

    println!("{:#?}", &res);

    let els = res.get_elements_by_name("element");

    // single element should be defined
    assert_eq!(1, els.len());

    let gr = els[0].typing().grouptype(&res).unwrap();
    assert_eq!(
        gr.items().len(),
        1,
        "should have one element defined called #block-contents"
    );

    match gr.items().first().unwrap() {
        GroupItem::Element(el) => {
            assert_eq!(el.resolve(&res).name(), "#block-contents")
        }
        GroupItem::Group(_) => {}
    }

    Ok(())
}

/// Block definitions may define attributes,
/// but when an element is defined using a Block expression
/// the element attributes should override the ones from the Block
#[test]
fn test_attr_merging() -> anyhow::Result<()> {
    // parse a schema with an element definition
    let ast = ast::SchemaFile::parse(
        r#"
        @attr-a: String
        @attr-b: Int
        #element: Block
        
        @attr-b: Bool
        @attr-c: String
        Block {}
    "#,
    )?;

    // println!("{:#?}", &ast);

    let sourced_schema: SourcedSchemaFile = ast.into();

    // get only element definition that contains the attributes that should override the ones from the block definition
    let element = sourced_schema.elements_top_level()[0];

    // create schema instance so we can compile elements into it
    let mut target = model::Schema::default();

    // compile the attributes for the given element.
    // this will resolve the Block and merge the attributes found on that definition
    let attrs_obj = compiler::compile_element_attributes(&sourced_schema, element, &mut target)?;
    let mut attrs = attrs_obj.target.get(&target);

    attrs.sort_by_key(|a| a.name());

    dbg!(&attrs);

    assert_eq!(3, attrs.len());

    assert_eq!("attr-a", attrs[0].name());
    assert_eq!("attr-b", attrs[1].name());
    assert_eq!("attr-c", attrs[2].name());

    assert_eq!(
        attrs[0].typing.resolve(&target).to_type_name(&target),
        "String"
    );
    assert_eq!(
        attrs[1].typing.resolve(&target).to_type_name(&target),
        "Int"
    );
    assert_eq!(
        attrs[2].typing.resolve(&target).to_type_name(&target),
        "String"
    );

    Ok(())
}

#[test]
fn compile_test_schema_price() -> anyhow::Result<()> {
    let source = get_test_schema_ast();
    let mut sch = model::Schema::default();

    compiler::compile_type_definition(
        &source,
        &mut sch,
        source.find_type_by_name("Price").unwrap(),
    )?;

    sch.assert_type_name("Price")?
        .assert_element_name("amount")?
        .assert_element_name("desc")?
        .assert_element_name("type")?;

    Ok(())
}

#[test]
fn compile_test_schema_person() -> anyhow::Result<()> {
    let source = get_test_schema_ast();
    let mut sch = model::Schema::default();

    compiler::compile_type_definition(
        &source,
        &mut sch,
        source.find_type_by_name("Person").unwrap(),
    )?;

    sch.assert_type_name("Person")?
        .assert_element_name("modifier")?;

    Ok(())
}

#[test]
fn compile_test_schema_text() -> anyhow::Result<()> {
    init_logger();

    let source = get_test_schema_ast();
    let mut sch = model::Schema::default();

    compiler::compile_type_definition(
        &source,
        &mut sch,
        source.find_type_by_name("Text").unwrap(),
    )?;

    sch.assert_type_name("Text")?
        .assert_type_name("List")?
        .assert_type_name("ListItem")?
        .assert_element_name("list")?
        .assert_element_name("item")?;

    Ok(())
}

#[test]
fn compile_test_schema_milestone() -> anyhow::Result<()> {
    init_logger();

    let source = get_test_schema_ast();
    let mut sch = model::Schema::default();

    compiler::compile_type_definition(
        &source,
        &mut sch,
        source.find_type_by_name("Milestone").unwrap(),
    )?;

    sch.assert_type_name("Milestone")?
        .assert_type_name("List")?
        .assert_type_name("ListItem")?
        .assert_element_name("title")?
        .assert_element_name("taskdescription")?
        .assert_element_name("assumptions")?
        .assert_element_name("userstory")?;

    Ok(())
}

#[test]
fn compile_test_schema_tasklist() -> anyhow::Result<()> {
    init_logger();

    let source = get_test_schema_ast();
    let mut sch = model::Schema::default();

    compiler::compile_type_definition(
        &source,
        &mut sch,
        source.find_type_by_name("TaskList").unwrap(),
    )?;

    sch.assert_type_name("TaskList")?
        .assert_type_name("Task")?
        .assert_element_name("task")?
        .assert_element_name("description")?
        .assert_element_name("subtasks")?
        .assert_element_name("ticket")?;

    Ok(())
}

#[test]
fn compile_test_schema_estimate() -> anyhow::Result<()> {
    init_logger();

    let source = get_test_schema_ast();
    let mut sch = model::Schema::default();

    compiler::compile_type_definition(
        &source,
        &mut sch,
        source.find_type_by_name("Estimate").unwrap(),
    )?;

    sch.assert_type_name("Estimate")?
        .assert_type_name("TimeUnit")?
        .assert_element_name("type")?
        .assert_element_name("amount")?;

    let tu = sch
        .get_simpletype_by_name("TimeUnit")
        .expect("TimeUnit not found");

    assert!(tu.is_derived());

    assert_eq!(
        tu.restrictions().unwrap(),
        &SimpleTypeRestriction::default().tap_mut(|r| {
            r.pattern = Some("days|hours|person days".to_owned());
        }),
        "make sure the final regex string does not include the regex delimiters '/'"
    );

    Ok(())
}

#[test]
fn compile_test_schema() -> anyhow::Result<()> {
    let source = get_test_schema_ast();
    let mut sch = model::Schema::default();

    let res = compiler::compile(&source)?;

    println!("{:#?}", &res);

    Ok(())
}

// when testing the binary, it turned out repeated invocations would case different states
#[test]
fn compile_test_schema_determinism_5x() -> anyhow::Result<()> {
    // rin the same compilation run a few times ina row to find out
    // if one of the invocations would fail, indicating indeterminism
    for _ in 0..5 {
        let source = get_test_schema_ast();
        let mut sch = model::Schema::default();

        let res = compiler::compile(&source)?;
    }

    // println!("{:#?}", &res);

    Ok(())
}
