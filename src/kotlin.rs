use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use tracing::warn;
use tree_sitter::{Parser, Tree};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct KotlinFile {
    pub path: PathBuf,
    pub package: String,
    pub imports: Vec<String>,
    pub classes: Vec<String>,
}

impl KotlinFile {
    fn new(path: PathBuf, tree: &Tree, content: String) -> Result<KotlinFile> {
        let package = get_package(tree, &content)?;
        let imports = get_imports(tree, &content)?;
        let classes = get_classes(tree, &content)?;

        Ok(KotlinFile {
            path,
            package,
            imports,
            classes,
        })
    }
}

fn get_package(tree: &Tree, content: &str) -> Result<String> {
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "package" {
            return Ok(node
                .next_sibling()
                .context("no package found")?
                .utf8_text(content.as_bytes())?
                .to_string());
        }

        if cursor.goto_first_child() {
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }

            if !cursor.goto_parent() {
                bail!("no package found");
            }
        }
    }
}

fn get_imports(tree: &Tree, content: &str) -> Result<Vec<String>> {
    let mut imports = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "import" {
            let import = node
                .next_sibling()
                .context("malformed import")?
                .utf8_text(content.as_bytes())
                .context("malformed import")?
                .to_string();

            imports.push(import);
        }

        if cursor.goto_first_child() {
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }

            if !cursor.goto_parent() {
                return Ok(imports);
            }
        }
    }
}

fn get_classes(tree: &Tree, content: &str) -> Result<Vec<String>> {
    let mut classes = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "class_declaration" {
            for child in node.children(&mut cursor) {
                if child.kind() == "type_identifier" {
                    classes.push(
                        child
                            .utf8_text(content.as_bytes())
                            .context("malformed class")?
                            .to_string(),
                    );
                }
            }
        }

        if cursor.goto_first_child() {
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }

            if !cursor.goto_parent() {
                return Ok(classes);
            }
        }
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
            let content = std::fs::read_to_string(&path).unwrap();
            let tree = parser.parse(&content, None).unwrap();
            match KotlinFile::new(path.clone(), &tree, content) {
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
            dbg!(file);
        }
    }
}
