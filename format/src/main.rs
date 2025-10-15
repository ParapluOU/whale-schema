#![feature(associated_type_bounds)]
#![feature(let_chains)]
#![feature(try_blocks)]
#![feature(absolute_path)]

mod ast;
pub(crate) mod cli;
mod compiler;
mod export;
mod formats;
mod import;
mod model;
mod sourced;
pub(crate) mod tests;
mod tools;
mod validation;

use log::LevelFilter;
use pest::Parser;
use pest_derive::Parser;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::fs::File;
use std::path::Path;
use tools::default;

use crate::export::{Exporter, FontoSchemaExporter};
use crate::tools::init_logger;
pub(crate) use {ast::*, cli::*, validation::*};

#[derive(Parser)]
#[grammar = "../schema.pest"] // relative to src
pub struct WHASParser;

fn main() -> anyhow::Result<()> {
    init_logger();

    let args = cli::Args::get();

    if args.fonto {
        let schema = model::Schema::from_file(&args.input)?;

        // save to file
        if let Some(ref dir) = args.output_dir {
            std::fs::create_dir_all(dir)?;
        }

        let output_filename = Path::new(&args.input)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        FontoSchemaExporter::with_version(args.fonto_schema_version()?).export_to_file(
            &schema,
            format!(
                "{}/fonto.schema.json",
                args.output_dir.clone().unwrap_or("./".to_string())
            ),
        )?;
    }

    if args.xsd {
        // todo
    }

    Ok(())
}

#[test]
fn it_compiles() {}
