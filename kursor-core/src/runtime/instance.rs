use std::{
    any::{Any, TypeId},
    rc::Rc,
};

use crate::{
    component::{AnyComponent, blueprint::Blueprint, environment::Environment, key::Key},
    layout::{offset::Offset, rect::Rect, size::Size},
};

pub struct Instance {
    pub key: Option<Key>,
    pub component: Box<dyn AnyComponent>,
    pub props: Rc<dyn Any>,
    pub declared_children: Rc<[Blueprint]>,
    pub children: Rc<[Blueprint]>,
    pub type_id: TypeId,
    pub rect: Rect,
    pub offset: Offset,
    pub measured: Size,
    pub available: Option<Size>,
    pub is_measure_valid: bool,
    pub is_layout_valid: bool,
    pub env: Environment,
    pub inherited: Environment,
    pub origin: Offset,
    pub clip: Rect,
}
