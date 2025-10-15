use crate::{model, Rule, WHASParser};
use std::fmt::Debug;

use crate::ast::*;
use from_pest::FromPest;
use pest::Parser;
use tap::Tap;

/// helper
fn assert_ast<'a, AST: Debug + FromPest<'a, Rule = Rule, FatalError: Debug>>(
    rule: Rule,
    input: &'a str,
) -> AST {
    // Parse your input string using the Pest parser
    let mut parsed = WHASParser::parse(rule, input).unwrap();

    println!("parsed rules data: {:#?}", &parsed);

    // Convert the Pest AST to your Rust structs
    let ident = AST::from_pest(&mut parsed);

    assert!(
        ident.is_ok(),
        "[{:#?}] failed to parse AST for input '{}': {:#?}",
        rule,
        input,
        &ident
    );

    ident
        .unwrap()
        .tap(|res| println!("[{:?}] res: {:#?}", rule, res))
}

#[test]
fn test_comments() {
    assert_ast::<CommentLine>(Rule::comment_line, "// this is a comment\n");
    assert_ast::<Comment>(Rule::comment, "// this is a comment\n");
    assert_ast::<CommentMarkdown>(
        Rule::comment_md,
        r#"```
        this is a **Markdown comment**
    ```"#,
    );
    assert_ast::<CommentMarkdown>(Rule::comment_md, r#"```$```inner markdown$``````"#);
    assert_ast::<CommentMarkdown>(
        Rule::comment_md,
        r#"```
        markdown comments are done like this:

        $```
            this is a **Markdown comment**
        $```
    ```"#,
    );

    assert_ast::<Comment>(
        Rule::comment,
        "```
        markdown comments are done like this:

        $```
            this is a **Markdown comment**
        $```
    ```",
    );

    assert_ast::<Comment>(Rule::comment, "/* inline wild comment */");

    assert_ast::<Comment>(
        Rule::comment,
        r#"/*  
            block wild comment
        */"#,
    );
}

#[test]
fn test_idents() {
    assert_ast::<IdentLowercase>(Rule::ident_lowercase, "attr-ident");
    assert_ast::<IdentTypeNonPrimitive>(Rule::ident_type_nonprimitive, "StringIntTimeType");
    assert_ast::<IdentTypeNonPrimitive>(Rule::ident_type_nonprimitive, "Type");
    assert_ast::<IdentType>(Rule::ident_type, "Type");
    assert_ast::<IdentAttr>(Rule::ident_attr, "@attr-ident");
    assert_ast::<IdentElement>(Rule::ident_element, "#elem-ident");
    assert_ast::<Ident>(Rule::ident, "#elem-ident");
}

#[test]
fn test_symbols() {
    assert_ast::<SymbolModOpt>(Rule::sym_mod_opt, "?");
    assert_ast::<ModRange>(Rule::mod_range, "[0..5]");
    assert_ast::<ModRange>(Rule::mod_range, "[5]");
}

#[test]
fn test_primitives() {
    assert_ast::<Primitive>(Rule::primitive, "String");
    assert_ast::<Primitive>(Rule::primitive, "+Int");
    assert_ast::<NonPrimitive>(Rule::ident_type_nonprimitive, "NonString");
}

#[test]
fn test_attributes() {
    assert_ast::<AttrAssign>(Rule::attr_assign, "@attr?");
    assert_ast::<AttrAssign>(Rule::attr_assign, "@attr");

    assert_ast::<SimpleTypingInline>(Rule::simple_compound_inline, "String+Int");

    assert_ast::<AttrDef>(Rule::attrdef, "@attr?: AssignedPersonIds");
    assert_ast::<AttrDef>(Rule::attrdef, "// comment\n@attr?: AssignedPersonIds");
    assert_ast::<AttrDef>(Rule::attrdef, "@attr: String");
    assert_ast::<AttrDef>(Rule::attrdef, "@attr");

    assert_ast::<AttrDef>(Rule::attrdef, "@attr // attribute");
    assert_ast::<AttrDef>(Rule::attrdef, "@attr?: AssignedPersonIds // attribute");

    assert_eq!(
        2,
        assert_ast::<Attributes>(Rule::attributes, "@attr?: String\n@attr2")
            .0
            .len()
    );
}

#[test]
fn test_imports() {
    assert_ast::<ImportPath>(Rule::import_path, "'./filepath.whas'");
    assert_ast::<ImportPath>(Rule::import_path, "\"also::a::filepath\"");

    assert_ast::<ImportSelector>(Rule::import_selector, "*");
    assert_ast::<ImportSelector>(Rule::import_selector, "{}");
    assert_ast::<ImportSelector>(Rule::import_selector, "{Definition}");
    assert_ast::<ImportSelector>(Rule::import_selector, "{Definition1,Definition2}");
    assert_ast::<ImportSelector>(Rule::import_selector, "{TypeWithGeneric}");

    assert_ast::<ImportInline>(Rule::import_inline, "import './filepath.whas'");
    assert_ast::<ImportInline>(Rule::import_inline, "import * from './filepath.whas'");
    assert_ast::<ImportInline>(Rule::import_inline, "import {} from './filepath.whas'");
    assert_ast::<ImportInline>(Rule::import_inline, "import {Type} from './filepath.whas'");
    assert_ast::<ImportInline>(
        Rule::import_inline,
        "import {Type1, Type2} from './filepath.whas'",
    );

    assert_ast::<ImportExtended>(Rule::import_extended, "import './filepath.whas' {}");
    assert_ast::<ImportExtended>(Rule::import_extended, "import from './filepath.whas' {}");
    assert_ast::<ImportExtended>(
        Rule::import_extended,
        "import from './filepath.whas' {Type, Type2}",
    );
    assert_ast::<ImportExtended>(Rule::import_extended, "import from './filepath.whas' *");

    assert_ast::<Import>(Rule::import, "import {Type} from './filepath.whas'");
    assert_ast::<Import>(Rule::import, "import from './filepath.whas' {Type, Type2}");

    // todo
    // assert_ast::<ImportExtended>(
    //     Rule::import_extended,
    //     "import from './filepath.whas' {\
    //     \
    //     // comment \
    //     Type, \
    //     \
    //     \
    //     Type2\
    // }",
    // );

    // todo
    // assert_ast::<ImportSelector>(Rule::import_selector, "{/*with comment*/ TypeWithGeneric}");
}

#[test]
fn test_typevars() {
    assert_ast::<TypeDefVars>(Rule::typedef_vars, "()");
    assert_ast::<TypeDefVars>(Rule::typedef_vars, "(arg1)");
    assert_ast::<TypeDefVars>(Rule::typedef_vars, "(arg1, arg2)");
}

#[test]
fn test_typeargs() {
    assert_ast::<TypeArgs>(Rule::type_args, "()");
    assert_ast::<TypeArgs>(Rule::type_args, "(Arg1)");
    assert_ast::<TypeArgs>(Rule::type_args, "(Arg1, Arg2)");
    assert_ast::<TypeArgs>(Rule::type_args, "(Arg1(String), Arg2(X(Item)))");
}

#[test]
fn test_typedef_inline() {
    assert_ast::<TypeDefInlineTyping>(Rule::typedef_inline_typing, "String");
    assert_ast::<TypeName>(Rule::typename, "String");

    assert_ast::<TypeDefInline>(Rule::typedef_inline, "Type: String");
    assert_ast::<TypeDefInline>(Rule::typedef_inline, "Type(): String");
    assert_ast::<TypeDefInline>(Rule::typedef_inline, "Type(arg1): String");
    assert_ast::<TypeDefInline>(Rule::typedef_inline, "Type(arg1, arg2): String");
    assert_ast::<TypeDefInline>(Rule::typedef_inline, "Type(arg): arg");
}

#[test]
fn test_typing() {
    assert_ast::<Typing>(Rule::typing, "String");
    assert_ast::<Typing>(Rule::typing, "List(Item)");
    assert_ast::<Typing>(Rule::typing, "var");
}

#[test]
fn test_block() {
    // modifiers
    assert_ast::<Block>(Rule::block, "{}");
    assert_ast::<Block>(Rule::block, "?{}");
    assert_ast::<Block>(Rule::block, "x?{}");
    assert_ast::<Block>(Rule::block, "!{}");
    assert_ast::<Block>(Rule::block, "x!x{}");

    assert_ast::<Block>(
        Rule::block,
        "{\
        #element: String\
    }",
    );

    assert_ast::<Block>(
        Rule::block,
        "{\
        ...{}\
    }",
    );

    assert_ast::<Block>(
        Rule::block,
        "{\
        ...{\
            #element: String\
        }\
    }",
    );

    assert_ast::<Block>(
        Rule::block,
        "{\
        ...BlockType
    }",
    );

    assert_ast::<Block>(
        Rule::block,
        "{\
        ...var
    }",
    );
}

#[test]
fn test_typedef_block() {
    // modifiers
    assert_ast::<TypeDefBlock>(Rule::typedef_block, "Type: {}");
    assert_ast::<TypeDefBlock>(Rule::typedef_block, "Type: x{}");
    assert_ast::<TypeDefBlock>(Rule::typedef_block, "Type: x?{}");
    assert_ast::<TypeDefBlock>(Rule::typedef_block, "Type: x!{}");
    assert_ast::<TypeDefBlock>(Rule::typedef_block, "Type: x!x{}");
    assert_ast::<TypeDefBlock>(Rule::typedef_block, "Type(): x!x{}");
    assert_ast::<TypeDefBlock>(Rule::typedef_block, "Type(var1): x!x{}");
    assert_ast::<TypeDefBlock>(Rule::typedef_block, "Type(var1, var2): x!x{}");
    assert_ast::<TypeDefBlock>(
        Rule::typedef_block,
        "Type(var1, var2): x!x{\
        #element: String\
    }",
    );
    assert_ast::<TypeDefBlock>(
        Rule::typedef_block,
        "Type(var1, var2): x!x{\
        #element: var1\
    }",
    );
    assert_ast::<TypeDefBlock>(
        Rule::typedef_block,
        "Type(var1, var2): x!x{\
        ...var2
    }",
    );
}

#[test]
fn test_element() {
    // as element with type
    assert_ast::<ElementWithType>(Rule::element_with_type, "#element: String");
    assert_ast::<ElementWithType>(Rule::element_with_type, "#element: List(Item)");
    assert_ast::<ElementWithType>(Rule::element_with_type, "#element: var");
    assert_ast::<ElementWithType>(Rule::element_with_type, "#element: /(this|that)/");

    // with : delimiter
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element: {}");
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element: x{}");
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element: ?{}");
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element: !{}");
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element: x?{}");

    // withut delimiter
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element {}");
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element x{}");
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element ?{}");
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element !{}");
    assert_ast::<ElementWithBlock>(Rule::element_with_block, "#element x?{}");

    // as item
    assert_ast::<ElementItem>(Rule::element_item, "#element: String");
    assert_ast::<ElementItem>(Rule::element_item, "#element: List(Item)");
    assert_ast::<ElementItem>(Rule::element_item, "#element: var");
    assert_ast::<ElementItem>(Rule::element_item, "#element: /(this|that)/");

    // as element
    assert_ast::<Element>(Rule::element, "#element: String");
    assert_ast::<Element>(Rule::element, "#element: List(Item)");
    assert_ast::<Element>(Rule::element, "#element: var");
    assert_ast::<Element>(Rule::element, "#element: /(this|that)/");

    // with attributes
    assert_ast::<Element>(Rule::element, "@attr: String\n@attr2\n#element: String");
    assert_ast::<Element>(Rule::element, "@attr: String\n@attr2\n#element: List(Item)");
    assert_ast::<Element>(Rule::element, "@attr: String\n@attr2\n#element: var");
    assert_ast::<Element>(
        Rule::element,
        "@attr: String\n@attr2\n#element: /(this|that)/",
    );
}

#[test]
fn test_schema_item() {
    assert_ast::<SchemaItem>(Rule::schema_item, "#element: String");
    assert_ast::<SchemaItem>(Rule::schema_item, "//comment");
    assert_ast::<SchemaItem>(
        Rule::schema_item,
        "@attr: String\n@attr2\n#element: /(this|that)/",
    );
    assert_ast::<SchemaItem>(
        Rule::schema_item,
        "Type(var1, var2): x!x{\
        ...var2
    }",
    );
}

// #[test]
// fn test_dbg() {
//     assert_ast::<SchemaDbg>(Rule::schema_dbg, "// comment\n#element: String");
// }

#[test]
fn test_schema() {
    // empty
    assert_ast::<SchemaFile>(Rule::schema, "");

    // with space
    assert_ast::<SchemaFile>(Rule::schema, " ");

    // with tab
    assert_ast::<SchemaFile>(Rule::schema, "\t");

    // with newline
    assert_ast::<SchemaFile>(Rule::schema, "\n");

    // with mixed whitespace
    assert_ast::<SchemaFile>(
        Rule::schema,
        "



    ",
    );

    // with comments
    assert_ast::<SchemaFile>(
        Rule::schema,
        "
        // comment
    ",
    );

    // with wild comments
    assert_ast::<SchemaFile>(
        Rule::schema,
        "
        // comment

        /*
            COMMENT
        */
    ",
    );

    // with comments
    assert_ast::<SchemaFile>(
        Rule::schema,
        "
        // comment

        /*
            COMMENT
        */

        ```
            MARKDOWN
        ```
    ",
    );

    assert_ast::<SchemaFile>(
        Rule::schema,
        "
        // comment

        /*
            COMMENT
        */

        ```
            MARKDOWN

            $```
                with inner markdown
            $```
        ```
    ",
    );

    assert_eq!(
        "\n// comment\n#element: String\n".trim(),
        "
        // comment
#element: String
    "
        .trim()
    );

    assert_eq!(
        r#"
        // comment
        #element: String
    "#,
        "
        // comment
        #element: String
    "
    );

    // with comments and elements
    assert_ast::<SchemaFile>(Rule::schema, "// comment\n#element: String");

    // with comments and elements
    assert_ast::<SchemaFile>(
        Rule::schema,
        "
        // comment
        #element: String
    ",
    );

    // with comments and elements
    assert_ast::<SchemaFile>(
        Rule::schema,
        r#"
        // comment
        #element: String
    "#,
    );

    // with comments and elements
    assert_ast::<SchemaFile>(
        Rule::schema,
        "
        // comment
        #element: String
    ",
    );

    // with comments and elements
    assert_ast::<SchemaFile>(
        Rule::schema,
        "
        // single Task that may contain subtasks
        // contains the @assigned attribute that can take multiple @id references
        // to people
        @assigned?: AssignedPersonIds
        Task {
            #ticket?: URI
            #description: String
            #subtasks?: TaskList
        }
        
        AssignedPersonIds: [IDRef]
        
        Estimate {
            // how many hours, days etc
            #amount: Double
        
            // enumerations and inline type restrictions are defined by regex
            #type: TimeUnit
        }
        
        
        TimeUnit: /days|hours|person days/",
    );
}

#[test]
fn test_schema_file() {
    // Parse your input string using the Pest parser
    let input = std::fs::read_to_string("./test.whas").unwrap();
    let mut parsed = WHASParser::parse(Rule::schema, input.as_str()).unwrap();

    // Convert the Pest AST to your Rust structs
    let schema = SchemaFile::from_pest(&mut parsed).unwrap();
    println!("{:#?}", schema);
}
