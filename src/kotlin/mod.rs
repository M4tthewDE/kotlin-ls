use std::hash::Hash;

use anyhow::Result;
use tower_lsp::lsp_types::{Hover, Position};
use tree_sitter::Tree;

use self::{class::KotlinClass, import::Import, package::Package};

mod class;
mod import;
mod package;

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

    pub fn hover_element(&self, pos: &Position) -> Option<Hover> {
        for class in &self.classes {
            let elem = class.get_elem(pos);
            if elem.is_some() {
                return elem;
            }
        }

        None
    }
}
