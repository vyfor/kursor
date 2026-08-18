#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}
