use crate::export::Exporter;
use crate::model;
use anyhow::Result;

/// XSD XML Exporter - exports WHAS model to XSD (XML Schema Definition)
pub struct XsdExporter {
    /// Target namespace (if supported)
    target_namespace: Option<String>,
}

impl Default for XsdExporter {
    fn default() -> Self {
        Self {
            target_namespace: None,
        }
    }
}

impl Exporter for XsdExporter {
    type Output = String;

    fn export_schema(self, schema: &model::Schema) -> Result<Self::Output> {
        let mut xsd = String::new();

        // XML declaration
        xsd.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xsd.push('\n');

        // xs:schema root element
        xsd.push_str(r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema""#);

        if let Some(ns) = &self.target_namespace {
            xsd.push_str(&format!(r#" targetNamespace="{}""#, ns));
        }

        xsd.push_str(" elementFormDefault=\"qualified\"");
        xsd.push_str(">\n");

        // Export simple types (primitives are built into XSD, only custom types need export)
        // Sort type names for deterministic output
        let mut type_names = schema.all_type_names();
        type_names.sort();

        for type_name in &type_names {
            if let Some(simple_type) = schema.get_simpletype_by_name(type_name) {
                if !simple_type.is_builtin() {
                    xsd.push_str(&self.export_simple_type(type_name, simple_type, schema)?);
                }
            }
        }

        // Export complex types (groups) - sorted for deterministic output
        for type_name in &type_names {
            if let Some(group) = schema.get_group_by_name(type_name) {
                xsd.push_str(&self.export_complex_type(type_name, group, schema)?);
            }
        }

        // Export top-level elements (sorted by name for deterministic output)
        // Elements are named by their name() method, not via the type name mapping
        let mut root_elements = schema.get_elements_root();
        root_elements.sort_by_key(|el| el.name());

        for element in &root_elements {
            xsd.push_str(&self.export_element(element.name(), element, schema)?);
        }

        // Close schema
        xsd.push_str("</xs:schema>\n");

        Ok(xsd)
    }
}

impl XsdExporter {
    pub fn with_namespace(namespace: impl Into<String>) -> Self {
        Self {
            target_namespace: Some(namespace.into()),
        }
    }

    fn export_simple_type(
        &self,
        name: &str,
        simple_type: &model::SimpleType,
        schema: &model::Schema,
    ) -> Result<String> {
        let mut xsd = String::new();

        xsd.push_str(&format!("  <xs:simpleType name=\"{}\">\n", name));

        match simple_type {
            model::SimpleType::Derived { base, restrictions, .. } => {
                let base_name = base.resolve(schema).to_type_name(schema);
                xsd.push_str(&format!("    <xs:restriction base=\"xs:{}\">\n",
                    self.map_primitive_to_xsd(&base_name)));

                // Export all facets using helper
                xsd.push_str(&self.export_restrictions(restrictions, 3)?);

                xsd.push_str("    </xs:restriction>\n");
            }
            model::SimpleType::Union { member_types } => {
                xsd.push_str("    <xs:union memberTypes=\"");
                let members: Vec<String> = member_types
                    .iter()
                    .map(|t| {
                        let type_name = t.resolve(schema).to_type_name(schema);
                        format!("xs:{}", self.map_primitive_to_xsd(&type_name))
                    })
                    .collect();
                xsd.push_str(&members.join(" "));
                xsd.push_str("\"/>\n");
            }
            model::SimpleType::List { item_type, separator: _ } => {
                let item_name = item_type.resolve(schema).to_type_name(schema);
                xsd.push_str(&format!("    <xs:list itemType=\"xs:{}\"/>\n",
                    self.map_primitive_to_xsd(&item_name)));
            }
            model::SimpleType::Builtin { .. } => {
                // Should not reach here - builtins are filtered out
            }
        }

        xsd.push_str("  </xs:simpleType>\n");

        Ok(xsd)
    }

    fn export_complex_type(
        &self,
        name: &str,
        group: &model::Group,
        schema: &model::Schema,
    ) -> Result<String> {
        let mut xsd = String::new();

        // Add abstract attribute if type is abstract
        if group.is_abstract() {
            xsd.push_str(&format!("  <xs:complexType name=\"{}\" abstract=\"true\">\n", name));
        } else {
            xsd.push_str(&format!("  <xs:complexType name=\"{}\">\n", name));
        }

        // Handle inheritance with xs:extension
        if let Some(base_ref) = group.base_type() {
            let base_group = base_ref.resolve(schema);
            // Find the base type name
            if let Some(base_name) = schema.get_type_name_for_group(base_ref) {
                xsd.push_str("    <xs:complexContent>\n");
                xsd.push_str(&format!("      <xs:extension base=\"{}\">\n", base_name));

                // Export only local fields (not inherited)
                xsd.push_str(&self.export_group_content_local(group, schema, 4)?);

                xsd.push_str("      </xs:extension>\n");
                xsd.push_str("    </xs:complexContent>\n");
            } else {
                // Fallback if base name not found - export all content
                xsd.push_str(&self.export_group_content(group, schema, 2)?);
            }
        } else {
            // No inheritance - export group content normally
            xsd.push_str(&self.export_group_content(group, schema, 2)?);
        }

        xsd.push_str("  </xs:complexType>\n");

        Ok(xsd)
    }

    fn export_group_content(
        &self,
        group: &model::Group,
        schema: &model::Schema,
        indent_level: usize,
    ) -> Result<String> {
        let mut xsd = String::new();
        let indent = "  ".repeat(indent_level);

        // Determine group type
        let group_tag = match group.ty() {
            model::GroupType::Sequence => "xs:sequence",
            model::GroupType::Choice => "xs:choice",
            model::GroupType::All => "xs:all",
        };

        xsd.push_str(&format!("{}<{}>\n", indent, group_tag));

        // Export items
        for item in group.items() {
            match item {
                model::GroupItem::Element(el_ref) => {
                    let element = el_ref.resolve(schema);
                    xsd.push_str(&self.export_element_inline(element, schema, indent_level + 1)?);
                }
                model::GroupItem::Group(g_ref) => {
                    let nested_group = g_ref.resolve(schema);
                    xsd.push_str(&self.export_group_content(nested_group, schema, indent_level + 1)?);
                }
            }
        }

        xsd.push_str(&format!("{}</{}>\n", indent, group_tag));

        Ok(xsd)
    }

    /// Export only local group content (excludes inherited fields from base type)
    fn export_group_content_local(
        &self,
        group: &model::Group,
        schema: &model::Schema,
        indent_level: usize,
    ) -> Result<String> {
        let mut xsd = String::new();
        let indent = "  ".repeat(indent_level);

        // Determine group type
        let group_tag = match group.ty() {
            model::GroupType::Sequence => "xs:sequence",
            model::GroupType::Choice => "xs:choice",
            model::GroupType::All => "xs:all",
        };

        xsd.push_str(&format!("{}<{}>\n", indent, group_tag));

        // Export only local items (all items in this group are local by definition)
        // Inheritance is handled by XSD's extension mechanism
        for item in group.items() {
            match item {
                model::GroupItem::Element(el_ref) => {
                    let element = el_ref.resolve(schema);
                    xsd.push_str(&self.export_element_inline(element, schema, indent_level + 1)?);
                }
                model::GroupItem::Group(g_ref) => {
                    let nested_group = g_ref.resolve(schema);
                    xsd.push_str(&self.export_group_content(nested_group, schema, indent_level + 1)?);
                }
            }
        }

        xsd.push_str(&format!("{}</{}>\n", indent, group_tag));

        Ok(xsd)
    }

    fn export_element(
        &self,
        name: &str,
        element: &model::Element,
        schema: &model::Schema,
    ) -> Result<String> {
        let mut xsd = String::new();

        xsd.push_str(&format!("  <xs:element name=\"{}\"", name));

        // Add occurrence constraints
        xsd.push_str(&format!(" minOccurs=\"{}\"", element.min_occurs()));
        if let Some(max) = element.max_occurs() {
            xsd.push_str(&format!(" maxOccurs=\"{}\"", max));
        } else {
            xsd.push_str(" maxOccurs=\"unbounded\"");
        }

        // Get attributes
        let attrs = element.group_merged_attributes(schema);
        let has_attrs = !attrs.as_vec().is_empty();

        // Check if it has complex type
        if let Some(group_type) = element.typing().grouptype(schema) {
            xsd.push_str(">\n");
            // Check for mixed content
            if element.is_mixed_content(schema) {
                xsd.push_str("    <xs:complexType mixed=\"true\">\n");
            } else {
                xsd.push_str("    <xs:complexType>\n");
            }
            xsd.push_str(&self.export_group_content(group_type, schema, 3)?);
            xsd.push_str(&self.export_attributes(&attrs, schema, 3)?);
            xsd.push_str("    </xs:complexType>\n");
            xsd.push_str("  </xs:element>\n");
        } else if let model::TypeRef::Simple(simple_ref) = element.typing() {
            let simple_type = simple_ref.resolve(schema);

            // Check if this is an anonymous union (inline union)
            let is_anonymous_union = matches!(simple_type, model::SimpleType::Union { .. }) &&
                                     schema.get_type_name_for_simpletype(simple_ref).is_none();

            // Simple type with attributes (simpleContent)
            if has_attrs {
                xsd.push_str(">\n");
                xsd.push_str("    <xs:complexType>\n");
                xsd.push_str("      <xs:simpleContent>\n");

                if is_anonymous_union {
                    // For anonymous unions with attributes, we need to define the union inline
                    // Use xs:restriction with an inline simpleType
                    xsd.push_str("        <xs:restriction>\n");
                    xsd.push_str(&self.export_simple_type_inline(simple_type, schema, 5)?);
                    xsd.push_str(&self.export_attributes(&attrs, schema, 5)?);
                    xsd.push_str("        </xs:restriction>\n");
                } else {
                    // Named type - use extension
                    let type_name = self.get_simple_type_xsd_name(simple_ref, schema);
                    xsd.push_str(&format!("        <xs:extension base=\"{}\">\n", type_name));
                    xsd.push_str(&self.export_attributes(&attrs, schema, 5)?);
                    xsd.push_str("        </xs:extension>\n");
                }

                xsd.push_str("      </xs:simpleContent>\n");
                xsd.push_str("    </xs:complexType>\n");
                xsd.push_str("  </xs:element>\n");
            } else {
                // Simple type without attributes
                if is_anonymous_union {
                    // Export inline union
                    xsd.push_str(">\n");
                    xsd.push_str(&self.export_simple_type_inline(simple_type, schema, 2)?);
                    xsd.push_str("  </xs:element>\n");
                } else {
                    let type_name = self.get_simple_type_xsd_name(simple_ref, schema);
                    xsd.push_str(&format!(" type=\"{}\"/>\n", type_name));
                }
            }
        } else if has_attrs {
            // Attributes only (empty content)
            xsd.push_str(">\n");
            xsd.push_str("    <xs:complexType>\n");
            xsd.push_str(&self.export_attributes(&attrs, schema, 3)?);
            xsd.push_str("    </xs:complexType>\n");
            xsd.push_str("  </xs:element>\n");
        } else {
            xsd.push_str("/>\n");
        }

        Ok(xsd)
    }

    fn export_attributes(
        &self,
        attrs: &model::Attributes,
        schema: &model::Schema,
        indent_level: usize,
    ) -> Result<String> {
        let mut xsd = String::new();
        let indent = "  ".repeat(indent_level);

        // Sort attributes by name for deterministic output
        let mut attr_vec = attrs.as_vec().clone();
        attr_vec.sort_by_key(|attr_ref| attr_ref.resolve(schema).name());

        for attr_ref in attr_vec {
            let attr = attr_ref.resolve(schema);
            xsd.push_str(&format!("{}<xs:attribute name=\"{}\"", indent, attr.name()));

            // Type - attr.typing is directly a Ref<SimpleType>
            let attr_type = attr.typing.resolve(schema);

            // Check if this is an anonymous union type (inline union)
            if matches!(attr_type, model::SimpleType::Union { .. }) &&
               schema.get_type_name_for_simpletype(&attr.typing).is_none() {
                // Anonymous inline union - export inline
                xsd.push_str(">\n");
                xsd.push_str(&self.export_simple_type_inline(attr_type, schema, indent_level + 1)?);
                xsd.push_str(&format!("{}</xs:attribute>\n", indent));
            } else {
                // Named type or non-union - use type reference
                let type_name = self.get_simple_type_xsd_name(&attr.typing, schema);
                xsd.push_str(&format!(" type=\"{}\"", type_name));

                // Required/optional (use="required" vs use="optional")
                if *attr.required() {
                    xsd.push_str(" use=\"required\"");
                } // Optional is the default, no need to specify

                xsd.push_str("/>\n");
            }
        }

        Ok(xsd)
    }

    fn export_element_inline(
        &self,
        element: &model::Element,
        schema: &model::Schema,
        indent_level: usize,
    ) -> Result<String> {
        let indent = "  ".repeat(indent_level);
        let mut xsd = String::new();

        xsd.push_str(&format!("{}<xs:element name=\"{}\"", indent, element.name()));

        // Occurrence constraints
        xsd.push_str(&format!(" minOccurs=\"{}\"", element.min_occurs()));
        if let Some(max) = element.max_occurs() {
            xsd.push_str(&format!(" maxOccurs=\"{}\"", max));
        } else {
            xsd.push_str(" maxOccurs=\"unbounded\"");
        }

        // Type reference
        if let model::TypeRef::Simple(simple_ref) = element.typing() {
            let simple_type = simple_ref.resolve(schema);

            // Check if this is an anonymous type (inline facets or inline unions)
            if schema.get_type_name_for_simpletype(simple_ref).is_none() &&
               (simple_type.is_derived() || matches!(simple_type, model::SimpleType::Union { .. })) {
                // Export as anonymous inline simpleType
                xsd.push_str(">\n");
                xsd.push_str(&self.export_simple_type_inline(simple_type, schema, indent_level + 1)?);
                xsd.push_str(&format!("{}</xs:element>\n", indent));
            } else {
                // Named type reference
                let type_name = self.get_simple_type_xsd_name(simple_ref, schema);
                xsd.push_str(&format!(" type=\"{}\"", type_name));
                xsd.push_str("/>\n");
            }
        } else if let Some(group_type) = element.typing().grouptype(schema) {
            xsd.push_str(">\n");
            xsd.push_str(&format!("{}  <xs:complexType>\n", indent));
            xsd.push_str(&self.export_group_content(group_type, schema, indent_level + 2)?);
            xsd.push_str(&format!("{}  </xs:complexType>\n", indent));
            xsd.push_str(&format!("{}</xs:element>\n", indent));
        } else {
            xsd.push_str("/>\n");
        }

        Ok(xsd)
    }

    /// Export an inline anonymous simpleType (for inline facets or inline unions)
    fn export_simple_type_inline(
        &self,
        simple_type: &model::SimpleType,
        schema: &model::Schema,
        indent_level: usize,
    ) -> Result<String> {
        let mut xsd = String::new();
        let indent = "  ".repeat(indent_level);

        xsd.push_str(&format!("{}<xs:simpleType>\n", indent));

        match simple_type {
            model::SimpleType::Derived { base, restrictions, .. } => {
                let base_name = base.resolve(schema).to_type_name(schema);
                xsd.push_str(&format!("{}  <xs:restriction base=\"xs:{}\">\n", indent,
                    self.map_primitive_to_xsd(&base_name)));

                xsd.push_str(&self.export_restrictions(restrictions, indent_level + 2)?);

                xsd.push_str(&format!("{}  </xs:restriction>\n", indent));
            }
            model::SimpleType::Union { member_types } => {
                xsd.push_str(&format!("{}  <xs:union memberTypes=\"", indent));
                let members: Vec<String> = member_types
                    .iter()
                    .map(|t| {
                        let type_name = self.get_simple_type_xsd_name(t, schema);
                        // Only add xs: prefix if not already present
                        if type_name.starts_with("xs:") {
                            type_name
                        } else {
                            format!("xs:{}", self.map_primitive_to_xsd(&type_name))
                        }
                    })
                    .collect();
                xsd.push_str(&members.join(" "));
                xsd.push_str("\"/>\n");
            }
            _ => {
                // Shouldn't happen for inline types, but handle gracefully
            }
        }

        xsd.push_str(&format!("{}</xs:simpleType>\n", indent));

        Ok(xsd)
    }

    /// Export restriction facets (helper for reuse)
    fn export_restrictions(
        &self,
        restrictions: &model::restriction::SimpleTypeRestriction,
        indent_level: usize,
    ) -> Result<String> {
        let mut xsd = String::new();
        let indent = "  ".repeat(indent_level);

        // Enumeration facet
        if let Some(enumeration) = restrictions.enumeration.as_ref() {
            for value in enumeration {
                xsd.push_str(&format!("{}<xs:enumeration value=\"{}\"/>\n", indent,
                    Self::escape_xml(value)));
            }
        }

        // Length facets
        if let Some(length) = restrictions.length {
            xsd.push_str(&format!("{}<xs:length value=\"{}\"/>\n", indent, length));
        }
        if let Some(min_length) = restrictions.min_length {
            xsd.push_str(&format!("{}<xs:minLength value=\"{}\"/>\n", indent, min_length));
        }
        if let Some(max_length) = restrictions.max_length {
            xsd.push_str(&format!("{}<xs:maxLength value=\"{}\"/>\n", indent, max_length));
        }

        // Pattern facet
        if let Some(pattern) = restrictions.pattern.as_ref() {
            xsd.push_str(&format!("{}<xs:pattern value=\"{}\"/>\n", indent,
                Self::escape_xml(pattern)));
        }

        // Whitespace facet
        if let Some(white_space) = restrictions.white_space {
            let ws_value = match white_space {
                model::restriction::WhiteSpaceHandling::Preserve => "preserve",
                model::restriction::WhiteSpaceHandling::Replace => "replace",
                model::restriction::WhiteSpaceHandling::Collapse => "collapse",
            };
            xsd.push_str(&format!("{}<xs:whiteSpace value=\"{}\"/>\n", indent, ws_value));
        }

        // Numeric range facets
        if let Some(min_inclusive) = restrictions.min_inclusive.as_ref() {
            xsd.push_str(&format!("{}<xs:minInclusive value=\"{}\"/>\n", indent, min_inclusive));
        }
        if let Some(max_inclusive) = restrictions.max_inclusive.as_ref() {
            xsd.push_str(&format!("{}<xs:maxInclusive value=\"{}\"/>\n", indent, max_inclusive));
        }
        if let Some(min_exclusive) = restrictions.min_exclusive.as_ref() {
            xsd.push_str(&format!("{}<xs:minExclusive value=\"{}\"/>\n", indent, min_exclusive));
        }
        if let Some(max_exclusive) = restrictions.max_exclusive.as_ref() {
            xsd.push_str(&format!("{}<xs:maxExclusive value=\"{}\"/>\n", indent, max_exclusive));
        }

        // Decimal precision facets
        if let Some(total_digits) = restrictions.total_digits {
            xsd.push_str(&format!("{}<xs:totalDigits value=\"{}\"/>\n", indent, total_digits));
        }
        if let Some(fraction_digits) = restrictions.fraction_digits {
            xsd.push_str(&format!("{}<xs:fractionDigits value=\"{}\"/>\n", indent, fraction_digits));
        }

        Ok(xsd)
    }

    /// Get the XSD type name for a simple type reference
    /// Checks if the type has a custom name in the schema, otherwise returns the primitive type name
    fn get_simple_type_xsd_name(&self, simple_ref: &model::Ref<model::SimpleType>, schema: &model::Schema) -> String {
        let simple_type = simple_ref.resolve(schema);

        // Check if this is a builtin - builtins always use xs: prefix even if registered in schema
        if simple_type.is_builtin() {
            let base_name = simple_type.to_type_name(schema);
            return format!("xs:{}", self.map_primitive_to_xsd(&base_name));
        }

        // Check if this type has a custom name (like "FlexibleId")
        if let Some(custom_name) = schema.get_type_name_for_simpletype(simple_ref) {
            return custom_name;
        }

        // Otherwise, get the primitive base type and map to XSD
        let base_name = simple_type.to_type_name(schema);
        format!("xs:{}", self.map_primitive_to_xsd(&base_name))
    }

    /// Map WHAS primitive type names to XSD type names
    fn map_primitive_to_xsd(&self, whas_type: &str) -> String {
        match whas_type {
            "String" => "string",
            "Int" | "Integer" => "integer",
            "Bool" | "Boolean" => "boolean",
            "Date" => "date",
            "DateTime" => "dateTime",
            "DateTimestamp" => "dateTime",
            "Time" => "time",
            "Duration" => "duration",
            "Float" => "float",
            "Double" => "double",
            "Short" => "short",
            "Decimal" => "decimal",
            "ID" => "ID",
            "IDRef" => "IDREF",
            "IDRefs" => "IDREFS",
            "URI" => "anyURI",
            "Lang" => "language",
            "Name" => "Name",
            "NoColName" => "NCName",
            "-Int" => "negativeInteger",
            "+Int" => "nonNegativeInteger",
            "Token" => "token",
            "NameToken" => "NMTOKEN",
            "NameTokens" => "NMTOKENS",
            _ => whas_type, // Custom type, use as-is
        }.to_string()
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_mapping() {
        let exporter = XsdExporter::default();
        assert_eq!(exporter.map_primitive_to_xsd("String"), "string");
        assert_eq!(exporter.map_primitive_to_xsd("Int"), "integer");
        assert_eq!(exporter.map_primitive_to_xsd("Bool"), "boolean");
        assert_eq!(exporter.map_primitive_to_xsd("Date"), "date");
        assert_eq!(exporter.map_primitive_to_xsd("URI"), "anyURI");
    }
}
