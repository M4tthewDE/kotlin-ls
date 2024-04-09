use std::{hash::Hash, path::PathBuf};

use anyhow::{Context, Result};
use dashmap::DashMap;
use tree_sitter::{Parser, Tree};
use walkdir::WalkDir;

use self::{class::Class, import::Import, package::Package};

mod argument;
mod assignment;
mod class;
mod constructor_invocation;
mod delegation;
mod expression;
mod function;
mod getter;
mod import;
mod label;
mod lambda;
mod literal;
mod modifier;
mod object;
mod package;
mod property;
mod statement;
mod types;
mod variable_declaration;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KotlinFile {
    pub package: Package,
    pub imports: Vec<Import>,
    pub classes: Vec<Class>,
}

impl KotlinFile {
    pub fn new(tree: &Tree, content: &[u8]) -> Result<KotlinFile> {
        let package = package::get_package(tree, content)?;
        let imports = import::get_imports(tree, content)?;
        let classes = class::get_classes(tree, content)?;

        Ok(KotlinFile {
            package,
            imports,
            classes,
        })
    }
}

pub fn from_path(p: &str) -> Result<DashMap<PathBuf, Result<KotlinFile>>> {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_kotlin::language())
        .context("failed to create kotlin parser")?;

    let files = DashMap::new();
    for path in WalkDir::new(p)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "kt"))
        .map(|e| e.into_path())
    {
        let content = std::fs::read(&path)?;
        let tree = parser
            .parse(&content, None)
            .context(format!("failed to parse {path:?}"))?;
        files.insert(
            path.clone(),
            KotlinFile::new(&tree, &content).context(format!("failed to analyze {path:?}")),
        );
    }

    Ok(files)
}
