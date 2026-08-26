use crate::tree::id::NodeId;

pub(crate) struct Node<T> {
    pub(crate) generation: u32,
    pub(crate) data: T,
    pub(crate) parent: Option<NodeId>,
    pub(crate) depth: usize,
    pub(crate) children: Vec<NodeId>,
}
