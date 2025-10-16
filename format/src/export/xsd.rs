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

                // Export restrictions
                if let Some(pattern) = restrictions.pattern.as_ref() {
                    xsd.push_str(&format!("      <xs:pattern value=\"{}\"/>\n",
                        Self::escape_xml(pattern)));
                }

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
            let type_name = self.get_simple_type_xsd_name(simple_ref, schema);

            // Simple type with attributes (simpleContent)
            if has_attrs {
                xsd.push_str(">\n");
                xsd.push_str("    <xs:complexType>\n");
                xsd.push_str("      <xs:simpleContent>\n");
                xsd.push_str(&format!("        <xs:extension base=\"{}\">\n", type_name));
                xsd.push_str(&self.export_attributes(&attrs, schema, 5)?);
                xsd.push_str("        </xs:extension>\n");
                xsd.push_str("      </xs:simpleContent>\n");
                xsd.push_str("    </xs:complexType>\n");
                xsd.push_str("  </xs:element>\n");
            } else {
                // Simple type without attributes
                xsd.push_str(&format!(" type=\"{}\"/>\n", type_name));
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
            let type_name = attr_type.to_type_name(schema);
            xsd.push_str(&format!(" type=\"xs:{}\"", self.map_primitive_to_xsd(&type_name)));

            // Required/optional (use="required" vs use="optional")
            if *attr.required() {
                xsd.push_str(" use=\"required\"");
            } // Optional is the default, no need to specify

            xsd.push_str("/>\n");
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
            let type_name = self.get_simple_type_xsd_name(simple_ref, schema);
            xsd.push_str(&format!(" type=\"{}\"", type_name));
            xsd.push_str("/>\n");
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

    /// Get the XSD type name for a simple type reference
    /// Checks if the type has a custom name in the schema, otherwise returns the primitive type name
    fn get_simple_type_xsd_name(&self, simple_ref: &model::Ref<model::SimpleType>, schema: &model::Schema) -> String {
        // First check if this type has a custom name (like "FlexibleId")
        if let Some(custom_name) = schema.get_type_name_for_simpletype(simple_ref) {
            return custom_name;
        }

        // Otherwise, get the primitive base type and map to XSD
        let simple_type = simple_ref.resolve(schema);
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
