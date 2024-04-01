use tower_lsp::lsp_types::Position;
use tracing::info;
use tree_sitter::{Node, Tree};

pub fn get_node<'a>(tree: &'a Tree, pos: &Position) -> Option<Node<'a>> {
    let mut cursor = tree.walk();
    let mut target_node = None;

    loop {
        let node = cursor.node();
        if node.start_position().row <= pos.line as usize
            && node.start_position().column <= pos.character as usize
            && node.end_position().row >= pos.line as usize
            && node.end_position().column >= pos.character as usize
        {
            target_node = Some(node);
        }

        if cursor.goto_first_child() {
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }

            if !cursor.goto_parent() {
                return target_node;
            }
        }
    }
}

pub fn get_function(tree: &Tree, content: &str, name: &str) -> Option<String> {
    let mut cursor = tree.walk();

    loop {
        let node = cursor.node();
        if node.utf8_text(&content.as_bytes()).unwrap() == name
            && node.parent().unwrap().kind() == "function_declaration"
        {
            let parent = node.parent().unwrap();
            info!("{:?}", parent);
            return Some(
                content
                    .lines()
                    .skip(parent.start_position().row)
                    .take(parent.end_position().row + 1 - parent.start_position().row)
                    .collect::<Vec<&str>>()
                    .join("\n"),
            );
        }

        if cursor.goto_first_child() {
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }

            if !cursor.goto_parent() {
                return None;
            }
        }
    }
}
