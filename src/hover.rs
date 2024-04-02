use anyhow::{anyhow, bail, Context, Result};
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use tracing::info;
use tree_sitter::Tree;

use crate::{tree, Backend};

impl Backend {
    pub fn get_hover(&self, pos: &Position, tree: &Tree, content: &str) -> Result<Option<Hover>> {
        let node =
            tree::get_node(tree, pos).with_context(|| format!("node at {pos:?} not found"))?;
        let parent = node.parent().context("node has no parent")?;
        match parent.kind() {
            "call_expression" => {
                let name = node.utf8_text(content.as_bytes())?;
                let function = tree::get_function(tree, content, name)
                    .with_context(|| format!("function {name} not found"))?;

                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```kotlin\n{function}\n```"),
                    }),
                    range: None,
                }))
            }
            "navigation_expression" => {
                let name = node.utf8_text(content.as_bytes())?;
                let navigation = tree::get_navigation(tree, content, name)
                    .with_context(|| format!("navigation expression {name} not found"))?;
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("```kotlin\n{navigation}\n```"),
                    }),
                    range: None,
                }))
            }
            "navigation_suffix" => {
                let name = node.utf8_text(content.as_bytes())?;
                let target = node
                    .parent()
                    .and_then(|p| p.prev_sibling())
                    .context("no navigation expression found")?;
                let nav_name = target.utf8_text(content.as_bytes())?;
                let nav_type = tree::get_navigation_type(tree, content, nav_name)
                    .context("no type for navigation expression found")?;

                for entry in self.trees.iter() {
                    if let Some(stem) = entry.key().file_stem() {
                        if stem == nav_type.as_str() {
                            // assuming that it's a function for now
                            // will have to be expanded for instance variables in the future
                            info!("file: {:?}", entry.key());

                            return tree::get_function(&entry.0, &entry.1, name)
                                .map(|s| {
                                    Some(Hover {
                                        contents: HoverContents::Markup(MarkupContent {
                                            kind: MarkupKind::Markdown,
                                            value: format!("```kotlin\n{s}\n```"),
                                        }),
                                        range: None,
                                    })
                                })
                                .with_context(|| {
                                    format!(
                                        "no function with name {name} found in {:?}",
                                        entry.key()
                                    )
                                });
                        }
                    }
                }

                bail!("No file found for {nav_type}");
            }

            _ => Err(anyhow!("{} is not supported", parent.kind())),
        }
    }
}
