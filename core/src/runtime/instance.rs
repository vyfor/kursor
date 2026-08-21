use std::{
    any::{Any, TypeId},
    rc::Rc,
};

use crate::{
    component::{AnyComponent, blueprint::Blueprint, environment::Environment},
    layout::{offset::Offset, rect::Rect, size::Size},
};

pub struct Instance {
    pub component: Box<dyn AnyComponent>,
    pub props: Rc<dyn Any>,
    pub children: Rc<[Blueprint]>,
    pub type_id: TypeId,
    pub rect: Rect,
    pub offset: Offset,
    pub measured: Size,
    pub available: Option<Size>,
    pub env: Environment,
    pub inherited: Environment,
}
