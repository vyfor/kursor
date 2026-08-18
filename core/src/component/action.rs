use crate::tree::id::NodeId;

pub enum Action {
    Focus(Option<NodeId>),
    Capture(NodeId),
    Release,
    Invalidate(NodeId),
    Cursor(NodeId, Option<(u16, u16)>),
}
