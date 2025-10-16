# WHAS Examples

This directory contains comprehensive examples demonstrating all supported WHAS syntax and features.

## Example Files

### 01-basic-types.whas
**Demonstrates:**
- All primitive types (String, Int, Float, Date, etc.)
- Simple element definitions
- Attributes (required and optional)
- Nested elements (blocks)
- Empty elements
- Mixed content

**Complexity:** Beginner
**Lines:** ~100

### 02-union-types.whas
**Demonstrates:**
- Basic type unions (`Int | String`)
- Literal value unions (enum-like: `"active" | "inactive"`)
- Numeric literal unions (`80 | 443 | 8080`)
- Mixed unions (types + literals)
- Regex patterns in unions
- Practical union type examples

**Complexity:** Beginner to Intermediate
**Lines:** ~150
**Feature Introduced:** Union types with `|` operator

### 03-advanced-types.whas
**Demonstrates:**
- Abstract types (`Type: a{ ... }`)
- Type inheritance (`DerivedType < BaseType`)
- Multiple inheritance levels
- Abstract + inheritance patterns
- Attribute inheritance
- Practical document structure example

**Complexity:** Intermediate to Advanced
**Lines:** ~200
**Feature Introduced:** Abstract types and inheritance with `<` operator

### 04-groups-and-modifiers.whas
**Demonstrates:**
- Duplicity modifiers (`?`, `*`, `+`, `[n]`, `[n..m]`)
- Occurrence modifiers (sequence, choice `?{}`, all `!{}`)
- Mixed content modifier (`x{}`)
- Type splatting (`...TypeName`)
- Inline groups
- List types

**Complexity:** Intermediate
**Lines:** ~250

### 05-complete-example.whas
**Demonstrates:**
- Complete blog system schema
- All features combined in realistic scenario
- Union types for flexible values
- Abstract base types
- Derived content, user, and media types
- Rich content structure
- Complex taxonomies
- Site configuration

**Complexity:** Advanced
**Lines:** ~350
**Use Case:** Real-world blog/CMS system

## Quick Reference

### Type Syntax

```whas
// Primitive type
#element: String

// Union type
FlexibleId: Int | String | "auto"

// Type alias
TypeAlias: String

// Abstract type
BaseType: a{ ... }

// Inheritance
DerivedType < BaseType { ... }

// Block (nested elements)
ComplexType {
    #child: String
}
```

### Modifiers

```whas
// Duplicity modifiers (on elements)
#optional?: String          // 0..1
#multiple*: String          // 0..*
#required+: String          // 1..*
#exact[3]: String           // exactly 3
#range[2..5]: String        // 2 to 5

// Occurrence modifiers (on blocks)
Type { ... }                // sequence (default)
Type: ?{ ... }              // choice
Type: !{ ... }              // all
Type: x{ ... }              // mixed content

// Combined
Type: ?x{ ... }             // choice + mixed

// Type splatting
...TypeName                 // inline type
...TypeName?                // with modifier
```

### Attributes

```whas
// Required attribute
@name: String
#element: Type

// Optional attribute
@email?: String

// Multiple attributes
@id: ID
@status: String
@created: DateTime
#item: Type
```

## Compiling Examples

To compile these examples to XSD:

```bash
# From the format directory
cargo run -- -x -o ./output examples/01-basic-types.whas
cargo run -- -x -o ./output examples/02-union-types.whas
# ... etc
```

To compile to Fonto schema JSON:

```bash
cargo run -- --fonto -o ./output examples/05-complete-example.whas
```

## Feature Coverage

| Feature | Example File |
|---------|-------------|
| Primitive types | 01-basic-types.whas |
| Elements & attributes | 01-basic-types.whas |
| Nested blocks | 01-basic-types.whas |
| Mixed content | 01-basic-types.whas |
| Union types | 02-union-types.whas |
| Literal unions (enums) | 02-union-types.whas |
| Abstract types | 03-advanced-types.whas |
| Inheritance | 03-advanced-types.whas |
| Duplicity modifiers | 04-groups-and-modifiers.whas |
| Occurrence modifiers | 04-groups-and-modifiers.whas |
| Type splatting | 04-groups-and-modifiers.whas |
| List types | 04-groups-and-modifiers.whas |
| Complete integration | 05-complete-example.whas |

## Learning Path

1. **Start with** `01-basic-types.whas` to understand fundamental concepts
2. **Move to** `02-union-types.whas` to learn the new union type feature
3. **Study** `03-advanced-types.whas` for type hierarchies and inheritance
4. **Explore** `04-groups-and-modifiers.whas` for structural patterns
5. **Review** `05-complete-example.whas` to see everything combined

## Additional Resources

- [Main README](../README.md) - Full syntax documentation
- [COMPLIANCE.md](../format/src/tests/schemas/xsd/COMPLIANCE.md) - XSD feature support matrix
- [Test schemas](../format/src/tests/schemas/xsd/) - More focused examples for specific features

## Notes

- All examples follow WHAS naming conventions:
  - Element names and attributes: lowercase
  - Type names: Capitalized

- Examples are designed to be self-contained and compilable

- Comments explain the purpose and behavior of each pattern

- Examples progress from simple to complex concepts

## Contributing

When adding new examples:
1. Follow the numbering pattern (06-, 07-, etc.)
2. Include comprehensive comments
3. Update this README with feature coverage
4. Ensure examples compile without errors
5. Focus on practical, real-world scenarios
