use std::any::{Any, TypeId};

use crate::{
    component::{AnyComponent, blueprint::Blueprint, environment::Environment},
    layout::{rect::Rect, size::Size},
};

pub struct Instance {
    pub component: Box<dyn AnyComponent>,
    pub props: Box<dyn Any>,
    pub children: Vec<Blueprint>,
    pub type_id: TypeId,
    pub rect: Rect,
    pub measured: Size,
    pub env: Environment,
    pub inherited: Environment,
}
