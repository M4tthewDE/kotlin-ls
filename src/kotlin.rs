use tree_sitter::Tree;

pub struct KotlinFile {
    package: String,
}

impl From<Tree> for KotlinFile {
    fn from(tree: Tree) -> Self {
        todo!()
    }
}
