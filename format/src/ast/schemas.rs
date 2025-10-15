use super::*;
use pseudonym::alias;
use tap::Pipe;

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::schema))]
pub struct SchemaFile {
    /// optional top-level comments
    pub doc: Vec<Comment>,

    /// declaration of the namespace for this particular schema
    pub namespace: Option<Namespace>,

    /// specification of other types from other definitions that have to be included.
    /// the imported files may have different namespaces
    pub imports: Vec<Import>,

    /// all items for the current schema files
    items: Vec<SchemaItem>,

    /// end of file. required to be here to make sure that parsing doesnt quit halfway
    _eoi: FileEnd,
}

impl SchemaFile {
    #[alias(from_file)]
    pub fn new_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let ctx_msg = format!("reading schema from {}", path.as_ref().display());
        let mut path = Self::resolve_file_path(&path)?;

        let schema = Self::parse(std::fs::read_to_string(&path).context(ctx_msg)?.as_str())?;

        let parent_dir = path.parent().unwrap_or_else(|| Path::new(""));
        schema.validate_imports(parent_dir)?;

        Ok(schema)
    }

    pub fn parse(input: &str) -> anyhow::Result<Self> {
        let mut parsed = WHASParser::parse(Rule::schema, input)?;

        // Convert the Pest AST to your Rust structs
        Ok(SchemaFile::from_pest(&mut parsed)?)
    }

    /// resolve file path relative to where this schemafile is defined
    pub fn resolve_file_path(path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        let path = path.as_ref();
        let with_ext = path.with_extension("whas");
        Ok(if path.exists() {
            path.to_path_buf()
        } else if with_ext.exists() {
            with_ext
        } else {
            Err(anyhow::anyhow!(
                "file not found at paths: {}, {}",
                path.display(),
                with_ext.display()
            ))?
        })
    }

    pub fn has_imports(&self) -> bool {
        !self.imports.is_empty()
    }

    // make sure imports can be parsed.
    // the referemce is dir is the location of this Schema, relative to which the imports are resolved
    pub fn validate_imports(&self, reference_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        for import in &self.imports {
            import.validate(&reference_dir)?;
        }
        Ok(())
    }

    // all types from this document
    pub fn types_own(&self) -> Vec<&TypeDef> {
        self.items
            .iter()
            .filter_map(|item| match item {
                SchemaItem::TypeDefinition(typedef) => Some(typedef),
                _ => None,
            })
            .collect()
    }

    /// all type definitions suitable for attributes (like SimpleTypes)
    pub fn types_simple(&self) -> anyhow::Result<Vec<&TypeDef>> {
        Ok(self
            .types_own()
            .into_iter()
            .map(|ty| ty.is_simple_type(self).map(|res| res.then_some(ty)))
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter()
            .filter_map(identity)
            .collect())
    }

    pub fn types_group(&self) -> Vec<&TypeDef> {
        self.types_own()
            .into_iter()
            .filter(|ty| ty.is_group())
            .collect()
    }

    pub fn types_count(&self) -> usize {
        self.types_own().len()
    }

    pub fn elements_top_level(&self) -> Vec<&Element> {
        self.items
            .iter()
            .filter_map(|item| match item {
                SchemaItem::Element(element) => Some(element),
                _ => None,
            })
            .collect()
    }

    pub fn find_type(&self, name: &IdentTypeNonPrimitive) -> Option<&TypeDef> {
        self.types_own()
            .into_iter()
            .find(|item| item.is_named(name))
    }

    pub fn find_type_by_name(&self, name: &str) -> Option<&TypeDef> {
        self.types_own()
            .into_iter()
            .find(|item| item.has_name(name))
    }
}

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::schema_item))]
pub enum SchemaItem {
    Element(Element),
    TypeDefinition(TypeDef),
    Comment(Comment),
}

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::namespace_value))]
pub struct Namespace {
    #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
    pub value: String,
}
