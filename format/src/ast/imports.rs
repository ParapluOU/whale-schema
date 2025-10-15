use super::*;

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::import))]
pub enum Import {
    /// import "./other.whas" { Type1, Type2 }
    Extended(ImportExtended),

    /// import "./other.whas"
    /// import * from "./other.whas"
    Inline(ImportInline),
}

impl Import {
    pub fn path(&self) -> &Path {
        match self {
            Import::Inline(inline) => inline.path(),
            Import::Extended(extended) => extended.path(),
        }
    }

    pub fn is_wildcard(&self) -> bool {
        match self {
            Import::Inline(inline) => inline.is_wildcard(),
            Import::Extended(extended) => extended.is_wildcard(),
        }
    }

    pub fn is_absolute(&self) -> bool {
        self.path().is_absolute()
    }

    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    pub fn selector(&self) -> &ImportSelector {
        match self {
            Import::Inline(inline) => inline.selector.as_ref().unwrap(),
            Import::Extended(extended) => &extended.selector,
        }
    }

    pub fn absolute_path(&self, reference_dir: impl AsRef<Path>) -> PathBuf {
        if self.is_absolute() {
            self.path().to_path_buf()
        } else {
            reference_dir.as_ref().join(self.path())
        }
    }

    pub fn absolute_dir(&self, reference_dir: impl AsRef<Path>) -> PathBuf {
        self.absolute_path(reference_dir)
            .parent()
            .unwrap()
            .to_path_buf()
    }

    pub fn validate(&self, reference_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let abspath = self.absolute_path(&reference_dir);

        self.try_read_schema(Some(&reference_dir))
            .context(format!("error reading schema: {}", abspath.display()))?
            .validate_imports(self.absolute_dir(&reference_dir))
    }

    pub fn try_read_schema(
        &self,
        reference_dir: Option<impl AsRef<Path>>,
    ) -> anyhow::Result<SchemaFile> {
        let reference_dir = reference_dir
            .map(|rd| rd.as_ref().to_path_buf())
            .unwrap_or_default();
        let abspath = self.absolute_path(reference_dir);
        SchemaFile::new_file(&abspath)
            .context(format!("error reading schema: {}", abspath.display()))
    }

    // mimicing the one on ast::Schema.
    // read the actual imported file and provide a list of all exported types
    // note: will not return schema's imports
    // pub fn types_all(
    //     &self,
    //     reference_dir: Option<impl AsRef<Path>>,
    // ) -> anyhow::Result<Vec<TypeDef>> {
    //     Ok(self
    //         .try_read_schema(reference_dir)?
    //         .types_own()?
    //         .into_iter()
    //         .collect())
    // }

    // /// only the types explicitly listed in the import statement
    // pub fn types(&self, reference_dir: Option<impl AsRef<Path>>) -> anyhow::Result<Vec<TypeDef>> {
    //     if self.is_wildcard() {
    //         // return self.types_all(reference_dir);
    //         Err(anyhow::anyhow!(
    //             "cant safely read nested schema without recursion. Use SchemaFileManager instead"
    //         ))?
    //     }
    //
    //     // list of type names explicitly mentioned in the import statement
    //     let typenames = self
    //         .selector()
    //         .explicit_type_names()
    //         .into_iter()
    //         .map(|t| t.ident())
    //         .collect::<Vec<_>>();
    //
    //     // filter all type definitions in the referenced schema by the types in the selection
    //     Ok(self
    //         .types_all(reference_dir)?
    //         .into_iter()
    //         .filter(|t| typenames.contains(&t.ident()))
    //         .collect())
    // }
}

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::import_inline))]
pub struct ImportInline {
    pub selector: Option<ImportSelector>,
    pub path: ImportPath,
}

impl ImportInline {
    pub fn path(&self) -> &Path {
        Path::new(&self.path.value)
    }

    pub fn is_wildcard(&self) -> bool {
        self.selector.is_none() || self.selector.as_ref().unwrap().is_wildcard()
    }
}

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::import_extended))]
pub struct ImportExtended {
    pub path: ImportPath,
    pub selector: ImportSelector,
}

impl ImportExtended {
    pub fn path(&self) -> &Path {
        Path::new(&self.path.value)
    }

    pub fn is_wildcard(&self) -> bool {
        self.selector.is_wildcard()
    }
}

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::import_selector))]
pub enum ImportSelector {
    // *
    Any(SymbolModAny),
    // {Def, Def2}
    Types(ImportSelectorBlock),
}

impl ImportSelector {
    pub fn is_wildcard(&self) -> bool {
        match self {
            ImportSelector::Any(_) => true,
            ImportSelector::Types(_) => false,
        }
    }

    pub fn explicit_type_names(&self) -> Vec<TypeWithoutGeneric> {
        match self {
            ImportSelector::Any(_) => vec![],
            ImportSelector::Types(ImportSelectorBlock(explicits)) => {
                explicits.clone().unwrap_or_default()
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::import_selector_block))]
pub struct ImportSelectorBlock(Option<Vec<TypeWithoutGeneric>>);

#[derive(Debug, Eq, PartialEq, FromPest)]
#[pest_ast(rule(Rule::import_path))]
pub struct ImportPath {
    #[pest_ast(outer(with(span_into_str), with(strip_delimiters)))]
    pub value: String,
}
