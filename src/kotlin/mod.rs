use std::{hash::Hash, path::PathBuf};

use anyhow::{Context, Result};
use dashmap::DashMap;
use tree_sitter::{Parser, Tree};
use walkdir::WalkDir;

use self::{class::KotlinClass, import::Import, package::Package};

mod class;
mod import;
mod package;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DataType(pub String);

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Position {
    line: usize,
    char: usize,
}

impl Position {
    pub fn new(line: usize, char: usize) -> Position {
        Position { line, char }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KotlinFile {
    pub package: Package,
    pub imports: Vec<Import>,
    pub classes: Vec<KotlinClass>,
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

pub fn from_path(p: &str) -> Result<DashMap<PathBuf, KotlinFile>> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_kotlin::language()).unwrap();

    let files = DashMap::new();
    for path in WalkDir::new(p)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "kt"))
        .map(|e| e.into_path())
    {
        let content = std::fs::read(&path).unwrap();
        let tree = parser.parse(&content, None).unwrap();
        files.insert(
            path.clone(),
            KotlinFile::new(&tree, &content).context(format!("file: {path:?}"))?,
        );
    }

    Ok(files)
}
