use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use tracing::warn;
use tree_sitter::{Node, Parser, Tree};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum ClassModifier {
    Class(String),
    Visibility(String),
    Annotation(String),
    Inheritance(String),
}

#[derive(Debug)]
pub enum PropertyModifier {
    Annotation(String),
    Member(String),
    Visibility(String),
    Inheritance(String),
}

#[derive(Debug)]
pub struct Property {
    pub modifiers: Vec<PropertyModifier>,
    pub name: String,
    pub type_identifier: Option<String>,
}

#[derive(Debug)]
pub struct ClassBody {
    pub properties: Vec<Property>,
}

#[derive(Debug)]
pub struct KotlinClass {
    pub name: String,
    pub modifiers: Vec<ClassModifier>,
    pub supertypes: Vec<String>,
    pub body: ClassBody,
}

#[derive(Debug)]
pub struct KotlinFile {
    pub path: PathBuf,
    pub package: String,
    pub imports: Vec<String>,
    pub classes: Vec<KotlinClass>,
}

impl KotlinFile {
    fn new(path: PathBuf, tree: &Tree, content: &[u8]) -> Result<KotlinFile> {
        let package = get_package(tree, content)?;
        let imports = get_imports(tree, content)?;
        let classes = get_classes(tree, content)?;

        Ok(KotlinFile {
            path,
            package,
            imports,
            classes,
        })
    }
}

fn get_package(tree: &Tree, content: &[u8]) -> Result<String> {
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "package" {
            return Ok(node
                .next_sibling()
                .context("no package found")?
                .utf8_text(content)?
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

fn get_imports(tree: &Tree, content: &[u8]) -> Result<Vec<String>> {
    let mut imports = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "import" {
            let import = node
                .next_sibling()
                .context("malformed import")?
                .utf8_text(content)
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

fn get_classes(tree: &Tree, content: &[u8]) -> Result<Vec<KotlinClass>> {
    let mut classes = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "class_declaration" {
            let name = get_class_name(&node, content)?;
            let modifiers = get_class_modifiers(&node, content)?;
            let supertypes = get_supertypes(&node, content)?;
            let body = get_class_body(&node, content)?;
            classes.push(KotlinClass {
                name,
                modifiers,
                supertypes,
                body,
            });
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

fn get_class_name(node: &Node, content: &[u8]) -> Result<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "type_identifier" {
            return Ok(child
                .utf8_text(content)
                .context("malformed class")?
                .to_string());
        }
    }

    bail!("no class name found");
}

fn get_class_modifiers(node: &Node, content: &[u8]) -> Result<Vec<ClassModifier>> {
    let mut modifiers: Vec<ClassModifier> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor.clone()) {
        if child.kind() == "modifiers" {
            for child in child.children(&mut cursor) {
                match child.kind() {
                    "visibility_modifier" => modifiers.push(ClassModifier::Visibility(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "class_modifier" => {
                        modifiers.push(ClassModifier::Class(child.utf8_text(content)?.to_string()))
                    }
                    "annotation" => modifiers.push(ClassModifier::Annotation(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "inheritance_modifier" => modifiers.push(ClassModifier::Inheritance(
                        child.utf8_text(content)?.to_string(),
                    )),
                    _ => bail!("unknown modifier {}", child.kind()),
                }
            }
        }
    }

    Ok(modifiers)
}

fn get_supertypes(node: &Node, content: &[u8]) -> Result<Vec<String>> {
    let mut supertypes: Vec<String> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "delegation_specifier" {
            supertypes.push(child.utf8_text(content)?.to_string());
        }
    }

    Ok(supertypes)
}

fn get_class_body(node: &Node, content: &[u8]) -> Result<ClassBody> {
    let mut properties: Vec<Property> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor.clone()) {
        if child.kind() == "class_body" {
            for child in child.children(&mut cursor) {
                if child.kind() == "property_declaration" {
                    properties.push(get_property(&child, content)?);
                }
            }
        }
    }

    Ok(ClassBody { properties })
}

fn get_property(node: &Node, content: &[u8]) -> Result<Property> {
    let mut modifiers: Vec<PropertyModifier> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor.clone()) {
        if child.kind() == "modifiers" {
            for child in child.children(&mut cursor) {
                match child.kind() {
                    "annotation" => modifiers.push(PropertyModifier::Annotation(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "member_modifier" => modifiers.push(PropertyModifier::Member(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "visibility_modifier" => modifiers.push(PropertyModifier::Visibility(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "inheritance_modifier" => modifiers.push(PropertyModifier::Inheritance(
                        child.utf8_text(content)?.to_string(),
                    )),
                    _ => bail!("unknown modifier {}", child.kind()),
                }
            }
        }

        if child.kind() == "variable_declaration" {
            let name = child
                .child(0)
                .context("no name found for variable declaration")?
                .utf8_text(content)?
                .to_string();
            let type_identifier = if let Some(type_node) = child.child(2) {
                Some(type_node.utf8_text(content)?.to_string())
            } else {
                None
            };

            return Ok(Property {
                modifiers,
                name,
                type_identifier,
            });
        }
    }

    bail!("no property found");
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
