# WHAS XSD Compliance Matrix

This document shows which XSD features are supported by WHAS and which are not yet implemented.

## ✅ Fully Supported (25 features)

| XSD Feature | WHAS Syntax | Test File | Notes |
|-------------|-------------|-----------|-------|
| Primitive types | `String`, `Int`, `Bool`, `Date`, etc. | `primitives.whas` | All XSD built-in types mapped |
| xs:sequence | `{ ... }` (default) | `sequence.whas` | Default block behavior |
| xs:choice | `Type: ?{ ... }` with splat | `choice.whas` | Via type with choice modifier |
| xs:all | `!{ ... }` | `all.whas` | Exclamation prefix |
| xs:group | `...TypeName` | `group.whas` | Type splatting |
| Attributes | `@name: Type` | `attributes.whas` | Required and optional |
| Mixed content | `x{ ... }` | `mixed.whas` | Mixed content modifier |
| Occurrence constraints | `?`, `*`, `+`, `[n..m]` | `occurrences.whas` | minOccurs/maxOccurs |
| Pattern restrictions | `/regex/` | `restrictions.whas` | xs:pattern facet |
| Type derivation | `Type: BaseType` | `derivation.whas` | Type aliasing |
| List types | `[IDRef]` | `list.whas` | xs:list |
| Recursive types | Nested type references | `nested.whas` | Self-referencing types |
| Complex elements | Element with children + attrs | `complex_element.whas` | xs:complexType |
| Empty elements | `{}` | `empty.whas` | Elements with no content |
| Nested control structures | Groups within sequences | `choice_in_sequence.whas` | Composition |
| Multiple attributes | Multiple `@` declarations | `multi_attributes.whas` | Any number of attributes |
| Type splatting | `...Type` in blocks | `splat_modifiers.whas` | Group reuse |
| Realistic schemas | Complex nested structures | `realistic.whas` | Real-world example |
| xs:simpleContent | Simple type + attributes | `simple_content.whas` | Element with text + attrs |
| xs:complexContent | Child elements | `complex_content.whas` | Default for blocks |
| Enumeration (via regex) | `/val1\|val2\|val3/` | `facets_enumeration.whas` | Works but not ideal |
| xs:union | `Type1 \| Type2 \| "literal"` | `union.whas`, `union_literals.whas`, `union_mixed.whas` | Union types with pipe operator |
| Abstract types | `Type: a{ ... }` | `abstract.whas`, `abstract_inheritance.whas` | Cannot be directly instantiated |
| Inheritance | `DerivedType < BaseType { ... }` | `inheritance.whas`, `abstract_inheritance.whas` | xs:extension support |
| Attribute groups | Type splatting with attributes | `attribute_groups.whas` | Via type splatting workaround |

## 🟡 Partially Supported (1 feature)

| XSD Feature | Status | Test File | Notes |
|-------------|--------|-----------|-------|
| Default values | Attributes only? | `default_fixed_values.whas` | Need to verify model::Attribute support |

## ❌ Not Yet Supported (11 features)

| XSD Feature | Test File | Roadmap Status | Priority |
|-------------|-----------|----------------|----------|
| Namespaces | `namespaces.whas` | Marked as TODO | High |
| xs:any wildcard | `any_wildcard.whas` | Not mentioned | Medium |
| xs:anyAttribute | `any_attribute.whas` | Not mentioned | Medium |
| Substitution groups | `substitution_groups.whas` | Not mentioned | Low |
| Identity constraints | `identity_constraints.whas` | Not mentioned | Medium |
| Fixed values | `default_fixed_values.whas` | Related to default values TODO | Medium |
| Nillable elements | `nillable.whas` | Not mentioned | Low |
| Length facets | `facets_length.whas` | Not mentioned | Medium |
| Numeric facets | `facets_numeric.whas` | Not mentioned | Medium |
| whiteSpace facet | `facets_whitespace.whas` | Not mentioned | Low |
| elementFormDefault | `qualified_elements.whas` | Requires namespaces | Medium |
| block/final attributes | `block_final.whas` | Not mentioned | Low |
| xs:notation | `notation.whas` | Not mentioned | Very Low |

## Summary

- **Total XSD features tested**: 38
- **Fully supported**: 25 (66%)
- **Partially supported**: 1 (3%)
- **Not supported**: 11 (29%)
- **Recently added**: Union types, Abstract types, Inheritance/Extension

## Notes

### Attribute Groups
Can be simulated using type splatting, but requires creating a dummy element group with attributes. A dedicated attribute group syntax would be cleaner.

### Facets
Only `pattern` (regex) restrictions are supported. All other facets (length, numeric ranges, whitespace) are not yet implemented.

### Union Types
Union types are now fully supported using the `|` operator syntax (e.g., `Type1 | Type2 | "literal"`). Union types can only contain simple types, not complex types (blocks), as per XSD specification.

## Recommendations for XSD Export

When implementing an XSD exporter:

1. **High Priority**: Focus on the 25 fully supported features first
2. **Namespaces**: Critical for real-world XSD - should be prioritized
3. **Additional Facets**: Would improve validation capabilities (length, numeric ranges)
4. **Default/Fixed Values**: Would enable more constraint options

## Test Execution

All 38 tests pass:
- 21 tests verify supported features work correctly
- 2 tests verify partial support
- 15 tests explicitly fail with "not supported" messages

Run tests: `cargo test xsd --lib`
