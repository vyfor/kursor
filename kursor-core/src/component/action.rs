use std::time::Duration;

use crate::tree::id::NodeId;

pub enum Action {
    Focus(Option<NodeId>),
    Capture(NodeId),
    Release,
    Remeasure(NodeId),
    Repaint(NodeId),
    Relayout(NodeId),
    Cursor(NodeId, Option<(u16, u16)>),
    Wake(NodeId, Duration),
}
