pub mod instance;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    component::{Children, Component, Focus, action::Action, blueprint::Blueprint, context::Cx},
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
    state::{deps, id::AtomId, queue::dirty_queue, scope},
    tree::{Tree, id::NodeId},
};

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
        let mut origin = Offset::ZERO;
        let mut clip = self.rect;

        for node in self.tree.path_to_root(id).into_iter().rev() {
            let ins = self.tree.get(node)?;
            origin = Self::translate(origin, ins.rect, ins.offset);
            clip = Self::clip(origin, Size::new(ins.rect.width, ins.rect.height), clip)?;
        }

        Some((origin, clip))
    }

    pub fn handle_event(&mut self, event: Event) -> EventResult {
        let event = self.normalize(event);
        let result = match &event {
            Event::FocusIn | Event::FocusOut => self
                .focus
                .filter(|&id| self.tree.contains(id))
                .map_or(EventResult::Ignored, |id| self.dispatch(id, &event)),
            Event::Key(_) | Event::Paste(_) => self
                .focus
                .filter(|&id| self.tree.contains(id))
                .or_else(|| self.tree.root())
                .map_or(EventResult::Ignored, |id| self.dispatch(id, &event)),
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
        for &child in self.tree.children(id).iter().rev() {
            if let Some(target) = self.hit(child, origin, clip, x, y) {
                return Some(target);
            }
        }
        Some(id)
    }

    fn dispatch(&mut self, target: NodeId, event: &Event) -> EventResult {
        let path = self.tree.path_to_root(target);
        let mut sent = Vec::with_capacity(path.len());
        let mut res = EventResult::Ignored;

        for &node in path.iter().rev() {
            let current = self.send_event(node, event, Phase::Ascending);
            if current.is_handled() {
                sent.push(node);
                res = current;
            }
            if current.should_stop() {
                break;
            }
        }

        if !res.should_stop() {
            for &node in &path {
                let current = self.send_event(node, event, Phase::Descending);
                if current.is_handled() {
                    sent.push(node);
                    res = current;
                }
                if current.should_stop() {
                    break;
                }
            }
        }

        for node in sent {
            if self.tree.contains(node) {
                self.update_dirty.insert(node);
            }
        }

        res
    }

    fn send_event(&mut self, id: NodeId, event: &Event, phase: Phase) -> EventResult {
        if !self.tree.contains(id) {
            return EventResult::Ignored;
        }

        let (result, actions) = {
            let ins = self.tree.get_mut(id).unwrap();
            let mut pending = Vec::new();
            let mut cx = Cx {
                rect: Self::local_rect(ins.rect),
                node: Some(id),
                actions: Some(&mut pending),
                env: ins.env.clone(),
            };
            let result = ins
                .component
                .event_any(&mut cx, ins.props.as_ref(), event, phase);
            (result, pending)
        };
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
        for atom_id in dirty_queue().drain() {
            if let Some(nodes) = self.deps.get(&atom_id) {
                for &node in nodes {
                    self.update_dirty.insert(node);
                }
            }
        }

        loop {
            self.update_dirty.retain(|&n| self.tree.contains(n));

            let Some(id) = self
                .update_dirty
                .iter()
                .copied()
                .filter(|&id| self.tree.contains(id))
                .min_by_key(|&id| self.tree.depth(id))
            else {
                break;
            };

            self.update_dirty.remove(&id);
            self.do_build(id);
        }
    }

    fn do_build(&mut self, id: NodeId) -> bool {
        let inherited = self
            .tree
            .parent(id)
            .and_then(|parent| self.tree.get(parent).map(|node| node.env.clone()))
            .or_else(|| self.tree.get(id).map(|node| node.inherited.clone()))
            .unwrap_or_default();
        let (replacement, dep_reads) = deps::collect(|| {
            let ins = self.tree.get_mut(id).unwrap();
            let mut children = Children::new(ins.children.clone());
            let mut cx = Cx {
                rect: Self::local_rect(ins.rect),
                node: Some(id),
                actions: None,
                env: inherited.clone(),
            };
            ins.component
                .build_any(&mut cx, ins.props.as_ref(), &mut children);
            ins.inherited = inherited.clone();
            ins.env = cx.env.clone();
            children.finish()
        });

        self.do_deps(id, dep_reads);
        let replaced = replacement.is_some();
        if let Some(blueprints) = replacement {
            let blueprints: Rc<[Blueprint]> = blueprints.into();
            self.tree.get_mut(id).unwrap().children = blueprints.clone();
            self.sync(id, &blueprints);
        }
        self.layout_dirty.insert(id);
        self.paint_dirty.insert(id);
        replaced
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
                    let old_atom = &old_deps[i];
                    if let Some(nodes) = self.deps.get_mut(old_atom) {
                        nodes.retain(|&n| n != id);
                        if nodes.is_empty() {
                            self.deps.remove(old_atom);
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
            let old_atom = &old_deps[i];
            if let Some(nodes) = self.deps.get_mut(old_atom) {
                nodes.retain(|&n| n != id);
                if nodes.is_empty() {
                    self.deps.remove(old_atom);
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
        let oldc = self.tree.children(parent).to_vec();
        let parent_env = self.tree.get(parent).unwrap().env.clone();
        let mut used = HashSet::new();
        let mut newc = Vec::new();

        for bp in blueprints {
            let m = oldc.iter().copied().find(|&c| {
                !used.contains(&c)
                    && self
                        .tree
                        .get(c)
                        .is_some_and(|ins| ins.type_id == bp.type_id)
            });
            let child = match m {
                Some(ch) => {
                    used.insert(ch);
                    let (should_update, env_changed, children_changed) = {
                        let ins = self.tree.get_mut(ch).unwrap();
                        let should_update = ins
                            .component
                            .changed_any(ins.props.as_ref(), bp.props.as_ref());
                        let env_changed = !ins.inherited.same(&parent_env);
                        let children_changed = !Rc::ptr_eq(&ins.children, &bp.children);

                        ins.props = bp.props.clone();
                        ins.inherited = parent_env.clone();
                        if children_changed {
                            ins.children = bp.children.clone();
                        }

                        (should_update, env_changed, children_changed)
                    };

                    if children_changed {
                        self.sync(ch, &bp.children);
                    }

                    if should_update || env_changed {
                        self.update_dirty.insert(ch);
                    }

                    ch
                }
                None => self.do_create(bp.clone(), Some(parent)),
            };
            newc.push(child);
        }

        for old in oldc {
            if !used.contains(&old) {
                self.drop_node(old);
            }
        }

        self.tree.set_children(parent, &newc);
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

        for &node in &subtree {
            self.unsub(node);
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
    }

    fn unsub(&mut self, id: NodeId) {
        if let Some(atoms) = self.node_deps.remove(&id) {
            for atom in atoms {
                if let Some(nodes) = self.deps.get_mut(&atom) {
                    nodes.retain(|&n| n != id);
                    if nodes.is_empty() {
                        self.deps.remove(&atom);
                    }
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
                env: inherited.clone(),
            },
            bp.props.as_ref(),
        );

        let ins = Instance {
            component,
            props: bp.props,
            children: bp.children,
            type_id: bp.type_id,
            rect: Rect::new(0, 0, 0, 0),
            offset: Offset::ZERO,
            measured: Size::default(),
            available: None,
            env: inherited.clone(),
            inherited,
        };

        let id = match parent {
            Some(p) => self.tree.insert(p, ins),
            None => self.tree.create_root(ins),
        };

        if !self.do_build(id) {
            let children = self.tree.get(id).unwrap().children.clone();
            self.sync(id, &children);
        }
        id
    }

    fn do_layout(&mut self) {
        loop {
            self.layout_dirty.retain(|&n| self.tree.contains(n));

            let Some(id) = self
                .layout_dirty
                .iter()
                .copied()
                .filter(|&id| self.tree.contains(id))
                .min_by_key(|&id| self.tree.depth(id))
            else {
                break;
            };

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

    fn apply_measure(&mut self, id: NodeId, available: Size) -> bool {
        let child_ids = self.tree.children(id).to_vec();
        let child_count = child_ids.len();
        let env = self.tree.get(id).unwrap().env.clone();
        let rect = self.tree.get(id).unwrap().rect;

        let mut measures: Vec<Option<Size>> = vec![None; child_count];

        let mut component = {
            let ins = self.tree.get_mut(id).unwrap();
            mem::replace(&mut ins.component, Box::new(Placeholder))
        };
        let props = {
            let ins = self.tree.get_mut(id).unwrap();
            mem::replace(&mut ins.props, Rc::new(()))
        };

        let mut measure = |index: usize, child_available: Size| -> Size {
            if index >= child_count {
                return Size::default();
            }
            match measures[index] {
                Some(size) if child_available == available => return size,
                _ => {}
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

    fn measure_node(tree: &mut Tree<Instance>, id: NodeId, available: Size) -> Size {
        let child_ids = tree.children(id).to_vec();
        let child_count = child_ids.len();

        if child_count == 0
            && let Some(cached) = tree.get(id).and_then(|ins| ins.available)
            && cached == available
            && let Some(ins) = tree.get(id)
        {
            return ins.measured;
        }

        let env = tree.get(id).unwrap().env.clone();
        let rect = tree.get(id).unwrap().rect;

        let mut measures: Vec<Option<Size>> = vec![None; child_count];

        let mut component = {
            let ins = tree.get_mut(id).unwrap();
            mem::replace(&mut ins.component, Box::new(Placeholder))
        };
        let props = {
            let ins = tree.get_mut(id).unwrap();
            mem::replace(&mut ins.props, Rc::new(()))
        };

        let mut measure = |index: usize, child_available: Size| -> Size {
            if index >= child_count {
                return Size::default();
            }
            match measures[index] {
                Some(size) if child_available == available => return size,
                _ => {}
            }
            let child_id = child_ids[index];
            if !tree.contains(child_id) {
                return Size::default();
            }
            let size = Self::measure_node(tree, child_id, child_available);
            measures[index] = Some(size);
            size
        };

        let measured = {
            let mut cx = Cx {
                node: Some(id),
                rect,
                actions: None,
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
            let ins = self.tree.get_mut(id).unwrap();
            ins.rect = rect;
            ins.offset = offset;
            return;
        }

        let child_ids = self.tree.children(id).to_vec();
        let child_sizes: Vec<Size> = child_ids
            .iter()
            .map(|child| {
                self.tree
                    .get(*child)
                    .map_or(Size::default(), |node| node.measured)
            })
            .collect();
        let mut rects = vec![Rect::default(); child_ids.len()];
        let mut offsets = vec![Offset::ZERO; child_ids.len()];
        let layouts = {
            let ins = self.tree.get_mut(id).unwrap();
            let local = Self::local_rect(rect);
            let mut cx = Cx {
                node: Some(id),
                rect: local,
                actions: None,
                env: ins.env.clone(),
            };
            let mut children = LayoutCx::new(&child_sizes, &mut rects, &mut offsets);
            ins.component
                .layout_any(&mut cx, ins.props.as_ref(), local, &mut children);
            (children.rects().to_vec(), children.offsets().to_vec())
        };
        {
            let ins = self.tree.get_mut(id).unwrap();
            ins.rect = rect;
            ins.offset = offset;
        }

        for ((child, rect), offset) in child_ids.iter().zip(layouts.0).zip(layouts.1) {
            if self.tree.contains(*child) {
                self.apply_layout(*child, rect, offset);
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
            self.paint_dirty.retain(|&n| self.tree.contains(n));

            let Some(id) = self
                .paint_dirty
                .iter()
                .copied()
                .filter(|&id| self.tree.contains(id))
                .min_by_key(|&id| self.tree.depth(id))
            else {
                break;
            };

            self.tree.visit_subtree(id, |node| {
                self.paint_dirty.remove(&node);
            });

            if self.tree.root() == Some(id) {
                self.back.clear();
            }
            let (origin, clip) = self.resolve(id).unwrap();
            self.apply_paint(id, origin, clip);
        }
    }

    fn apply_paint(&mut self, id: NodeId, origin: Offset, clip: Rect) {
        let children = {
            let ins = self.tree.get(id).unwrap();
            let mut cx = Cx {
                node: Some(id),
                rect: Self::local_rect(ins.rect),
                actions: None,
                env: ins.env.clone(),
            };
            let mut canvas = Canvas::new(&mut self.back, clip, origin);
            ins.component
                .paint_any(&mut cx, ins.props.as_ref(), &mut canvas);
            self.tree.children(id).to_vec()
        };
        for child in children {
            if self.tree.contains(child) {
                let child_ins = self.tree.get(child).unwrap();
                let child_origin = Self::translate(origin, child_ins.rect, child_ins.offset);
                let Some(child_clip) = Self::clip(
                    child_origin,
                    Size::new(child_ins.rect.width, child_ins.rect.height),
                    clip,
                ) else {
                    continue;
                };
                self.apply_paint(child, child_origin, child_clip);
            }
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
