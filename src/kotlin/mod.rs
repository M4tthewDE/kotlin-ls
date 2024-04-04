use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::warn;
use tree_sitter::{Parser, Tree};
use walkdir::WalkDir;

use self::{class::KotlinClass, import::Import, package::Package};

mod class;
mod import;
mod package;

#[derive(Debug)]
pub struct KotlinFile {
    pub path: PathBuf,
    pub package: Package,
    pub imports: Vec<Import>,
    pub classes: Vec<KotlinClass>,
}

impl KotlinFile {
    fn new(path: PathBuf, tree: &Tree, content: &[u8]) -> Result<KotlinFile> {
        let package = package::get_package(tree, content)?;
        let imports = import::get_imports(tree, content)?;
        let classes = class::get_classes(tree, content)?;

        Ok(KotlinFile {
            path,
            package,
            imports,
            classes,
        })
    }
}

pub struct KotlinProject {
    pub files: Vec<KotlinFile>,
}

impl KotlinProject {
    pub fn new(p: &Path) -> Result<KotlinProject> {
        let mut files = Vec::new();
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_kotlin::language()).unwrap();

        for path in WalkDir::new(p)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "kt"))
            .map(|e| e.into_path())
        {
            let content = std::fs::read(&path).unwrap();
            let tree = parser.parse(&content, None).unwrap();
            match KotlinFile::new(path.clone(), &tree, &content) {
                Ok(f) => files.push(f),
                Err(err) => warn!("Failed to parse {:?}: {}", path, err),
            }
        }

        Ok(KotlinProject { files })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::kotlin::KotlinProject;

    #[test]
    fn test_new() {
        tracing_subscriber::fmt().init();
        let p = Path::new("/home/matti/Programming/contributing/DankChat");

        let project = KotlinProject::new(&p).unwrap();
        for file in project.files {
            if file
                .path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("DankChatApplication.kt")
            {
                dbg!(file);
            }
        }
    }
}
