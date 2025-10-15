use crate::formats::FontoSchemaCompilerVersion;
use crate::formats::FontoVersion;
use clap::Parser;
use log::warn;
use tap::Tap;

/// Whale Schema Compiler
///
/// Compile a *.whas schema file to:
///     - Fonto Schema .json
///     - XML Schema XSD
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct Args {
    /// path to entrypoint WHAS schema
    pub input: String,

    /// compile to a Fonto schema
    #[arg(short, long, default_value_t = true)]
    pub fonto: bool,

    /// by default we compile for the toolset of v8.8 but with this flag we
    /// can specify it further. It's important because it will change the version numbering
    /// int he generated JSON schema. When the Fonto instance is incompatible with it,
    /// documents will not be openeable
    #[arg(long)]
    pub fonto_version: Option<String>,

    ///compile to an XSD schema
    #[arg(short, long, default_value_t = true)]
    pub xsd: bool,

    /// output directory to export generated assets in
    #[arg(short, long = "output-dir")]
    pub output_dir: Option<String>,
}

impl Args {
    pub fn get() -> Self {
        Self::parse()
    }

    pub fn fonto_schema_version(&self) -> anyhow::Result<FontoSchemaCompilerVersion> {
        Ok(if let Some(v) = &self.fonto_version {
            FontoVersion::try_from_str(v)?.min_schema_compiler_version()
        } else {
            warn!("assuming default Fonto schema version");
            (FontoSchemaCompilerVersion::default())
        })
    }
}
