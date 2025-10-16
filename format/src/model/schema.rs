use crate::ast::TypeDef;
use crate::model::attr::Attribute;
use crate::model::element::Element;
use crate::model::group::Group;
use crate::model::primitive::PrimitiveType;
use crate::model::r#type::TypeMap;
use crate::model::simpletype::SimpleType;
use crate::model::typehash::{GetTypeHash, TypeHash};
use crate::model::{primitive, simpletype, Comment, TypeBor, TypeRef, TypeVariant};
use crate::sourced::{SchemaFileManager, SourcedSchemaFile};
use crate::validation::ValidationError;
use crate::Rule::typedef;
use crate::{ast, compiler, model, tools::default};
use anyhow::anyhow;
use derive_getters::Getters;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(PartialEq, Eq, Debug, Clone, Getters)]
pub struct Schema {
    /// simple value types. These are type definitions and have no name associated with them
    types_simple: TypeMap<SimpleType>,

    /// complex types with attributes and sub-elements.
    /// these are type definitions and have no name associated with them
    types_group: TypeMap<Group>,

    /// attributes are always explicit and uniqified by type, not by namw
    types_attribute: TypeMap<Attribute>,

    /// mapping from id to type definition name.
    /// the associated name is mostly a matter for the schema author because they are not
    /// neccessary internally. The ID is the important part.
    /// In the compiler we should only use these ID's to make sure that every Type cannot be used
    /// as-is but has to go through this Schema to be resolved
    mapping_type_id_name: IdMap<HashSet<String>>,

    /// mapping from id to type definition hash. The hash has to be checked in all three
    /// type maps.
    mapping_type_id_hash: IdMap<TypeHash>,

    /// element definitions
    elements: TypeMap<Element>,

    /// buffer that builds comment elements until a new breaking element is registered
    /// after which the comments are cleared and assignrd to that new element
    _buffer_comments: Vec<Comment>,
}

impl Default for Schema {
    fn default() -> Self {
        let mut instance = Self {
            types_simple: Default::default(),
            types_group: Default::default(),
            types_attribute: Default::default(),
            mapping_type_id_name: Default::default(),
            mapping_type_id_hash: Default::default(),
            elements: Default::default(),
            _buffer_comments: vec![],
        };

        // register simple types
        for st in PrimitiveType::iter() {
            let count_before = instance.types_simple.len();
            instance.register_primitive_type(st).unwrap();

            assert_eq!(
                count_before + 1,
                instance.types_simple.len(),
                "expected new simpletype to be registered"
            )
        }

        instance
    }
}

impl Schema {
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        compiler::compile(&SchemaFileManager::from_root_schema(path)?)
    }

    //
    // MAIN REGISTRATION FUNCTIONS
    //

    /// register definition type names before we do anything else.
    /// this will attach ID's to them that we can use to create references in type definitions.
    /// This doesnt register any type definition itself yet and doesnt map any type name to any definition yet
    pub fn register_type_definition_name(
        &mut self,
        type_id: &SchemaObjId,
        top_level_de: &ast::TypeDef,
    ) -> anyhow::Result<&SchemaObjId> {
        self.register_type_name(&type_id, top_level_de.ident_nonprim().to_string())
    }

    pub fn register_attribute(
        &mut self,
        top_level_de: model::Attribute,
    ) -> anyhow::Result<Ref<model::Attribute>> {
        let hash = top_level_de.id();
        self.types_attribute.insert(hash, top_level_de);
        let new_id = self.register_type_mapping(hash)?;
        Ok(Ref(new_id.clone(), default()))
    }

    /// register a (custom) simple type definition by its hash. since we have no name attached to it,
    /// there is no ID yet
    pub fn register_simple_type(
        &mut self,
        top_level_de: model::SimpleType,
    ) -> anyhow::Result<Ref<SimpleType>> {
        let hash = top_level_de.id();
        // safe and idempotent
        self.types_simple.insert(hash, top_level_de);
        let new_id = self.register_type_mapping(hash)?;
        Ok(Ref(new_id.clone(), default()))
    }

    pub fn register_group(&mut self, top_level_de: model::Group) -> anyhow::Result<Ref<Group>> {
        let hash = top_level_de.id();
        self.types_group.insert(hash, top_level_de);
        let new_id = self.register_type_mapping(hash)?;
        Ok(Ref(new_id.clone(), default()))
    }

    /// register element and generate id for it
    pub fn register_element(
        &mut self,
        top_level_de: model::Element,
    ) -> anyhow::Result<Ref<Element>> {
        let hash = top_level_de.id();
        self.elements.insert(hash.clone(), top_level_de);
        let id = self.register_type_mapping(hash.clone())?;
        Ok(Ref(id.clone(), default()))
    }

    /// register a primitive as a SimpleType. Since primitives have inherent names, we
    /// can register them by name and generate/retrieve ID's for the types
    pub fn register_primitive_type(
        &mut self,
        top_level_de: primitive::PrimitiveType,
    ) -> anyhow::Result<Ref<SimpleType>> {
        // create the type that we are saving
        let simpletype = SimpleType::from(top_level_de);
        // get the type hash, because we need it later
        let typehash = simpletype.id();
        // register the type itself
        let reff = self.register_simple_type(simpletype)?;
        // register the type name and map to a Type ID
        let type_id = self
            .register_type_name(&*reff, top_level_de.to_string())?
            .clone();

        Ok(reff)
    }

    pub fn register_preliminary_id_type(
        &mut self,
        prelim_id: &SchemaObjId,
        target_ty: TypeRef,
    ) -> anyhow::Result<TypeRef> {
        let id = prelim_id;
        let target_typehash = self
            .typehash_for_id(target_ty.schema_object_id())
            .unwrap()
            .clone();
        if self.mapping_type_id_hash.contains_key(id) {
            Err(anyhow!("preliminary type ID already mapped to type"))?;
        }
        self.mapping_type_id_hash
            .insert(id.clone(), target_typehash);
        Ok(target_ty)
    }

    pub fn push_comment(&mut self, comment: model::Comment) {
        self._buffer_comments.push(comment);
    }

    //
    // HELPERS
    //

    /// request a priliminary id for a type that is not resolved yet, needed as circuit breaker
    pub fn preliminary_ref_for_typename(
        &self,
        typedefinition: &TypeDef,
        source: &SourcedSchemaFile,
    ) -> Option<PreliminaryId> {
        let type_id = self.id_for_type_definition(typedefinition)?;
        PreliminaryId(match typedefinition.type_variant(source).unwrap() {
            TypeVariant::Simple => {
                let rf: Ref<SimpleType> = Ref(type_id.clone(), default());
                rf.into()
            }
            TypeVariant::Group => {
                let rf: Ref<Group> = Ref(type_id.clone(), default());
                rf.into()
            }
        })
        .into()
    }

    pub fn has_type_definition(&self, hash: &TypeHash) -> bool {
        self.types_group.contains_key(hash)
            || self.types_simple.contains_key(hash)
            || self.types_attribute.contains_key(hash)
            || self.elements.contains_key(hash)
    }

    pub fn id_for_type_default(&self) -> Option<&SchemaObjId> {
        self.id_for_type(&model::Type::default()) // String type
    }

    /// lookup the ID for a named type definition
    pub fn id_for_type_definition(&self, typedefinition: &ast::TypeDef) -> Option<&SchemaObjId> {
        let typedef_name = typedefinition.ident_nonprim().to_string();
        self.mapping_type_id_name
            .iter()
            .find(|(_, names)| names.contains(&typedef_name))
            .map(|(id, _)| id)
    }

    /// given a certain Type definition, retrieve the ID that is associated with it, if any
    pub fn id_for_type(&self, typedefinition: &model::Type) -> Option<&SchemaObjId> {
        self.id_for_type_hash(&typedefinition.id())
    }

    pub fn typehash_for_id(&self, id: &SchemaObjId) -> Option<&TypeHash> {
        self.mapping_type_id_hash.get(id)
    }

    pub fn all_type_names(&self) -> Vec<&String> {
        self.mapping_type_id_name
            .values()
            .flat_map(|set| set.into_iter())
            .collect()
    }

    //
    // ASSERTS
    //

    pub fn assert_type_definition(&self, hash: &TypeHash) -> anyhow::Result<&Self> {
        self.has_type_definition(hash)
            .then_some(self)
            .ok_or(anyhow!("no type found with type hash {}", hash))
    }

    pub fn assert_type_name(&self, name: &str) -> anyhow::Result<&Self> {
        self.id_for_type_name(name)
            .map(|res| self)
            .ok_or(anyhow!("no type found with name '{}'", name))
    }

    pub fn assert_element_name(&self, name: &str) -> anyhow::Result<&Self> {
        self.elements
            .values()
            .find(|el| el.name() == name)
            .map(|_| self)
            .ok_or(anyhow!("no element found with name '{}'", name))
    }

    //
    // GET components
    //

    pub fn get_attribute(&self, rf: &Ref<Attribute>) -> Option<&Attribute> {
        self.types_attribute.get(&*self.typehash_for_id(&rf.0)?)
    }

    pub fn get_attribute_name(&self, rf: &Ref<Attribute>) -> Option<&String> {
        self.get_attribute(rf).map(|attr| &attr.name)
    }

    pub fn get_simpletype(&self, rf: &Ref<SimpleType>) -> Option<&model::SimpleType> {
        self.types_simple.get(&*self.typehash_for_id(&rf.0)?)
    }

    pub fn get_simpletype_ref(&self, rf: &SimpleType) -> Option<Ref<model::SimpleType>> {
        self.id_for_type_hash(&rf.id())
            .map(|id| Ref(id.clone(), default()))
    }

    pub fn get_simpletype_by_name(&self, target: impl AsRef<str>) -> Option<&SimpleType> {
        for (id, names) in &self.mapping_type_id_name {
            if names.contains(target.as_ref()) {
                return self.get_simpletype(&Ref(id.clone(), default()));
            }
        }

        None
    }

    pub fn get_group(&self, rf: &Ref<Group>) -> Option<&Group> {
        self.types_group.get(&*self.typehash_for_id(&rf.0)?)
    }

    pub fn get_group_by_name(&self, target: impl AsRef<str>) -> Option<&Group> {
        for (id, names) in &self.mapping_type_id_name {
            if names.contains(target.as_ref()) {
                return self.get_group(&Ref(id.clone(), default()));
            }
        }

        None
    }

    /// Get the type name for a given Group reference (for XSD export)
    pub fn get_type_name_for_group(&self, group_ref: &Ref<Group>) -> Option<String> {
        // Find the ID's type names
        self.mapping_type_id_name
            .get(&group_ref.0)
            .and_then(|names| names.iter().next().cloned())
    }

    /// Get the type name for a given SimpleType reference (for XSD export)
    pub fn get_type_name_for_simpletype(&self, simple_ref: &Ref<SimpleType>) -> Option<String> {
        // Find the ID's type names
        self.mapping_type_id_name
            .get(&simple_ref.0)
            .and_then(|names| names.iter().next().cloned())
    }

    pub fn get_element(&self, rf: &Ref<Element>) -> Option<&Element> {
        self.elements.get(&*self.typehash_for_id(&rf.0)?)
    }

    pub fn get_element_ref(&self, rf: &Element) -> Option<Ref<Element>> {
        self.id_for_type_hash(&rf.id())
            .map(|id| Ref(id.clone(), default()))
    }

    pub fn get_elements_by_name(&self, name: &str) -> Vec<&Element> {
        self.elements
            .values()
            .filter(|el| el.name() == name)
            .collect()
    }

    /// get all localName elements that only exist in Group definitions
    /// todo: determine this when compiling
    pub fn get_elements_local(&self) -> Vec<&Element> {
        self.elements
            .values()
            .filter(|el| el.is_local(self))
            .collect()
    }

    /// get all elements that are defined in the root of the schema
    pub fn get_elements_root(&self) -> Vec<&Element> {
        let local = self.get_elements_local();
        self.elements
            .values()
            .filter(|el| !local.contains(el))
            .collect()
    }

    /// try retrieve a Type definition by its user-defined name or alias
    pub fn get_type_by_name(&self, name: &str) -> Option<TypeBor> {
        self.id_for_type_name(name).and_then(|id| {
            self.mapping_type_id_hash
                .get(id)
                .and_then(|hash| self.get_type_by_hash(hash))
        })
    }

    ///try retrieve a Type definition by its hash
    pub fn get_type_by_hash(&self, hash: &TypeHash) -> Option<TypeBor> {
        self.types_simple
            .get(hash)
            .map(|st| TypeBor::Simple(st))
            .or_else(|| self.types_group.get(hash).map(|g| TypeBor::Group(g)))
    }

    //
    // VALIDATION
    //

    pub fn validate(&self, xml: &String) -> Result<(), Vec<ValidationError>> {
        // let doc = roxmltree::Document::parse(&xml)?;
        // let root = doc.root_element();
        // let root_name = root.tag_name().name();
        // let root_element = self.get_element_by_name(root_name).ok_or(anyhow!(
        //     "root element '{}' not found in schema",
        //     root_name
        // ))?;
        //
        // self.validate_element(root_element, root)?;

        // todo

        Ok(())
    }

    //
    // PRIVATE
    //

    fn id_for_type_hash(&self, typehash: &TypeHash) -> Option<&SchemaObjId> {
        self.mapping_type_id_hash
            .iter()
            .find(|(_, hash)| hash == &typehash)
            .map(|(id, _)| id)
    }

    fn id_for_type_name(&self, typename: &str) -> Option<&SchemaObjId> {
        self.mapping_type_id_name
            .iter()
            .find(|(_, tnames)| tnames.contains(typename))
            .map(|(id, _)| id)
    }

    /// associate a type ID with a concrete type definition hash
    fn register_type_mapping(&mut self, hash: TypeHash) -> anyhow::Result<&SchemaObjId> {
        // assert that a type referenced by this hash already exists
        self.assert_type_definition(&hash)?;

        let mut insert_new = true;

        for (id, typehash) in &self.mapping_type_id_hash {
            if typehash == &hash {
                insert_new = false;
            }
        }

        if insert_new {
            self.mapping_type_id_hash
                .insert(SchemaObjId::new(), hash.clone());
        }

        for (id, typename) in &self.mapping_type_id_hash {
            if typename == &hash {
                return Ok(id);
            }
        }

        unreachable!()
    }

    /// register a type name (identifier) and generate an ID for it
    fn register_type_name(
        &mut self,
        id: &SchemaObjId,
        top_level_def_name: impl AsRef<str>,
    ) -> anyhow::Result<&SchemaObjId> {
        let top_level_def_name = top_level_def_name.as_ref().to_string();

        let mut insert_new = true;

        for (existing_id, names) in &self.mapping_type_id_name {
            let name_match = names.contains(&top_level_def_name);
            let id_match = existing_id == id;

            // identical already exists
            if name_match && id_match {
                insert_new = false;
            }
            // name already exists but under other ID
            // else if name_match {
            //     Err(anyhow!(
            //         "type name '{}' already exists with different ID",
            //         top_level_def_name
            //     ))?;
            // }
            // name is new but ID is already used for something else
            // else if id_match {
            //     Err(anyhow!(
            //         "type ID '{}' for name '{}' already exists with different name ({})",
            //         id.0,
            //         top_level_def_name,
            //         name
            //     ))?;
            // }
        }

        //  make sure the set is initialized
        if !self.mapping_type_id_name.contains_key(id) {
            self.mapping_type_id_name.insert(id.clone(), HashSet::new());
        }

        // no match found
        // create ID=> name mapping
        if insert_new {
            self.mapping_type_id_name
                .get_mut(id)
                .unwrap()
                .insert(top_level_def_name);
        }

        for key in self.mapping_type_id_name.keys() {
            if key == id {
                return Ok(key);
            }
        }

        unreachable!()
    }
}

/// simple counter for the generation of logical ID's for encountered types
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// identifier for structures that cant be hashed due to recursion errors
#[derive(Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub struct SchemaObjId(u64);

impl SchemaObjId {
    pub fn new() -> Self {
        SchemaObjId(ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

// fake reference to a type that is not yet resolved
// with only an ID existing with a mapping to a type name
pub struct PreliminaryId(TypeRef);

impl PreliminaryId {
    pub fn get_ref(&self) -> TypeRef {
        self.0.clone()
    }

    // todo: make trait
    pub fn schema_object_id(&self) -> &SchemaObjId {
        self.0.schema_object_id()
    }
}

/// map ordered by logical ID
pub type IdMap<T> = HashMap<SchemaObjId, T>;

/// type reference. this was created to force us to retrieve actual
/// type definitions from the centralized collection so we wouldnt be creating conflicting
/// type definitions ad-hoc in different places
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Ref<T>(SchemaObjId, PhantomData<T>);

impl<T> Deref for Ref<T> {
    type Target = SchemaObjId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Ref<T> {
    pub fn schema_object_id(&self) -> &SchemaObjId {
        &self.0
    }
}

impl Ref<Element> {
    pub fn resolve<'a>(&self, schema: &'a Schema) -> &'a Element {
        schema.get_element(self).expect("element resolve issue")
    }
}

impl Ref<SimpleType> {
    pub fn resolve<'a>(&self, schema: &'a Schema) -> &'a SimpleType {
        schema
            .get_simpletype(self)
            .expect("simple type resolve issue")
    }
}

impl Ref<Group> {
    pub fn resolve<'a>(&self, schema: &'a Schema) -> &'a Group {
        schema.get_group(self).expect("group resolve issue")
    }
}

impl Ref<Attribute> {
    pub fn resolve<'a>(&self, schema: &'a Schema) -> &'a Attribute {
        schema.get_attribute(self).expect("attribute resolve issue")
    }
}

/// whether structures as we store them are named in the source schema or not
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedNess {
    Named,
    Anonymous,
}
