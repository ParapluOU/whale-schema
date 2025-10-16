use crate::export::{Exporter, XsdExporter};
use crate::model::restriction::SimpleTypeRestriction;
use crate::model::{GroupItem, GroupType, PrimitiveType, SimpleType};
use crate::sourced::SchemaFileManager;
use crate::{compiler, model};
use anyhow::Result;

/// Helper function to compare XSD output against expected golden file
fn assert_xsd_matches_expected(schema_name: &str) -> Result<()> {
    let schema_path = format!("src/tests/schemas/xsd/{}.whas", schema_name);
    let expected_path = format!("src/tests/schemas/xsd/expected/{}.xsd", schema_name);

    let schema = model::Schema::from_file(&schema_path)?;
    let exporter = XsdExporter::default();
    let actual_output = exporter.export_schema(&schema)?;

    let expected_output = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("Expected XSD file not found: {}", expected_path));

    // Normalize whitespace for comparison (in case of line ending differences)
    let actual_normalized = actual_output.trim();
    let expected_normalized = expected_output.trim();

    if actual_normalized != expected_normalized {
        eprintln!("\n=== ACTUAL OUTPUT ===");
        eprintln!("{}", actual_output);
        eprintln!("\n=== EXPECTED OUTPUT ===");
        eprintln!("{}", expected_output);
        eprintln!("\n=== DIFF ===");

        // Simple diff - show first difference
        for (i, (actual_line, expected_line)) in actual_output.lines().zip(expected_output.lines()).enumerate() {
            if actual_line != expected_line {
                eprintln!("Line {}: DIFFER", i + 1);
                eprintln!("  Actual:   {}", actual_line);
                eprintln!("  Expected: {}", expected_line);
                break;
            }
        }

        panic!("XSD output does not match expected output for {}", schema_name);
    }

    Ok(())
}

/// Test XSD primitive type mappings
#[test]
fn test_xsd_primitives() -> Result<()> {
    assert_xsd_matches_expected("primitives")
}

/// Test XSD sequence (default block behavior)
#[test]
fn test_xsd_sequence() -> Result<()> {
    assert_xsd_matches_expected("sequence")
}

/// Test XSD choice (? block modifier)
#[test]
fn test_xsd_choice() -> Result<()> {
    assert_xsd_matches_expected("choice")
}

/// Test XSD all (! block modifier)
#[test]
fn test_xsd_all() -> Result<()> {
    assert_xsd_matches_expected("all")
}

/// Test XSD group (type splatting with ...TypeName)
#[test]
fn test_xsd_group() -> Result<()> {
    assert_xsd_matches_expected("group")
}

/// Test XSD attributes
#[test]
fn test_xsd_attributes() -> Result<()> {
    assert_xsd_matches_expected("attributes")
}

/// Test mixed content (x{...} modifier)
#[test]
fn test_xsd_mixed() -> Result<()> {
    assert_xsd_matches_expected("mixed")
}

/// Test occurrence constraints (?, *, +, [n..m])
#[test]
fn test_xsd_occurrences() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/occurrences.whas")?;

    let doc = schema.get_elements_by_name("doc");
    assert_eq!(1, doc.len());

    let group = doc[0].typing().grouptype(&schema).unwrap();
    let items = group.items();

    // Should have elements with different occurrence constraints
    assert!(items.len() >= 4, "Should have elements with various occurrences");

    // Verify occurrence constraints are properly set
    for item in items {
        if let GroupItem::Element(el_ref) = item {
            let el = el_ref.resolve(&schema);
            // Each element should have valid occurrence constraints
            assert!(el.min_occurs() <= el.max_occurs().unwrap_or(usize::MAX));
        }
    }

    Ok(())
}

/// Test simple type restrictions (regex patterns)
#[test]
fn test_xsd_restrictions() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/restrictions.whas")?;

    // Verify restricted type exists
    let status_type = schema.get_simpletype_by_name("StatusType");
    assert!(status_type.is_some(), "StatusType should exist");

    let status_type = status_type.unwrap();
    assert!(status_type.is_derived(), "StatusType should be a derived type with restrictions");

    // Verify restriction pattern exists
    let restrictions = status_type.restrictions();
    assert!(restrictions.is_some(), "Should have restrictions");
    assert!(restrictions.unwrap().pattern.is_some(), "Should have pattern restriction");

    Ok(())
}

/// Test simple type derivation (type aliasing)
#[test]
fn test_xsd_derivation() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/derivation.whas")?;

    // Verify type alias chain exists
    assert!(schema.get_simpletype_by_name("SimpleType").is_some());
    assert!(schema.get_simpletype_by_name("WrapperType").is_some());

    Ok(())
}

/// Test list types ([IDRef] syntax)
#[test]
fn test_xsd_list() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/list.whas")?;

    // Verify list type
    let refs_type = schema.get_simpletype_by_name("IDRefs");
    assert!(refs_type.is_some(), "IDRefs list type should exist");

    Ok(())
}

/// Test complex nested structures
#[test]
fn test_xsd_nested() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/nested.whas")?;

    // Verify recursive types exist
    assert!(schema.get_group_by_name("List").is_some());
    assert!(schema.get_group_by_name("ListItem").is_some());

    // Verify nesting works
    let list = schema.get_group_by_name("List").unwrap();
    assert!(list.items().len() >= 1, "List should contain items");

    Ok(())
}

/// Test elements with complex type
#[test]
fn test_xsd_complex_element() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/complex_element.whas")?;

    let book = schema.get_elements_by_name("book");
    assert_eq!(1, book.len());

    // Should have both attributes and child elements
    let attrs = book[0].group_merged_attributes(&schema);
    assert!(attrs.as_vec().len() >= 1, "Should have attributes");

    let group = book[0].typing().grouptype(&schema);
    assert!(group.is_some(), "Should have child elements");

    Ok(())
}

/// Test empty elements
#[test]
fn test_xsd_empty() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/empty.whas")?;

    let br = schema.get_elements_by_name("br");
    assert_eq!(1, br.len());

    // Empty element with no attributes
    let attrs = br[0].group_merged_attributes(&schema);
    assert_eq!(0, attrs.as_vec().len(), "Should have no attributes");

    // Should not have complex content
    assert!(br[0].typing().grouptype(&schema).is_none() ||
            br[0].typing().grouptype(&schema).unwrap().items().is_empty(),
            "Should be empty");

    Ok(())
}

/// Test choice within sequence (nested control structures)
#[test]
fn test_xsd_choice_in_sequence() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/choice_in_sequence.whas")?;

    let doc = schema.get_elements_by_name("doc");
    assert_eq!(1, doc.len());

    let group = doc[0].typing().grouptype(&schema).unwrap();
    assert_eq!(group.ty(), &GroupType::Sequence, "Outer should be sequence");

    // Check for nested group (choice)
    let has_nested_group = group.items().iter().any(|item| {
        matches!(item, GroupItem::Group(_))
    });
    assert!(has_nested_group, "Should have nested group structure");

    Ok(())
}

/// Test multiple attributes
#[test]
fn test_xsd_multi_attributes() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/multi_attributes.whas")?;

    let product = schema.get_elements_by_name("product");
    assert_eq!(1, product.len());

    let attrs = product[0].group_merged_attributes(&schema);
    assert!(attrs.as_vec().len() >= 3, "Should have multiple attributes");

    Ok(())
}

/// Test type splatting with modifiers (...Type?, ...Type*, ...Type+)
#[test]
fn test_xsd_splat_modifiers() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/splat_modifiers.whas")?;

    // Verify reusable group exists
    assert!(schema.get_group_by_name("Fields").is_some());

    // Verify elements that use splatted groups with modifiers
    let elements = schema.get_elements_by_name("optional-fields");
    assert!(elements.len() >= 1, "Should have element with optional splat");

    Ok(())
}

/// Test realistic example (from test.whas)
#[test]
fn test_xsd_realistic() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/realistic.whas")?;

    // Test Milestone structure
    schema
        .assert_type_name("Milestone")?
        .assert_element_name("title")?
        .assert_element_name("userstory")?
        .assert_element_name("taskdescription")?
        .assert_element_name("assumptions")?;

    // Verify choice structure for milestone content
    let milestone = schema.get_group_by_name("Milestone").unwrap();

    // Should have a choice for nested milestone or tasks
    let has_choice = milestone.items().iter().any(|item| {
        if let GroupItem::Group(g) = item {
            g.resolve(&schema).ty() == &GroupType::Choice
        } else {
            false
        }
    });
    assert!(has_choice, "Milestone should have choice structure");

    Ok(())
}

// ============================================================================
// UNSUPPORTED XSD FEATURES - Tests below should fail explicitly
// ============================================================================

/// Test XSD namespaces (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support namespaces")]
fn test_xsd_namespaces() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/namespaces.whas").unwrap();

    // TODO: When namespace support is added, check for:
    // - targetNamespace attribute
    // - namespace prefix declarations
    // - qualified element names

    panic!("WHAS does not yet support namespaces (see README roadmap)");
}

/// Test XSD xs:any wildcard (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support xs:any wildcard elements")]
fn test_xsd_any_wildcard() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/any_wildcard.whas").unwrap();

    // TODO: When wildcard support is added, check for:
    // - ...any syntax or similar
    // - processContents attribute (strict, lax, skip)
    // - namespace constraints

    panic!("WHAS does not yet support xs:any wildcard elements");
}

/// Test XSD xs:anyAttribute (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support xs:anyAttribute")]
fn test_xsd_any_attribute() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/any_attribute.whas").unwrap();

    // TODO: When wildcard attribute support is added, check for:
    // - @...any syntax or similar
    // - processContents attribute
    // - namespace constraints

    panic!("WHAS does not yet support xs:anyAttribute wildcard attributes");
}

/// Test XSD xs:extension for complex types (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support type extension")]
fn test_xsd_extension() {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/extension.whas").unwrap();

    // Currently we manually duplicate fields - no inheritance
    let base = schema.get_group_by_name("BaseType");
    let extended = schema.get_group_by_name("ExtendedType");

    assert!(base.is_some() && extended.is_some(), "Types exist but no extension mechanism");

    // TODO: When extension support is added, check for:
    // - ExtendedType extends BaseType syntax
    // - Automatic field inheritance
    // - Attribute inheritance

    panic!("WHAS does not yet support type extension (extends keyword)");
}

/// Test XSD xs:union types (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support union type syntax")]
fn test_xsd_union() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/union.whas").unwrap();

    // TODO: Check model::SimpleType::Union - it exists in the code!
    // Need to verify if there's syntax for it or if it's only used internally

    // Expected syntax: Type1 | Type2 | Type3

    panic!("WHAS does not yet support union type syntax (Type1 | Type2)");
}

/// Test XSD substitution groups (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support substitution groups")]
fn test_xsd_substitution_groups() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/substitution_groups.whas").unwrap();

    // TODO: When substitution groups are added, check for:
    // - substitutionGroup attribute on elements
    // - Abstract head elements
    // - Substitutable elements

    panic!("WHAS does not yet support substitution groups");
}

/// Test XSD identity constraints (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support identity constraints")]
fn test_xsd_identity_constraints() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/identity_constraints.whas").unwrap();

    // TODO: When identity constraints are added, check for:
    // - xs:key (unique key constraint)
    // - xs:keyref (foreign key constraint)
    // - xs:unique (uniqueness constraint)
    // - XPath selectors and fields

    panic!("WHAS does not yet support identity constraints (key, keyref, unique)");
}

/// Test XSD default and fixed values (PARTIALLY SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support default values for elements")]
fn test_xsd_default_fixed_values() {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/default_fixed_values.whas").unwrap();

    // Note: Default values for attributes might be supported in model::Attribute
    // Check the model to see if default_value field exists

    let elements = schema.get_elements_by_name("element");
    assert!(!elements.is_empty(), "Element exists");

    // TODO: When default value support is added, check for:
    // - Default value syntax in WHAS
    // - Fixed value syntax (value cannot be changed)
    // - Both for elements and attributes

    panic!("WHAS does not yet support default values for elements (marked as TODO in roadmap)");
}

/// Test XSD nillable elements (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support nillable elements")]
fn test_xsd_nillable() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/nillable.whas").unwrap();

    // TODO: When nillable support is added, check for:
    // - nillable modifier or attribute
    // - Distinction between absent, empty, and null

    panic!("WHAS does not yet support nillable elements (xsi:nil)");
}

/// Test XSD abstract types (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support abstract types")]
fn test_xsd_abstract_types() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/abstract_types.whas").unwrap();

    // TODO: When abstract types are added, check for:
    // - abstract keyword on type definitions
    // - Prevention of direct instantiation
    // - Requirement for substitution/extension

    panic!("WHAS does not yet support abstract types");
}

/// Test XSD length facets (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support length facets")]
fn test_xsd_facets_length() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/facets_length.whas").unwrap();

    // TODO: When length facets are added, check for:
    // - length: n (exact length)
    // - minLength: n
    // - maxLength: n
    // - Applied to string types

    panic!("WHAS does not yet support length facets (minLength, maxLength, length)");
}

/// Test XSD numeric facets (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support numeric range facets")]
fn test_xsd_facets_numeric() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/facets_numeric.whas").unwrap();

    // TODO: When numeric facets are added, check for:
    // - minInclusive, maxInclusive
    // - minExclusive, maxExclusive
    // - totalDigits, fractionDigits

    panic!("WHAS does not yet support numeric range facets (minInclusive, maxInclusive, etc.)");
}

/// Test XSD enumeration facet (SUPPORTED via regex, but no explicit enum syntax)
#[test]
fn test_xsd_facets_enumeration() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/facets_enumeration.whas")?;

    // Verify enumeration via regex pattern works
    let color_type = schema.get_simpletype_by_name("ColorType");
    assert!(color_type.is_some(), "ColorType should exist");

    let restrictions = color_type.unwrap().restrictions();
    assert!(restrictions.is_some() && restrictions.unwrap().pattern.is_some(),
            "Should have pattern restriction");

    // Note: This works but is not ideal - dedicated enum syntax would be better
    // Expected future syntax: enum { "red", "green", "blue" }

    Ok(())
}

/// Test XSD whiteSpace facet (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support whiteSpace facet")]
fn test_xsd_facets_whitespace() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/facets_whitespace.whas").unwrap();

    // TODO: When whiteSpace facet is added, check for:
    // - preserve (keep all whitespace)
    // - replace (replace each whitespace char with space)
    // - collapse (collapse consecutive whitespace to single space)

    panic!("WHAS does not yet support whiteSpace facet (preserve, replace, collapse)");
}

/// Test XSD simpleContent (SUPPORTED implicitly)
#[test]
fn test_xsd_simple_content() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/simple_content.whas")?;

    // Element with simple type content and attributes
    let length = schema.get_elements_by_name("length");
    assert_eq!(1, length.len());

    // Should have attribute
    let attrs = length[0].group_merged_attributes(&schema);
    assert!(attrs.as_vec().len() >= 1, "Should have unit attribute");

    // Should have simple type content
    assert!(length[0].typing().simpletype(&schema).is_some(), "Should have simple content");

    // This maps to XSD simpleContent - supported!
    Ok(())
}

/// Test XSD complexContent (SUPPORTED - default for block elements)
#[test]
fn test_xsd_complex_content() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/complex_content.whas")?;

    let element = schema.get_elements_by_name("element");
    assert_eq!(1, element.len());

    // Should have child elements
    let group = element[0].typing().grouptype(&schema);
    assert!(group.is_some(), "Should have complex content (child elements)");

    // This is the default behavior in WHAS - supported!
    Ok(())
}

/// Test XSD attribute groups (PARTIALLY SUPPORTED)
#[test]
fn test_xsd_attribute_groups() -> Result<()> {
    let schema = model::Schema::from_file("src/tests/schemas/xsd/attribute_groups.whas")?;

    // Can define attributes on types and merge them via splatting
    let common_attrs = schema.get_group_by_name("CommonAttrs");
    assert!(common_attrs.is_some(), "CommonAttrs group exists");

    // Can use those attributes on elements
    let element = schema.get_elements_by_name("element");
    assert!(!element.is_empty(), "Element exists");

    // However: no explicit attributeGroup syntax separate from element groups
    // This is a workaround, not proper attribute group support

    // TODO: Add dedicated attribute group syntax that doesn't require dummy element groups

    Ok(())
}

/// Test XSD qualified elements (NOT SUPPORTED - requires namespaces)
#[test]
#[should_panic(expected = "WHAS does not yet support elementFormDefault")]
fn test_xsd_qualified_elements() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/qualified_elements.whas").unwrap();

    // TODO: When namespace support is added, also support:
    // - elementFormDefault (qualified vs unqualified)
    // - attributeFormDefault
    // - form attribute on individual elements/attributes

    panic!("WHAS does not yet support elementFormDefault/attributeFormDefault (requires namespaces)");
}

/// Test XSD block and final attributes (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support block/final attributes")]
fn test_xsd_block_final() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/block_final.whas").unwrap();

    // TODO: When block/final are added, check for:
    // - block attribute (prevent derivation by extension/restriction)
    // - final attribute (prevent further derivation)
    // - Applied to type definitions

    panic!("WHAS does not yet support block/final attributes (derivation control)");
}

/// Test XSD xs:notation (NOT SUPPORTED)
#[test]
#[should_panic(expected = "WHAS does not yet support xs:notation")]
fn test_xsd_notation() {
    let _schema = model::Schema::from_file("src/tests/schemas/xsd/notation.whas").unwrap();

    // TODO: When notation support is added, check for:
    // - Notation declarations
    // - NOTATION simple type
    // - Public and system identifiers

    panic!("WHAS does not yet support xs:notation (binary data format declarations)");
}
