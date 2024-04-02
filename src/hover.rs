use tower_lsp::lsp_types::{
    Hover, HoverContents, MarkedString, MarkupContent, MarkupKind, Position,
};
use tree_sitter::Tree;

use crate::tree;

pub fn get_hover(pos: &Position, tree: &Tree, content: &str) -> Option<Hover> {
    let node = tree::get_node(tree, pos)?;
    let parent = node.parent()?;
    match parent.kind() {
        "call_expression" => {
            let name = node.utf8_text(content.as_bytes()).ok()?;
            let function = tree::get_function(tree, content, name)?;

            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```kotlin\n{function}\n```"),
                }),
                range: None,
            })
        }
        "navigation_expression" => {
            let name = node.utf8_text(content.as_bytes()).ok()?;
            let navigation = tree::get_navigation(tree, content, name)?;
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```kotlin\n{navigation}\n```"),
                }),
                range: None,
            })
        }
        _ => Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(format!(
                "{} is not supported yet",
                parent.kind()
            ))),
            range: None,
        }),
    }
}
