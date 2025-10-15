use crate::*;

use pest::error::Error;
use pest::iterators::Pairs;
use pest::Parser;

fn assert_ok(rule: Rule, input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    let res = WHASParser::parse(rule, input);
    assert!(
        res.is_ok(),
        "rule {:?} did not match '{input}'.\nResult: {:#?}",
        rule,
        &res
    );

    // if res.is_ok() {
    //     println!("{:#?}", res);
    // }

    res
}

fn assert_fail(rule: Rule, input: &str) {
    assert!(
        WHASParser::parse(rule, input).is_err(),
        "rule {:?} should have failed '{input}'",
        rule
    )
}

macro_rules! assert_all {
    ($rule:ident {
        ok {
            $($input_ok:expr,)+
        }
        $(fail {
            $($input_fail:expr,)*
        })?
    }) => {
        $(
            assert_ok(Rule::$rule, $input_ok);
        )+
        $($(
            assert_fail(Rule::$rule, $input_fail);
        )*)?
    }
}

macro_rules! test {
    ($testname:ident $test:tt) => {
        #[test]
        fn $testname {
            assert_all!{
                $testname $test
            }
        }
    }
}

#[test]
fn test_ident_lowercase() {
    assert_all! {
        ident_lowercase {
            ok {
                "regular",
                "with1number",
                "with-hyphen",
            }
        }
    }
}

#[test]
fn test_ident_capitalized() {
    assert_all! {
        ident_capitalized {
            ok {
                "Regular",
                "With1number",
                "WithoutHyphen",
            }
        }
    }
}

#[test]
fn test_type_with_generic() {
    assert_all! {
        type_with_generic {
            ok {
                "Empty()",
                "Regular(Text)",
                "List(Item)",
                "Block(Arg, Arg, Arg)",
            }
        }
    }
}

#[test]
fn test_type() {
    assert_all! {
        type_without_generic {
            ok {
                "Empty",
                "Regular(Text)",
                "List",
                "Block(Arg, Arg, Arg)",
            }
        }
    }
}

#[test]
fn test_typing() {
    assert_all! {
        typing {
            ok {
                "Empty",
                "Regular(Text)",
                "List",
                "Block(Arg, Arg, Arg)",
                "/regex|other/",
            }
        }
    }
}

#[test]
fn test_ident_element() {
    assert_all! {
        ident_element {
            ok {
                "#regular",
                "#with1number",
                "#with-hyphen",
            }
        }
    }
}

#[test]
fn test_ident_attr() {
    assert_all! {
        ident_attr {
            ok {
                "@attr-test",
            }
        }
    }
}

#[test]
fn test_element_with_type() {
    assert_all! {
        element_with_type {
            ok {
                "#element: TypeNoArgs",
                "#element : Type()",
                "#with-hyphen: Type(Arg, Arg)",
            }
        }
    }
}

#[test]
fn test_attrdef() {
    assert_all! {
        attrdef {
            ok {
                "@attr: String\n",
                "@attr?\n",
            }
        }
    }
}

#[test]
fn test_typedefinline() {
    assert_all! {
        typedef_inline {
            ok {
                "TimeUnit: /days|hours|person days/",
            }
        }
    }
}

#[test]
fn test_typedef() {
    assert_all! {
        typedef {
            ok {
                r#"@attr1?: String
                 @attr2?
                 @attr3
                 Type x{}"#,

                // r#"@attr1?: String
                //  @attr2?
                //  @attr
                //  Type: String"#,

                r#"@attr: /test/ + "-" + Int
                 Type x{}"#,

                // r#"@attr1?: String
                //  @attr2?
                //  @attr3
                //  Type(): String"#,

                // r#"@attr1?: String
                //  @attr2?
                //  @attr3
                //  Type(arg1): arg1"#,

                // r#"@attr1?: String
                //  @attr2?
                //  @attr3
                //  Type(arg1, arg2): List(arg1)"#,

                "AttrType: String + /test/ + \"8y9i\"",

                "TimeUnit: /days|hours|person days/",
            }
        }
    }
}

#[test]
fn test_element_inline() {
    assert_all! {
        element {
            ok {
                // empty inline
                "#title?: String",
                "#element: /regex|not-regex/",
            }
        }
    }
}

#[test]
fn test_primitive() {
    assert_all! {
        primitive {
            ok {
                "Bool",
                "Boolean",
            }
        }
    }
}

#[test]
fn test_element_block() {
    assert_all! {
        element_with_block {
            ok {
                // simple block def
                "#workplan {
                    #title?: String
                }",
            }
        }
    }
}

#[test]
fn test_element() {
    assert_all! {
        element {
            ok {
                // empty block (no children allowed)
                "#workplan {}",

                // empty inline
                "#title?: String",

                // simple block def
                "#workplan {
                    #title?: String
                }",

                // simple nested block def
                "#workplan x{
                    #title?: String
                }",

                // nested block def
                "#workplan {
                    #title?: String
                    #meta? {
                        #project? {
                            // optional project duration data
                            #duration? {
                                //
                            }
                        }
                    }
                }",

                "#workplan {
                    #title?: String
                    #identifier?: String

                    // container for metadata about the project planning
                    #meta? {
                        #project? {
                            // optional project name
                            #name?: String

                            // optional project start date
                            #start?: Date

                            // optional project end date
                            #end?: Date

                            // optional project duration data
                            #duration? {
                                // project duration in terms of time
                                #time?: Estimate

                                // optional project duration accounted in money terms
                                #money?: Price
                            }
                        }
                    }

                    // introductory text about the project
                    #introduction?: Text

                    // general top-level assumptions
                    #assumptions?: List

                    // list of nesteable milestones
                    #milestone+: Milestone

                    // table of deliverables
                    #deliverables? x{
                        #deliverable*: {
                            #title: String,
                            #description: Text
                        }
                    }

                    // list of supplied materials by customer
                    #supplied-material? x{
                        #material*: {
                            #description: String,
                            #due: Date
                        }
                    }

                    // list of costs per role
                    #resources? x{
                        #rates?: {
                            #rate*: {
                                #role: String
                                #amount: Price
                            }
                        }

                        // todo: cost breakdown table
                    }

                    #customer-responsibilities?: Text
                    #contractor-responsibilities?: Text
                    #terms-conditions?: Text
                }",
            }
        }
    }
}

#[test]
fn test_comment_line() {
    assert_all! {
        comment_line {
            ok {
                "// this is a comment\n",
                "//\n",
                "// comment \n \n",
            }
        }
    }
}

#[test]
fn test_comment_md() {
    assert_all! {
        comment {
            ok {
                r#"```
                    this is a **markdown** comment.

                    It can $```document markdown$```
                ```"#,
            }
        }
    }
}

// #[test]
// fn test_schema_part() {
//     let part = r#"```
//             WHALE SCHEMA FILE
//             #############################
//
//             ## Design goals
//             - should be compileable to XML Schema and/or Fonto compiled schema
//             - should be simple to write, and simple to read
//             - should feel like an intuitive DSL with as little boilerplate as possible
//
//             ## Syntax Rules
//             ### Documentation
//             comments are written using two slashes inside schema definitions and support Markdown:
//
//                 // this is a **comment**
//
//             or within Markdown backticks (without the escape symbol) such as
//
//                 $```
//                     # Markdown header
//                 $```
//
//             Note that Markdown headings need a space between the pound and the text,
//             to disambiguate from example #element-names.
//
//             ## Naming convention
//             Element names and attribute names are always lowercase.
//             Names of types are always capitalized.
//
//
//             ### elements
//             elements are defined using a hash symbol and then the element name
//             like so:
//
//                 #element
//
//             if elements should be able to contain nested data, follow the declaration
//             with curly brackets to create a block:
//
//                 #element { .. }
//
//
//             ### Typing
//             Element definitions support a newline-oriented sequence
//             of 'element: Type' pairs. The colon is _optional_ when a block is provided as type:
//
//                 #mandatory: String // mandatory colon
//
//                 #optional: { .. }
//                 #optional  { .. } // optional colon
//
//             When multiple type definitions should be on a single line,
//             separate them by a comma:
//
//                 #one-liner { #first: String, #second: String }
//
//
//             ##### Data Types / Primitives (XSD => .whas primitive)
//             We support most XSD primitives for interoperability.
//
//             - xsd:string => String,
//             - xsd:anyURI => URI,
//             - xsd:date => Date,
//             - xsd:dateTime => DateTime,
//             - xsd:dateTimeStamp => DateTimestamp,
//             - xsd:time => Time,
//             - xsd:duration => Duration,
//             - xsd:boolean => Bool,
//             - xsd:integer => Int,
//             - xsd:float => Float,
//             - xsd:double => Double,
//             - xsd:short => Short,
//             - xsd:decimal => Decimal,
//             - xsd:ID => ID,
//             - xsd:IDREF => IDRef,
//             - xsd:IDREFS => [IDRef],
//             - xsd:language => Lang,
//             - xsd:Name => Name,
//             - xsd:NCName => NoColName,
//             - xsd:negativeInteger => -Int,
//             - xsd:nonNegativeInteger => +Int,
//             - xsd:token => Token,
//             - xsd:NMTOKEN => NameToken,
//             - xsd:NMTOKENS => [NameToken],
//
//
//             ### Modifiers
//
//             #### Duplicity modifiers
//             An element may be specified as optional by postfixing it with a '?' like in TypeScript.
//             An element may be specified as zero-or-more by postfixing it with a '*' like in DTD.
//             An element may be specified as one-or-more by postfixing it with a '+' like in DTD.
//             An element may be specified as limited in its repeating by postfixing a range like [0..3] or [5]
//
//                 #element-optional?: String,
//                 #element-zero-or-more*: String,
//                 #element-repeat[5]: String,
//
//
//             #### Occurrence modifiers
//             By default, a nested element definition behaves like <xs:sequence>, where every element
//             must occur _in order_. To configure the block to behave like a <xs:choice>, where only a single
//             element must be chosen, prefix the block with a block modifier like so:
//
//                 #choice-block-element ?{ #choice1: String, #choice2: String }
//
//             To configure the block to behave like an <xs:all>, where all elements must be present (subject to their modifiers)
//             _irregardless of ordering_, use the exclamation prefix:
//
//                 #all-present !{ #must1: String, #must2: String }
//
//
//             #### Mixed content modifier
//             To configure a block to allow mixed elements and plaintext, use the 'mixed content modifier':
//
//                 #allow-mixed-content: x{ .. }
//
//             This modifier can ironically be mixed with the occurrance modifier:
//
//                 #allow-mixed-content: ?x{ .. }
//                 #allow-mixed-content: x?{ .. }
//
//
//             ### Groups
//             Groups are created by encapsulating elements in a block without assigning
//             the block as an element. The group can have occurrence modifiers:
//
//                 #element {
//                     #sub: String
//
//                     ?{
//                         #choice1: String
//                         #choice2: String
//                     }
//                 }
//
//
//             ### Type definitions
//             Instead of only using the built-in XSD types, custom types can be defined and reused.
//             To do so, define a **top-level** capitalized type name with a block:
//
//                 NewType { .. }
//
//             A Type can be used for element declarations:
//
//                 #element: NewType
//
//             But types can also function like <xs:group> and be splatted into a structure:
//
//                 #element {
//                     #field1: String,
//
//                     ...NewType
//
//                     #field3: String
//                 }
//
//             Type definitions still support occurrence modifiers:
//
//                 NewType ?{ #choice1: String, #choice2: String }
//
//             So these are the same:
//
//                 #element {
//                     ...NewType
//                 }
//
//                 #element {
//                     ...?{ #choice1: String, #choice2: String }
//                 }
//
//
//             #### Generics
//             To prevent having to statically define all variants of types under different contexts,
//             type definitions support generics that can be expanded inside the block. All type arguments
//             are optional by default
//
//                 // list definition
//                 List(itemType) {
//                     #item+: ListItem(itemType)
//                 }
//
//                 // type definition of a list item with a generic content type
//                 ListItem(content) {
//                     // expand the 'content' Type argument in this position
//                     ...content
//
//                     #list?: List
//                 }
//
//                 ..
//
//                 #element {
//
//                     #milestones: List(Milestone)
//
//                     #plaintextlist: List(x{}) //empty block with mixed content
//
//                 }
//
//
//             #### Extend / Redefine
//             todo: still needed if we have generic args?
//
//
//             ### Core Types
//             A few Types are defined by default in the compiler:
//
//                 // plaintext; Empty Block with mixed type so that text nodes are allowed
//                 // but no subelements are defined
//                 Plain x{}
//
//
//             ### Attributes
//             todo
//
//         ```
//
//         //
//         // WORK PLAN DOCUMENT
//         //
//         #workplan {
//             #title?: String
//         }
//         "#;
//     if let Ok(mut res) = assert_ok(Rule::schema, part) {
//         let schema = res.next().unwrap();
//         for thing in schema.into_inner() {
//             match thing.as_rule() {
//                 Rule::COMMENT => {
//                     println!("[comment] {}", thing.as_str());
//                 }
//                 Rule::element => {
//                     println!("[element] {}", thing.as_str());
//                 }
//                 Rule::typedef => {
//                     println!("[type] {}", thing.as_str());
//                 }
//                 Rule::EOI => {}
//                 _ => panic!("unhandled top-level construct {:#?}", thing.as_rule()),
//             }
//         }
//     }
// }

#[test]
fn test_schema() {
    assert_ok(
        Rule::schema,
        std::fs::read_to_string("./test.whas").unwrap().as_str(),
    );
}
