#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
    pub is_entry: bool, // true if this is an actual entry file
}

impl TreeNode {
    pub fn new_entry(name: String) -> Self {
        TreeNode {
            name,
            children: Vec::new(),
            is_expanded: false,
            is_entry: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_new_entry() {
        let name = "20250920".to_string();
        let node = TreeNode::new_entry(name.clone());

        assert_eq!(node.name, name);
        assert!(node.children.is_empty());
        assert!(!node.is_expanded);
        assert!(node.is_entry);
    }
}
