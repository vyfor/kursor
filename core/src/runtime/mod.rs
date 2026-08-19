pub mod instance;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    mem,
    time::{Duration, Instant},
};

use crate::{
    component::{Component, Focus, action::Action, blueprint::Blueprint, context::Cx},
    event::{
        Event, EventResult, Phase,
        mouse::{MouseButton, MouseEvent, MouseKind},
    },
    layout::{
        context::{LayoutCx, MeasureCx},
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
        self.cursor.map(|(_, x, y)| (x, y))
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
                                .tree
                                .get(id)
                                .is_some_and(|node| node.rect.contains(mouse.column, mouse.row))
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
        self.hit(root, self.rect, mouse.column, mouse.row)
    }

    fn hit(&self, id: NodeId, clip: Rect, x: u16, y: u16) -> Option<NodeId> {
        let rect = self.tree.get(id)?.rect;
        let clip = clip.intersection(&rect)?;
        if !clip.contains(x, y) {
            return None;
        }
        for &child in self.tree.children(id).iter().rev() {
            if let Some(target) = self.hit(child, clip, x, y) {
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
                rect: ins.rect,
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

    fn do_build(&mut self, id: NodeId) {
        let inherited = self
            .tree
            .parent(id)
            .and_then(|parent| self.tree.get(parent).map(|node| node.env.clone()))
            .unwrap_or_default();
        let (bps, dep_reads) = deps::collect(|| {
            let ins = self.tree.get_mut(id).unwrap();
            let children = ins.children.clone();
            let mut cx = Cx {
                rect: ins.rect,
                node: Some(id),
                actions: None,
                env: inherited.clone(),
            };
            let output = ins
                .component
                .build_any(&mut cx, ins.props.as_ref(), children);
            ins.inherited = inherited.clone();
            ins.env = cx.env.clone();
            output
        });

        self.do_deps(id, dep_reads);
        self.sync(id, bps);
        self.layout_dirty.insert(id);
        self.paint_dirty.insert(id);
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

    fn sync(&mut self, parent: NodeId, blueprints: Vec<Blueprint>) {
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
                    let ins = self.tree.get_mut(ch).unwrap();
                    let should_update = ins
                        .component
                        .changed_any(ins.props.as_ref(), bp.props.as_ref());
                    let env_changed = !ins.inherited.same(&parent_env);

                    ins.props = bp.props;
                    ins.inherited = parent_env.clone();
                    ins.children = bp.children;

                    if should_update || env_changed {
                        self.update_dirty.insert(ch);
                    }

                    ch
                }
                None => self.do_create(bp, Some(parent)),
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
                rect: ins.rect,
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
            measured: Size::default(),
            available: None,
            env: inherited.clone(),
            inherited,
        };

        let id = match parent {
            Some(p) => self.tree.insert(p, ins),
            None => self.tree.create_root(ins),
        };

        self.do_build(id);
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

            self.apply_measure(id, Size::new(rect.width, rect.height));
            self.apply_layout(id, rect);
        }
    }

    fn apply_measure(&mut self, id: NodeId, available: Size) {
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
            mem::replace(&mut ins.props, Box::new(()))
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

        {
            let ins = self.tree.get_mut(id).unwrap();
            ins.component = component;
            ins.props = props;
            ins.measured = measured;
            ins.available = Some(available);
        }
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
            mem::replace(&mut ins.props, Box::new(()))
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

    fn apply_layout(&mut self, id: NodeId, rect: Rect) {
        let is_dirty = self.layout_dirty.remove(&id);
        let old_rect = self.tree.get(id).map_or(Rect::default(), |ins| ins.rect);

        if !is_dirty && old_rect == rect {
            return;
        }

        if old_rect != rect {
            self.paint_all = true;
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
        let layouts = {
            let ins = self.tree.get_mut(id).unwrap();
            let mut cx = Cx {
                node: Some(id),
                rect,
                actions: None,
                env: ins.env.clone(),
            };
            let mut children = LayoutCx::new(&child_sizes, &mut rects);
            ins.component
                .layout_any(&mut cx, ins.props.as_ref(), rect, &mut children);
            children.rects().to_vec()
        };
        self.tree.get_mut(id).unwrap().rect = rect;

        for (child, layout) in child_ids.iter().zip(layouts) {
            if self.tree.contains(*child) {
                self.apply_layout(*child, layout);
            }
        }
    }

    fn do_paint(&mut self) {
        if self.paint_all {
            self.paint_all = false;
            self.paint_dirty.clear();
            self.back.clear();
            if let Some(root) = self.tree.root() {
                self.apply_paint(root, self.rect);
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
            self.apply_paint(id, self.rect);
        }
    }

    fn apply_paint(&mut self, id: NodeId, parent_clip: Rect) {
        let (children, clip) = {
            let ins = self.tree.get(id).unwrap();
            let Some(clip) = parent_clip.intersection(&ins.rect) else {
                return;
            };
            let mut cx = Cx {
                node: Some(id),
                rect: ins.rect,
                actions: None,
                env: ins.env.clone(),
            };
            let mut canvas = Canvas::new(&mut self.back, clip);
            ins.component
                .paint_any(&mut cx, ins.props.as_ref(), &mut canvas);
            (self.tree.children(id).to_vec(), clip)
        };
        for child in children {
            if self.tree.contains(child) {
                self.apply_paint(child, clip);
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
