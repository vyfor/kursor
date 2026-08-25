pub mod instance;
use std::{
    cmp::{self, Ordering},
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    component::{
        Children, Component, Focus,
        action::Action,
        blueprint::{Blueprint, empty_children},
        context::Cx,
        environment::Environment,
    },
    event::{
        Event, EventResult, Phase,
        mouse::{MouseButton, MouseEvent, MouseKind},
    },
    layout::{
        context::{LayoutCx, MeasureCx},
        offset::Offset,
        rect::Rect,
        size::Size,
    },
    render::{buffer::Buffer, buffer::CellDiff, canvas::Canvas},
    runtime::instance::Instance,
    state::{LocalState, Signal, deps, id::AtomId, memo, queue::dirty_queue, scope, signal},
    tree::{Tree, id::NodeId},
};

struct Scratch {
    dirty_nodes: Vec<NodeId>,
    layout_pending: Vec<(NodeId, Rect, Offset)>,
    child_ids: Vec<NodeId>,
    child_sizes: Vec<Size>,
    rects: Vec<Rect>,
    offsets: Vec<Offset>,
    new_children: Vec<NodeId>,
    old_children: Vec<NodeId>,
    unkeyed: Vec<Option<NodeId>>,
    path: Vec<NodeId>,
    sent_nodes: Vec<NodeId>,
    listeners: Vec<NodeId>,
    paint_dirty: Vec<NodeId>,
    painted: HashSet<NodeId>,
    removed_nodes: HashSet<NodeId>,
    dep_atoms: HashSet<AtomId>,
    dirty_sources: Vec<AtomId>,
    memo_sources: Vec<AtomId>,
    actions: Vec<Action>,
}

impl Scratch {
    fn new() -> Self {
        Self {
            dirty_nodes: Vec::new(),
            layout_pending: Vec::new(),
            child_ids: Vec::new(),
            child_sizes: Vec::new(),
            rects: Vec::new(),
            offsets: Vec::new(),
            new_children: Vec::new(),
            old_children: Vec::new(),
            unkeyed: Vec::new(),
            path: Vec::new(),
            sent_nodes: Vec::new(),
            listeners: Vec::new(),
            paint_dirty: Vec::new(),
            painted: HashSet::new(),
            removed_nodes: HashSet::new(),
            dep_atoms: HashSet::new(),
            dirty_sources: Vec::new(),
            memo_sources: Vec::new(),
            actions: Vec::new(),
        }
    }
}

pub struct Runtime {
    tree: Tree<Instance>,
    deps: HashMap<AtomId, Vec<NodeId>>,
    node_deps: HashMap<NodeId, Vec<AtomId>>,
    rect: Rect,
    front: Buffer,
    back: Buffer,
    update_dirty: HashSet<NodeId>,
    layout_dirty: HashSet<NodeId>,
    paint_dirty: HashSet<NodeId>,
    paint_all: bool,
    focus: Option<NodeId>,
    hovered: Option<NodeId>,
    capture: Option<NodeId>,
    pressed: Option<(NodeId, MouseButton)>,
    cursor: Option<(NodeId, u16, u16)>,
    last_click: Option<(NodeId, MouseButton, Instant)>,
    global_listeners: Vec<NodeId>,
    local_queue: Rc<signal::LocalQueue>,
    scratch: Scratch,
}

impl Runtime {
    pub fn new(size: Size) -> Self {
        Self {
            tree: Tree::new(),
            deps: HashMap::new(),
            node_deps: HashMap::new(),
            rect: Rect::new(0, 0, size.width, size.height),
            front: Buffer::new(size),
            back: Buffer::new(size),
            update_dirty: HashSet::new(),
            layout_dirty: HashSet::new(),
            paint_dirty: HashSet::new(),
            paint_all: true,
            focus: None,
            hovered: None,
            capture: None,
            pressed: None,
            cursor: None,
            last_click: None,
            global_listeners: Vec::new(),
            local_queue: Rc::new(signal::LocalQueue::new()),
            scratch: Scratch::new(),
        }
    }

    pub fn front_buffer(&self) -> &Buffer {
        &self.front
    }

    pub fn cursor(&self) -> Option<(u16, u16)> {
        let (node, x, y) = self.cursor?;
        let (origin, clip) = self.resolve(node)?;
        let x = origin.x.saturating_add(i32::from(x));
        let y = origin.y.saturating_add(i32::from(y));
        (x >= 0 && y >= 0 && clip.contains(x as u16, y as u16)).then_some((x as u16, y as u16))
    }

    pub fn back_buffer(&self) -> &Buffer {
        &self.back
    }

    pub fn mount(&mut self, blueprint: Blueprint) {
        let _signals = signal::enter(&self.local_queue);
        if let Some(root) = self.tree.root() {
            self.drop_node(root);
        }
        self.focus = None;
        self.hovered = None;
        self.capture = None;
        self.pressed = None;
        self.cursor = None;
        self.last_click = None;
        self.paint_all = true;
        self.do_create(blueprint, None);
        self.flush();
    }

    fn local_rect(rect: Rect) -> Rect {
        Rect::new(0, 0, rect.width, rect.height)
    }

    fn translate(parent: Offset, rect: Rect, offset: Offset) -> Offset {
        Offset::new(
            parent
                .x
                .saturating_add(i32::from(rect.x))
                .saturating_add(offset.x),
            parent
                .y
                .saturating_add(i32::from(rect.y))
                .saturating_add(offset.y),
        )
    }

    fn clip(origin: Offset, size: Size, parent_clip: Rect) -> Option<Rect> {
        let left = origin.x.max(i32::from(parent_clip.left()));
        let top = origin.y.max(i32::from(parent_clip.top()));
        let right = origin
            .x
            .saturating_add(i32::from(size.width))
            .min(i32::from(parent_clip.right()));
        let bottom = origin
            .y
            .saturating_add(i32::from(size.height))
            .min(i32::from(parent_clip.bottom()));

        (left < right && top < bottom).then(|| {
            Rect::new(
                left as u16,
                top as u16,
                (right - left) as u16,
                (bottom - top) as u16,
            )
        })
    }

    fn resolve(&self, id: NodeId) -> Option<(Offset, Rect)> {
        let ins = self.tree.get(id)?;
        Some((ins.origin, ins.clip))
    }

    pub fn handle_event(&mut self, event: Event) -> EventResult {
        let _signals = signal::enter(&self.local_queue);
        let event = self.normalize(event);
        let result = match &event {
            Event::FocusIn | Event::FocusOut | Event::WindowFocus(_) => self
                .focus
                .filter(|&id| self.tree.contains(id))
                .map_or(EventResult::Ignored, |id| self.dispatch(id, &event)),
            Event::Key(_) | Event::Paste(_) => {
                let result = self
                    .focus
                    .filter(|&id| self.tree.contains(id))
                    .or_else(|| self.tree.root())
                    .map_or(EventResult::Ignored, |id| self.dispatch(id, &event));
                if result.is_handled() {
                    result
                } else {
                    self.dispatch_global_listener(&event)
                }
            }
            Event::Mouse(mouse) => {
                self.update_hover(mouse);
                let target = self
                    .capture
                    .filter(|&id| self.tree.contains(id))
                    .or_else(|| self.pick(mouse));
                let res = target.map_or(EventResult::Ignored, |id| self.dispatch(id, &event));
                match mouse.kind {
                    MouseKind::Down(button) => {
                        self.pressed = target.map(|id| (id, button));
                    }
                    MouseKind::Up(button) => {
                        let pressed = self.pressed.take();
                        self.capture = None;
                        if let Some((id, pressed_button)) = pressed
                            && pressed_button == button
                            && self.tree.contains(id)
                            && self
                                .resolve(id)
                                .is_some_and(|(_, clip)| clip.contains(mouse.column, mouse.row))
                        {
                            let now = Instant::now();
                            let double =
                                self.last_click.is_some_and(|(last_id, last_button, time)| {
                                    last_id == id
                                        && last_button == button
                                        && now.duration_since(time) <= Duration::from_millis(500)
                                });
                            let click = Event::Mouse(MouseEvent {
                                kind: MouseKind::Click(button),
                                column: mouse.column,
                                row: mouse.row,
                                modifiers: mouse.modifiers,
                            });
                            self.dispatch(id, &click);
                            if double {
                                let dc = Event::Mouse(MouseEvent {
                                    kind: MouseKind::DoubleClick(button),
                                    column: mouse.column,
                                    row: mouse.row,
                                    modifiers: mouse.modifiers,
                                });
                                self.dispatch(id, &dc);
                                self.last_click = None;
                            } else {
                                self.last_click = Some((id, button, now));
                            }
                        }
                    }
                    _ => {}
                }
                res
            }
            Event::Resize(width, height) => {
                self.rect = Rect::new(0, 0, *width, *height);
                self.front = Buffer::new(Size::new(*width, *height));
                self.back = Buffer::new(Size::new(*width, *height));
                self.paint_all = true;
                self.tree.root().map_or(EventResult::Ignored, |root| {
                    self.layout_dirty.insert(root);
                    self.paint_dirty.insert(root);
                    EventResult::Consumed
                })
            }
        };
        self.flush();
        result
    }

    fn normalize(&self, event: Event) -> Event {
        match event {
            Event::Mouse(mouse) => match self.pressed {
                Some((_, button)) if mouse.kind == MouseKind::Move => Event::Mouse(MouseEvent {
                    kind: MouseKind::Drag(button),
                    column: mouse.column,
                    row: mouse.row,
                    modifiers: mouse.modifiers,
                }),
                _ => Event::Mouse(mouse),
            },
            event => event,
        }
    }

    pub fn flush(&mut self) {
        let _signals = signal::enter(&self.local_queue);
        let _scope = scope::enter();
        self.do_update();
        self.do_layout();
    }

    pub fn render(&mut self) -> Vec<CellDiff> {
        let _scope = scope::enter();
        self.flush();
        self.do_paint();
        self.back_buffer().diff(self.front_buffer())
    }

    pub fn commit(&mut self, changes: &[CellDiff]) {
        for change in changes {
            self.front.set(change.x, change.y, change.cell);
        }
    }

    pub fn focus(&self) -> Option<NodeId> {
        self.focus.filter(|&id| self.tree.contains(id))
    }

    pub fn focusables(&self) -> Vec<NodeId> {
        let Some(root) = self.tree.root() else {
            return Vec::new();
        };
        let mut focusables = Vec::new();
        self.tree.visit_subtree(root, |id| {
            if self
                .tree
                .get(id)
                .is_some_and(|node| node.component.focus_any(node.props.as_ref()).focusable)
            {
                focusables.push(id);
            }
        });
        focusables
    }

    pub fn path_to_root(&self, id: NodeId) -> Vec<NodeId> {
        self.tree.path_to_root(id)
    }

    pub fn subtree(&self, id: NodeId) -> Vec<NodeId> {
        self.tree.subtree(id)
    }

    pub fn focus_of(&self, id: NodeId) -> Option<Focus> {
        self.tree
            .get(id)
            .map(|node| node.component.focus_any(node.props.as_ref()))
    }

    fn update_hover(&mut self, mouse: &MouseEvent) {
        let next = self.pick(mouse);
        if self.hovered == next {
            return;
        }
        let previous = self.hovered;
        self.hovered = next;
        if let Some(id) = previous.filter(|&id| self.tree.contains(id)) {
            let event = Event::Mouse(MouseEvent {
                kind: MouseKind::Leave,
                column: mouse.column,
                row: mouse.row,
                modifiers: mouse.modifiers,
            });
            self.dispatch(id, &event);
            self.paint_dirty.insert(id);
        }
        if let Some(id) = next.filter(|&id| self.tree.contains(id)) {
            let event = Event::Mouse(MouseEvent {
                kind: MouseKind::Enter,
                column: mouse.column,
                row: mouse.row,
                modifiers: mouse.modifiers,
            });
            self.dispatch(id, &event);
            self.paint_dirty.insert(id);
        }
    }

    fn pick(&self, mouse: &MouseEvent) -> Option<NodeId> {
        let root = self.tree.root()?;
        self.hit(root, Offset::ZERO, self.rect, mouse.column, mouse.row)
    }

    fn hit(
        &self,
        id: NodeId,
        parent_origin: Offset,
        parent_clip: Rect,
        x: u16,
        y: u16,
    ) -> Option<NodeId> {
        let ins = self.tree.get(id)?;
        let origin = Self::translate(parent_origin, ins.rect, ins.offset);
        let clip = Self::clip(
            origin,
            Size::new(ins.rect.width, ins.rect.height),
            parent_clip,
        )?;
        if !clip.contains(x, y) {
            return None;
        }
        let children = self.tree.children(id);
        for &child in children.iter().rev() {
            if let Some(target) = self.hit(child, origin, clip, x, y) {
                return Some(target);
            }
        }
        Some(id)
    }

    fn dispatch(&mut self, target: NodeId, event: &Event) -> EventResult {
        self.scratch.path.clear();
        let mut current = target;
        while self.tree.contains(current) {
            self.scratch.path.push(current);
            match self.tree.parent(current) {
                Some(p) => current = p,
                None => break,
            }
        }

        self.scratch.sent_nodes.clear();
        let mut res = EventResult::Ignored;
        let path_len = self.scratch.path.len();

        for i in (0..path_len).rev() {
            let node = self.scratch.path[i];
            let cur_res = self.send_event(node, event, Phase::Capture);
            if cur_res.is_handled() {
                self.scratch.sent_nodes.push(node);
                res = cur_res;
            }
            if cur_res.should_stop() {
                break;
            }
        }

        if !res.should_stop() {
            for i in 0..path_len {
                let node = self.scratch.path[i];
                let cur_res = self.send_event(node, event, Phase::Bubble);
                if cur_res.is_handled() {
                    self.scratch.sent_nodes.push(node);
                    res = cur_res;
                }
                if cur_res.should_stop() {
                    break;
                }
            }
        }

        let sent_len = self.scratch.sent_nodes.len();
        for i in 0..sent_len {
            let node = self.scratch.sent_nodes[i];
            if self.tree.contains(node) {
                self.update_dirty.insert(node);
            }
        }

        res
    }

    fn dispatch_global_listener(&mut self, event: &Event) -> EventResult {
        self.scratch.listeners.clear();
        self.scratch
            .listeners
            .extend_from_slice(&self.global_listeners);
        self.scratch
            .listeners
            .sort_by_key(|&id| cmp::Reverse(self.tree.depth(id)));

        let lc = self.scratch.listeners.len();
        for i in 0..lc {
            let id = self.scratch.listeners[i];
            let result = self.send_event(id, event, Phase::Global);
            if result.is_handled() {
                self.update_dirty.insert(id);
                return result;
            }
        }

        EventResult::Ignored
    }

    fn send_event(&mut self, id: NodeId, event: &Event, phase: Phase) -> EventResult {
        if !self.tree.contains(id) {
            return EventResult::Ignored;
        }

        let result = {
            let ins = self.tree.get_mut(id).unwrap();
            self.scratch.actions.clear();
            let mut cx = Cx {
                rect: Self::local_rect(ins.rect),
                node: Some(id),
                actions: Some(&mut self.scratch.actions),
                global_input: None,
                env: ins.env.clone(),
            };
            ins.component
                .event_any(&mut cx, ins.props.as_ref(), event, phase)
        };
        let actions = mem::take(&mut self.scratch.actions);
        self.apply_actions(actions);
        result
    }

    fn apply_actions(&mut self, actions: Vec<Action>) {
        for action in actions {
            match action {
                Action::Focus(next) => self.set_focus(next),
                Action::Capture(node) => {
                    if self.tree.contains(node) {
                        self.capture = Some(node);
                    }
                }
                Action::Release => self.capture = None,
                Action::Invalidate(node) => {
                    if self.tree.contains(node) {
                        self.update_dirty.insert(node);
                    }
                }
                Action::Cursor(node, position) => {
                    if self.tree.contains(node) {
                        self.cursor = position.map(|(x, y)| (node, x, y));
                    }
                }
            }
        }
    }

    pub fn set_focus(&mut self, next: Option<NodeId>) {
        let _signals = signal::enter(&self.local_queue);
        if let Some(id) = next
            && !self.tree.contains(id)
        {
            return;
        }
        if self.focus == next {
            return;
        }
        let previous = self.focus;
        self.focus = next;
        if let Some(id) = previous.filter(|&id| self.tree.contains(id)) {
            self.dispatch(id, &Event::FocusOut);
            self.paint_dirty.insert(id);
        }
        if let Some(id) = next.filter(|&id| self.tree.contains(id)) {
            self.dispatch(id, &Event::FocusIn);
            self.paint_dirty.insert(id);
        }
    }

    fn do_update(&mut self) {
        self.scratch.dirty_sources.clear();
        dirty_queue().drain_into(&mut self.scratch.dirty_sources);
        self.local_queue
            .drain_into(&mut self.scratch.dirty_sources);
        for &atom_id in &self.scratch.dirty_sources {
            if let Some(nodes) = self.deps.get(&atom_id) {
                for &node in nodes {
                    self.update_dirty.insert(node);
                }
            }
        }

        self.scratch.memo_sources.clear();
        self.scratch
            .memo_sources
            .extend_from_slice(&self.scratch.dirty_sources);
        while !self.scratch.memo_sources.is_empty() {
            let changed = memo::refresh_dependents(&self.scratch.memo_sources);
            if changed.is_empty() {
                break;
            }
            for &memo_id in &changed {
                if let Some(nodes) = self.deps.get(&memo_id) {
                    for &node in nodes {
                        self.update_dirty.insert(node);
                    }
                }
            }
            self.scratch.memo_sources.clear();
            self.scratch.memo_sources.extend(changed);
        }

        loop {
            self.scratch.dirty_nodes.clear();
            self.scratch.dirty_nodes.extend(
                self.update_dirty
                    .drain()
                    .filter(|&id| self.tree.contains(id)),
            );
            if self.scratch.dirty_nodes.is_empty() {
                break;
            }

            self.scratch
                .dirty_nodes
                .sort_unstable_by_key(|&id| (self.tree.depth(id), id));
            let dirty = mem::take(&mut self.scratch.dirty_nodes);
            for id in dirty {
                if self.tree.contains(id) {
                    self.do_build(id);
                }
            }
        }
    }

    fn do_build(&mut self, id: NodeId) -> bool {
        let inherited = self
            .tree
            .parent(id)
            .and_then(|parent| self.tree.get(parent).map(|node| node.env.clone()))
            .or_else(|| self.tree.get(id).map(|node| node.inherited.clone()))
            .unwrap_or_default();
        let (dc, cc, child_memo) = self
            .tree
            .get(id)
            .map(|instance| {
                (
                    instance.declared_children.clone(),
                    instance.children.clone(),
                    instance.child_memo,
                )
            })
            .unwrap();
        let (((replacement, next_memo, memo_hit), global_listener), dep_reads) =
            deps::collect(|| {
                let ins = self.tree.get_mut(id).unwrap();
                let mut children = Children::new(cc.clone(), child_memo);
                let mut global_key_listener = false;
                let mut cx = Cx {
                    rect: Self::local_rect(ins.rect),
                    node: Some(id),
                    actions: None,
                    global_input: Some(&mut global_key_listener),
                    env: inherited.clone(),
                };
                ins.component
                    .build_any(&mut cx, ins.props.as_ref(), &mut children);
                ins.inherited = inherited.clone();
                ins.env = cx.env.clone();
                (children.finish(), global_key_listener)
            });

        self.do_deps(id, dep_reads);
        if global_listener {
            if !self.global_listeners.contains(&id) {
                self.global_listeners.push(id);
            }
        } else {
            self.global_listeners.retain(|&node| node != id);
        }
        let next_children: Rc<[Blueprint]> = replacement
            .map(Into::into)
            .unwrap_or_else(|| if memo_hit { cc } else { dc });
        let children_changed = {
            let ins = self.tree.get_mut(id).unwrap();
            ins.child_memo = next_memo;
            let changed = !Rc::ptr_eq(&ins.children, &next_children);
            if changed {
                ins.children = next_children.clone();
            }
            changed
        };
        if children_changed {
            self.sync(id, &next_children);
        }
        self.layout_dirty.insert(id);
        self.paint_dirty.insert(id);

        children_changed
    }

    fn do_deps(&mut self, id: NodeId, mut dep_reads: Vec<AtomId>) {
        dep_reads.sort_unstable();
        dep_reads.dedup();

        match self.node_deps.get(&id) {
            Some(v) if v == &dep_reads => return,
            None if dep_reads.is_empty() => return,
            _ => {}
        }

        let old_deps = self.node_deps.remove(&id).unwrap_or_default();

        let mut i = 0;
        let mut j = 0;
        while i < old_deps.len() && j < dep_reads.len() {
            match old_deps[i].cmp(&dep_reads[j]) {
                Ordering::Less => {
                    let old_atom = old_deps[i];
                    if let Some(nodes) = self.deps.get_mut(&old_atom) {
                        nodes.retain(|&n| n != id);
                        if nodes.is_empty() {
                            self.deps.remove(&old_atom);
                        }
                    }
                    i += 1;
                }
                Ordering::Greater => {
                    self.deps.entry(dep_reads[j]).or_default().push(id);
                    j += 1;
                }
                Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
            }
        }
        while i < old_deps.len() {
            let old_atom = old_deps[i];
            if let Some(nodes) = self.deps.get_mut(&old_atom) {
                nodes.retain(|&n| n != id);
                if nodes.is_empty() {
                    self.deps.remove(&old_atom);
                }
            }
            i += 1;
        }
        while j < dep_reads.len() {
            self.deps.entry(dep_reads[j]).or_default().push(id);
            j += 1;
        }

        if !dep_reads.is_empty() {
            self.node_deps.insert(id, dep_reads);
        }
    }

    fn sync(&mut self, parent: NodeId, blueprints: &[Blueprint]) {
        self.scratch.old_children.clear();
        self.scratch
            .old_children
            .extend_from_slice(self.tree.children(parent));
        let parent_env = self.tree.get(parent).unwrap().env.clone();

        let ordered = self.scratch.old_children.len() == blueprints.len()
            && self
                .scratch
                .old_children
                .iter()
                .zip(blueprints)
                .all(|(&child, blueprint)| {
                    self.tree.get(child).is_some_and(|instance| {
                        instance.type_id == blueprint.type_id && instance.key == blueprint.key
                    })
                });

        if ordered {
            let oldc = mem::take(&mut self.scratch.old_children);
            for (&child, blueprint) in oldc.iter().zip(blueprints) {
                self.sync_existing(child, blueprint, &parent_env);
            }
            self.tree.set_children(parent, oldc);
            return;
        }

        let mut keyed = HashMap::new();
        self.scratch.unkeyed.clear();

        for &child in self.scratch.old_children.iter() {
            let key = self.tree.get(child).unwrap().key.clone();
            if let Some(key) = key {
                keyed.insert(key, child);
            } else {
                self.scratch.unkeyed.push(Some(child));
            }
        }
        self.scratch.new_children.clear();
        self.scratch.new_children.reserve(blueprints.len());
        let mut unkeyed_index = 0;

        for bp in blueprints {
            let m = match &bp.key {
                Some(key) => keyed.remove(key),
                None => {
                    let child = self
                        .scratch
                        .unkeyed
                        .get_mut(unkeyed_index)
                        .and_then(Option::take);
                    unkeyed_index += 1;
                    child
                }
            };
            let child = match m {
                Some(ch)
                    if self
                        .tree
                        .get(ch)
                        .is_some_and(|ins| ins.type_id == bp.type_id) =>
                {
                    self.sync_existing(ch, bp, &parent_env);
                    ch
                }
                Some(ch) => {
                    self.drop_node(ch);
                    self.do_create(bp.clone(), Some(parent))
                }
                None => self.do_create(bp.clone(), Some(parent)),
            };
            self.scratch.new_children.push(child);
        }

        for child in keyed.into_values() {
            if self.tree.contains(child) {
                self.drop_node(child);
            }
        }
        let unkeyed = mem::take(&mut self.scratch.unkeyed);
        for child in unkeyed.into_iter().flatten() {
            if self.tree.contains(child) {
                self.drop_node(child);
            }
        }

        let newc = mem::take(&mut self.scratch.new_children);
        self.tree.set_children(parent, newc);
    }

    fn sync_existing(&mut self, child: NodeId, blueprint: &Blueprint, parent_env: &Environment) {
        let (should_update, env_changed, declared_changed) = {
            let instance = self.tree.get_mut(child).unwrap();
            let should_update = instance
                .component
                .changed_any(instance.props.as_ref(), blueprint.props.as_ref());
            let env_changed = !instance.inherited.same(parent_env);
            let declared_changed = !Rc::ptr_eq(&instance.declared_children, &blueprint.children);

            if should_update {
                instance.props = blueprint.props.clone();
            }
            instance.key = blueprint.key.clone();
            instance.inherited = parent_env.clone();
            if declared_changed {
                instance.declared_children = blueprint.children.clone();
            }

            (should_update, env_changed, declared_changed)
        };

        if should_update || env_changed || declared_changed {
            let instance = self.tree.get_mut(child).unwrap();
            instance.available = None;
            self.update_dirty.insert(child);
        }
    }

    fn drop_node(&mut self, id: NodeId) {
        let parent = self.tree.parent(id);
        let subtree = self.tree.subtree(id);

        if self.focus.is_some_and(|node| subtree.contains(&node)) {
            self.focus = None;
        }
        if self.hovered.is_some_and(|node| subtree.contains(&node)) {
            self.hovered = None;
        }
        if self.capture.is_some_and(|node| subtree.contains(&node)) {
            self.capture = None;
        }
        if self
            .pressed
            .is_some_and(|(node, _)| subtree.contains(&node))
        {
            self.pressed = None;
        }
        if self
            .cursor
            .is_some_and(|(node, _, _)| subtree.contains(&node))
        {
            self.cursor = None;
        }
        if self
            .last_click
            .is_some_and(|(node, _, _)| subtree.contains(&node))
        {
            self.last_click = None;
        }

        self.unsub_subtree(&subtree);
        let st = subtree.clone();
        self.global_listeners
            .retain(|listener| !st.contains(listener));
        for &node in &subtree {
            self.update_dirty.remove(&node);
            self.layout_dirty.remove(&node);
            self.paint_dirty.remove(&node);
        }

        for &node in subtree.iter().rev() {
            let ins = self.tree.get_mut(node).unwrap();
            let mut cx = Cx {
                node: Some(node),
                rect: Self::local_rect(ins.rect),
                actions: None,
                global_input: None,
                env: ins.env.clone(),
            };

            ins.component.drop_any(&mut cx);
        }

        self.tree.remove(id);
        self.paint_all = true;
        if let Some(root) = self.tree.root() {
            self.paint_dirty.insert(root);
        }
        if let Some(parent) = parent {
            self.layout_dirty.insert(parent);
            self.paint_dirty.insert(parent);
        }
        self.scratch.removed_nodes.clear();
    }

    fn unsub_subtree(&mut self, subtree: &[NodeId]) {
        self.scratch.dep_atoms.clear();

        for &node in subtree {
            if let Some(node_atoms) = self.node_deps.remove(&node) {
                self.scratch.dep_atoms.extend(node_atoms);
            }
        }

        for atom in self.scratch.dep_atoms.iter().copied() {
            if let Some(nodes) = self.deps.get_mut(&atom) {
                nodes.retain(|node| !subtree.contains(node));
                if nodes.is_empty() {
                    self.deps.remove(&atom);
                }
            }
        }
    }

    fn do_create(&mut self, bp: Blueprint, parent: Option<NodeId>) -> NodeId {
        let inherited = parent
            .and_then(|parent| self.tree.get(parent).map(|node| node.env.clone()))
            .unwrap_or_default();
        let component = (bp.create)(
            &mut Cx {
                node: None,
                rect: Rect::new(0, 0, 0, 0),
                actions: None,
                global_input: None,
                env: inherited.clone(),
            },
            bp.props.as_ref(),
        );

        let ins = Instance {
            key: bp.key,
            component,
            props: bp.props,
            declared_children: bp.children,
            children: empty_children(),
            child_memo: None,
            type_id: bp.type_id,
            rect: Rect::new(0, 0, 0, 0),
            offset: Offset::ZERO,
            measured: Size::default(),
            available: None,
            env: inherited.clone(),
            inherited,
            origin: Offset::ZERO,
            clip: Rect::new(0, 0, 0, 0),
        };

        let id = match parent {
            Some(p) => self.tree.insert(p, ins),
            None => self.tree.create_root(ins),
        };

        self.update_dirty.insert(id);
        id
    }

    pub fn signal<T: LocalState>(&self, value: T) -> Signal<T> {
        Signal::new_in(self.local_queue.clone(), value)
    }

    fn do_layout(&mut self) {
        loop {
            self.scratch.dirty_nodes.clear();
            self.scratch.dirty_nodes.extend(
                self.layout_dirty
                    .drain()
                    .filter(|&id| self.tree.contains(id)),
            );
            if self.scratch.dirty_nodes.is_empty() {
                break;
            }

            self.scratch
                .dirty_nodes
                .sort_unstable_by_key(|&id| (self.tree.depth(id), id));
            self.layout_dirty
                .extend(self.scratch.dirty_nodes.iter().copied());

            let dirty = mem::take(&mut self.scratch.dirty_nodes);
            for id in dirty {
                if !self.tree.contains(id) || !self.layout_dirty.contains(&id) {
                    continue;
                }

                let rect = if self.tree.root() == Some(id) {
                    self.rect
                } else {
                    self.tree.get(id).map_or(Rect::new(0, 0, 0, 0), |c| c.rect)
                };

                let measured_changed = self.apply_measure(id, Size::new(rect.width, rect.height));
                if measured_changed && let Some(parent) = self.tree.parent(id) {
                    self.layout_dirty.insert(parent);
                }
                let offset = if self.tree.root() == Some(id) {
                    Offset::ZERO
                } else {
                    self.tree.get(id).map_or(Offset::ZERO, |ins| ins.offset)
                };
                self.apply_layout(id, rect, offset);
            }
        }
    }

    fn apply_measure(&mut self, id: NodeId, available: Size) -> bool {
        self.scratch.child_ids.clear();
        self.scratch
            .child_ids
            .extend_from_slice(self.tree.children(id));
        let child_count = self.scratch.child_ids.len();
        let env = self.tree.get(id).unwrap().env.clone();
        let rect = self.tree.get(id).unwrap().rect;

        self.scratch.child_sizes.clear();
        if child_count <= 32 {
            self.scratch
                .child_sizes
                .resize(child_count, Size::default());
            let use_cache = true;
            let mut measures: Vec<Option<Size>> = vec![None; child_count];

            let mut component = {
                let ins = self.tree.get_mut(id).unwrap();
                mem::replace(&mut ins.component, Box::new(Placeholder))
            };
            let props = {
                let ins = self.tree.get_mut(id).unwrap();
                mem::replace(&mut ins.props, Rc::new(()))
            };

            let child_ids = &self.scratch.child_ids;
            let mut measure = |index: usize, child_available: Size| -> Size {
                if index >= child_count {
                    return Size::default();
                }
                if let Some(size) = measures[index]
                    && child_available == available
                {
                    return size;
                }
                let child_id = child_ids[index];
                if !self.tree.contains(child_id) {
                    return Size::default();
                }
                let size = Self::measure_node(&mut self.tree, child_id, child_available);
                measures[index] = Some(size);
                size
            };

            let measured = {
                let mut cx = Cx {
                    node: Some(id),
                    rect,
                    actions: None,
                    global_input: None,
                    env,
                };
                let mut children = MeasureCx::new(&mut measure, child_count, available);
                component.measure_any(&mut cx, props.as_ref(), available, &mut children)
            };

            let changed = {
                let ins = self.tree.get_mut(id).unwrap();
                let changed = ins.measured != measured;
                ins.component = component;
                ins.props = props;
                ins.measured = measured;
                ins.available = Some(available);
                changed
            };
            let _ = use_cache;
            changed
        } else {
            let mut component = {
                let ins = self.tree.get_mut(id).unwrap();
                mem::replace(&mut ins.component, Box::new(Placeholder))
            };
            let props = {
                let ins = self.tree.get_mut(id).unwrap();
                mem::replace(&mut ins.props, Rc::new(()))
            };

            let child_ids = &self.scratch.child_ids;
            let mut measure = |index: usize, child_available: Size| -> Size {
                if index >= child_count {
                    return Size::default();
                }
                let child_id = child_ids[index];
                if !self.tree.contains(child_id) {
                    return Size::default();
                }
                Self::measure_node(&mut self.tree, child_id, child_available)
            };

            let measured = {
                let mut cx = Cx {
                    node: Some(id),
                    rect,
                    actions: None,
                    global_input: None,
                    env,
                };
                let mut children = MeasureCx::new(&mut measure, child_count, available);
                component.measure_any(&mut cx, props.as_ref(), available, &mut children)
            };

            let changed = {
                let ins = self.tree.get_mut(id).unwrap();
                let changed = ins.measured != measured;
                ins.component = component;
                ins.props = props;
                ins.measured = measured;
                ins.available = Some(available);
                changed
            };
            changed
        }
    }

    fn measure_node(tree: &mut Tree<Instance>, id: NodeId, available: Size) -> Size {
        let child_count = tree.children(id).len();

        if child_count == 0
            && let Some(cached) = tree.get(id).and_then(|ins| ins.available)
            && cached == available
            && let Some(ins) = tree.get(id)
        {
            return ins.measured;
        }

        let env = tree.get(id).unwrap().env.clone();
        let rect = tree.get(id).unwrap().rect;

        let mut measures = (child_count <= 32).then(|| vec![None; child_count]);

        let mut component = {
            let ins = tree.get_mut(id).unwrap();
            mem::replace(&mut ins.component, Box::new(Placeholder))
        };
        let props = {
            let ins = tree.get_mut(id).unwrap();
            mem::replace(&mut ins.props, Rc::new(()))
        };

        let mut child_ids = Vec::new();
        child_ids.extend_from_slice(tree.children(id));
        let mut measure = |index: usize, child_available: Size| -> Size {
            if index >= child_count {
                return Size::default();
            }
            if let Some(measures) = measures.as_ref()
                && let Some(size) = measures[index]
                && child_available == available
            {
                return size;
            }
            let child_id = child_ids[index];
            if !tree.contains(child_id) {
                return Size::default();
            }
            let size = Self::measure_node(tree, child_id, child_available);
            if let Some(measures) = measures.as_mut() {
                measures[index] = Some(size);
            }
            size
        };

        let measured = {
            let mut cx = Cx {
                node: Some(id),
                rect,
                actions: None,
                global_input: None,
                env,
            };
            let mut children = MeasureCx::new(&mut measure, child_count, available);
            component.measure_any(&mut cx, props.as_ref(), available, &mut children)
        };

        {
            let ins = tree.get_mut(id).unwrap();
            ins.component = component;
            ins.props = props;
            ins.measured = measured;
            ins.available = Some(available);
        }
        measured
    }

    fn apply_layout(&mut self, id: NodeId, rect: Rect, offset: Offset) {
        self.scratch.layout_pending.clear();
        self.scratch.layout_pending.push((id, rect, offset));

        while let Some((id, rect, offset)) = self.scratch.layout_pending.pop() {
            let is_dirty = self.layout_dirty.remove(&id);
            let (old_rect, old_offset) = self
                .tree
                .get(id)
                .map_or((Rect::default(), Offset::ZERO), |ins| {
                    (ins.rect, ins.offset)
                });
            let size_changed = old_rect.width != rect.width || old_rect.height != rect.height;

            if old_rect != rect || old_offset != offset {
                self.paint_all = true;
            }

            if !is_dirty && !size_changed {
                let (parent_origin, parent_clip) = match self.tree.parent(id) {
                    Some(p) => match self.tree.get(p) {
                        Some(p_ins) => (p_ins.origin, p_ins.clip),
                        None => (Offset::ZERO, self.rect),
                    },
                    None => (Offset::ZERO, self.rect),
                };
                let origin = Self::translate(parent_origin, rect, offset);
                let clip = Self::clip(origin, Size::new(rect.width, rect.height), parent_clip)
                    .unwrap_or(Rect::new(0, 0, 0, 0));
                let ins = self.tree.get_mut(id).unwrap();
                ins.rect = rect;
                ins.offset = offset;
                ins.origin = origin;
                ins.clip = clip;
                continue;
            }

            self.scratch.child_ids.clear();
            self.scratch
                .child_ids
                .extend_from_slice(self.tree.children(id));
            let child_count = self.scratch.child_ids.len();

            self.scratch.child_sizes.clear();
            self.scratch
                .child_sizes
                .resize(child_count, Size::default());
            for (i, &child) in self.scratch.child_ids.iter().enumerate() {
                if let Some(node) = self.tree.get(child) {
                    self.scratch.child_sizes[i] = node.measured;
                }
            }

            self.scratch.rects.clear();
            self.scratch.rects.resize(child_count, Rect::default());
            self.scratch.offsets.clear();
            self.scratch.offsets.resize(child_count, Offset::ZERO);

            {
                let ins = self.tree.get_mut(id).unwrap();
                let local = Self::local_rect(rect);
                let mut cx = Cx {
                    node: Some(id),
                    rect: local,
                    actions: None,
                    global_input: None,
                    env: ins.env.clone(),
                };
                let mut children = LayoutCx::new(
                    &self.scratch.child_sizes,
                    &mut self.scratch.rects,
                    &mut self.scratch.offsets,
                );
                ins.component
                    .layout_any(&mut cx, ins.props.as_ref(), local, &mut children);
            }

            let parent_origin = self
                .tree
                .parent(id)
                .and_then(|p| self.tree.get(p))
                .map_or(Offset::ZERO, |p| p.origin);
            let parent_clip = self
                .tree
                .parent(id)
                .and_then(|p| self.tree.get(p))
                .map_or(self.rect, |p| p.clip);
            let origin = Self::translate(parent_origin, rect, offset);
            let clip = Self::clip(origin, Size::new(rect.width, rect.height), parent_clip)
                .unwrap_or(Rect::new(0, 0, 0, 0));

            {
                let ins = self.tree.get_mut(id).unwrap();
                ins.rect = rect;
                ins.offset = offset;
                ins.origin = origin;
                ins.clip = clip;
            }

            let child_ids = mem::take(&mut self.scratch.child_ids);
            let rects = mem::take(&mut self.scratch.rects);
            let offsets = mem::take(&mut self.scratch.offsets);
            for ((&child, &child_rect), &child_offset) in
                child_ids.iter().zip(rects.iter()).zip(offsets.iter()).rev()
            {
                if self.tree.contains(child) {
                    self.scratch
                        .layout_pending
                        .push((child, child_rect, child_offset));
                }
            }
        }
    }

    fn do_paint(&mut self) {
        if self.paint_all {
            self.paint_all = false;
            self.paint_dirty.clear();
            self.back.clear();
            if let Some(root) = self.tree.root() {
                self.apply_paint(root, Offset::ZERO, self.rect);
            }
            return;
        }
        loop {
            self.scratch.paint_dirty.clear();
            self.scratch.paint_dirty.extend(
                self.paint_dirty
                    .drain()
                    .filter(|&id| self.tree.contains(id)),
            );
            if self.scratch.paint_dirty.is_empty() {
                break;
            }

            self.scratch
                .paint_dirty
                .sort_unstable_by_key(|&id| (self.tree.depth(id), id));
            self.scratch.painted.clear();

            let dirty_count = self.scratch.paint_dirty.len();
            for i in 0..dirty_count {
                let id = self.scratch.paint_dirty[i];
                if !self.tree.contains(id) || self.scratch.painted.contains(&id) {
                    continue;
                }

                self.tree.visit_subtree(id, |node| {
                    self.scratch.painted.insert(node);
                });

                if self.tree.root() == Some(id) {
                    self.back.clear();
                }
                self.apply_stacked_paint(id);
            }
            self.scratch.painted.clear();
        }
    }

    fn apply_stacked_paint(&mut self, id: NodeId) {
        let (origin, clip) = match self.resolve(id) {
            Some(resolved) => resolved,
            None => return,
        };
        self.apply_paint(id, origin, clip);

        let mut current = id;
        loop {
            let parent = match self.tree.parent(current) {
                Some(p) => p,
                None => break,
            };
            let siblings = self.tree.children(parent);
            let index = match siblings.iter().position(|&s| s == current) {
                Some(i) => i,
                None => break,
            };
            let siblings_after: Vec<NodeId> = siblings[index + 1..].to_vec();
            for sibling in siblings_after {
                if let Some((sibling_origin, sibling_clip)) = self.resolve(sibling) {
                    self.apply_paint(sibling, sibling_origin, sibling_clip);
                }
            }
            current = parent;
        }
    }

    fn apply_paint(&mut self, id: NodeId, origin: Offset, clip: Rect) {
        {
            let ins = self.tree.get(id).unwrap();
            let mut cx = Cx {
                node: Some(id),
                rect: Self::local_rect(ins.rect),
                actions: None,
                global_input: None,
                env: ins.env.clone(),
            };
            let mut canvas = Canvas::new(&mut self.back, clip, origin);
            ins.component
                .paint_any(&mut cx, ins.props.as_ref(), &mut canvas);
        }

        let children: Vec<NodeId> = self.tree.children(id).to_vec();
        for child in children {
            if !self.tree.contains(child) {
                continue;
            }
            let (child_rect, child_offset, child_w, child_h) = {
                let child_ins = self.tree.get(child).unwrap();
                (
                    child_ins.rect,
                    child_ins.offset,
                    child_ins.rect.width,
                    child_ins.rect.height,
                )
            };
            let child_origin = Self::translate(origin, child_rect, child_offset);
            let Some(child_clip) = Self::clip(child_origin, Size::new(child_w, child_h), clip)
            else {
                continue;
            };
            self.apply_paint(child, child_origin, child_clip);
        }
    }
}

struct Placeholder;

impl Component for Placeholder {
    type Props = ();
    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }
}
