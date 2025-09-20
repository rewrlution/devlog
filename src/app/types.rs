#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMode {
    Preview,
    Edit,
    DatePrompt,
    SavePrompt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Tree,
    Content,
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    Year,
    Month,
    Day { filename: String },
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub label: String,
    pub kind: NodeKind,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}
