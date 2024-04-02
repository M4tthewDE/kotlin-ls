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
        if node.utf8_text(content.as_bytes()).unwrap() == name
            && node.parent().unwrap().kind() == "function_declaration"
        {
            // FIXME: this probably breaks for functions with more than one modifier
            // this actually breaks for a lot of cases
            // for example UserDisplayViewmodel.kt:59
            let modifier_node = node.prev_sibling().unwrap().prev_sibling().unwrap();
            let modifier_text = modifier_node.utf8_text(content.as_bytes()).unwrap();

            let params_node = node.next_sibling().unwrap();
            let params_text = params_node.utf8_text(content.as_bytes()).unwrap();

            let return_node = params_node.next_sibling().unwrap().next_sibling().unwrap();
            let return_text = return_node.utf8_text(content.as_bytes()).unwrap();
            info!("{modifier_text} fun {name}{params_text}: {return_text}");

            return Some(format!(
                "{modifier_text} fun {name}{params_text}: {return_text}"
            ));
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
