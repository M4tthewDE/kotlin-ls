use anyhow::{anyhow, Context, Result};
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use tree_sitter::Tree;

use crate::tree;

pub fn get_hover(pos: &Position, tree: &Tree, content: &str) -> Result<Option<Hover>> {
    let node = tree::get_node(tree, pos).with_context(|| format!("node at {pos:?} not found"))?;
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
        _ => Err(anyhow!("{} is not supported yet", parent.kind())),
    }
}
