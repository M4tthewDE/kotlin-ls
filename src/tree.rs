use tower_lsp::lsp_types::Position;
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
            let mut result = String::new();

            let modifier_node = node.prev_sibling().unwrap().prev_sibling().unwrap();
            result.push_str(modifier_node.utf8_text(content.as_bytes()).unwrap());
            result.push_str(" fun ");
            result.push_str(name);

            let params_node = node.next_sibling().unwrap();
            result.push_str(params_node.utf8_text(content.as_bytes()).unwrap());

            if let Some(colon_node) = params_node.next_sibling() {
                if colon_node.kind() == ":" {
                    result.push_str(": ");
                    result.push_str(
                        colon_node
                            .next_sibling()
                            .unwrap()
                            .utf8_text(content.as_bytes())
                            .unwrap(),
                    );
                }
            }

            return Some(result);
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

pub fn get_navigation(tree: &Tree, content: &str, name: &str) -> Option<String> {
    let mut cursor = tree.walk();

    loop {
        let node = cursor.node();
        if node.utf8_text(content.as_bytes()).unwrap() == name {
            match node.kind() {
                "variable_declaration" => {
                    let mut result = String::new();

                    let modifier = node.prev_sibling().unwrap().prev_sibling().unwrap();
                    result.push_str(modifier.utf8_text(content.as_bytes()).unwrap());
                    result.push(' ');

                    let val = node.prev_sibling().unwrap();
                    result.push_str(val.utf8_text(content.as_bytes()).unwrap());
                    result.push(' ');
                    result.push_str(name);
                    result.push_str(" = ");
                    result.push_str(
                        node.next_sibling()
                            .unwrap()
                            .next_sibling()
                            .unwrap()
                            .utf8_text(content.as_bytes())
                            .unwrap(),
                    );

                    return Some(result);
                }
                _ => {
                    if node.parent().unwrap().kind() == "class_parameter" {
                        let mut result = String::new();
                        let modifier = node.prev_sibling().unwrap().prev_sibling().unwrap();
                        result.push_str(modifier.utf8_text(content.as_bytes()).unwrap());
                        result.push(' ');

                        let val = node.prev_sibling().unwrap();
                        result.push_str(val.utf8_text(content.as_bytes()).unwrap());
                        result.push(' ');
                        result.push_str(name);
                        result.push_str(": ");
                        result.push_str(
                            node.next_sibling()
                                .unwrap()
                                .next_sibling()
                                .unwrap()
                                .utf8_text(content.as_bytes())
                                .unwrap(),
                        );
                        return Some(result);
                    }
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
                return None;
            }
        }
    }
}
