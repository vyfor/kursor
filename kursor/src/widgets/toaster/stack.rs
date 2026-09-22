use std::{collections::VecDeque, time::Duration};

use kursor_core::{
    component::{
        Component, MountChildren, Update, blueprint::Blueprint, context::Cx,
    },
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    render::canvas::Canvas,
};

#[cfg(feature = "fx")]
use crate::fx::{Effect, FxCell};

use super::{
    ToasterProps,
    entry::{Entry, Motion},
    handle::{ToastId, ToasterCommand, Toasts},
};

struct QueuedToast {
    id: ToastId,
    content: Blueprint,
    duration: Option<Duration>,
}

#[derive(Clone, PartialEq)]
pub(crate) struct ToastStackProps {
    pub(crate) handle: Toasts,
    pub(crate) config: ToasterProps,
}

pub(crate) struct ToastStack {
    handle: Toasts,
    config: ToasterProps,
    entries: Vec<Entry>,
    queued: VecDeque<QueuedToast>,
    animating: bool,
}

impl ToastStack {
    pub(crate) fn blueprint(handle: Toasts, config: ToasterProps) -> Blueprint {
        Blueprint::new::<Self>(ToastStackProps { handle, config })
    }

    fn hard_limit(&self) -> Option<usize> {
        self.config
            .hard_limit
            .or_else(|| self.config.soft_limit.map(|s| s + 2))
    }

    fn blueprints(&self) -> Vec<Blueprint> {
        self.entries
            .iter()
            .map(|entry| {
                #[cfg(feature = "fx")]
                let child =
                    Effect::new(entry.content.clone(), entry.cell.clone());
                #[cfg(not(feature = "fx"))]
                let child = entry.content.clone();
                child.key(entry.id.0)
            })
            .collect()
    }

    fn force_limit(&mut self) -> bool {
        let Some(limit) = self.config.soft_limit else {
            return false;
        };
        if self.animating
            || self.entries.iter().any(|e| e.exiting || e.motion.is_some())
        {
            return false;
        }
        let live = self.entries.iter().filter(|e| !e.exiting).count();
        let mut excess = live.saturating_sub(limit);
        let mut changed = false;
        for entry in &mut self.entries {
            if excess == 0 {
                break;
            }
            if !entry.exiting && entry.motion.is_none() {
                entry.exit(&self.config);
                excess -= 1;
                changed = true;
            }
        }
        changed
    }
}

impl Component for ToastStack {
    type Props = ToastStackProps;

    fn create(_cx: &mut Cx, props: &Self::Props) -> Self {
        Self {
            handle: props.handle.clone(),
            config: props.config.clone(),
            entries: Vec::new(),
            queued: VecDeque::new(),
            animating: false,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn mount(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        _children: &mut MountChildren,
    ) {
        self.handle = props.handle.clone();
        self.config = props.config.clone();
        if let Some(host) = cx.node {
            self.handle.attach(host);
        }
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        self.config = props.config.clone();
        if let Some(host) = cx.node {
            self.handle.attach(host);
        }

        let mut changed = false;
        let hard_limit = self.hard_limit();

        for command in self.handle.drain() {
            match command {
                ToasterCommand::Push {
                    id,
                    content,
                    duration,
                } => {
                    if hard_limit.is_some_and(|hard| self.entries.len() >= hard)
                    {
                        if self.queued.len() >= 50 {
                            self.queued.pop_front();
                        }
                        self.queued.push_back(QueuedToast {
                            id,
                            content,
                            duration,
                        });
                    } else {
                        let deadline = duration.map(|d| cx.time().elapsed + d);
                        let entry = Entry {
                            id,
                            content,
                            deadline,
                            #[cfg(feature = "fx")]
                            cell: FxCell::new(),
                            exiting: false,
                            rect: Rect::new(0, 0, 0, 0),
                            motion: None,
                        };
                        #[cfg(feature = "fx")]
                        entry.cell.set(self.config.enter.clone());
                        self.entries.push(entry);
                        changed = true;
                    }
                }
                ToasterCommand::Dismiss(id) => {
                    self.queued.retain(|q| q.id != id);
                    if let Some(entry) = self
                        .entries
                        .iter_mut()
                        .find(|e| e.id == id && !e.exiting)
                    {
                        entry.exit(&self.config);
                        changed = true;
                    }
                }
                ToasterCommand::DismissAll => {
                    self.queued.clear();
                    for entry in &mut self.entries {
                        if !entry.exiting {
                            entry.exit(&self.config);
                        }
                    }
                    changed = true;
                }
                ToasterCommand::Replace(id, content) => {
                    if let Some(q) = self.queued.iter_mut().find(|q| q.id == id)
                    {
                        q.content = content.clone();
                    }
                    if let Some(entry) = self
                        .entries
                        .iter_mut()
                        .find(|e| e.id == id && !e.exiting)
                    {
                        entry.content = content;
                        changed = true;
                    }
                }
            }
        }

        if self.force_limit() {
            changed = true;
        }

        let before = self.entries.len();
        self.entries.retain(|entry| !entry.finished());
        let dropped = before != self.entries.len();
        if dropped {
            changed = true;
        }

        while let Some(queued) = self.queued.pop_front() {
            if hard_limit.is_some_and(|hard| self.entries.len() >= hard) {
                self.queued.push_front(queued);
                break;
            }
            let deadline = queued.duration.map(|d| cx.time().elapsed + d);
            let entry = Entry {
                id: queued.id,
                content: queued.content,
                deadline,
                #[cfg(feature = "fx")]
                cell: FxCell::new(),
                exiting: false,
                rect: Rect::new(0, 0, 0, 0),
                motion: None,
            };
            #[cfg(feature = "fx")]
            entry.cell.set(self.config.enter.clone());
            self.entries.push(entry);
            changed = true;
        }

        let busy = dropped
            || self.animating
            || self.entries.iter().any(|e| e.exiting || e.motion.is_some());
        if !busy {
            if self.force_limit() {
                changed = true;
            }
            let now = cx.time().elapsed;
            for entry in &mut self.entries {
                if entry.motion.is_none()
                    && entry.deadline.is_some_and(|deadline| deadline <= now)
                {
                    entry.exit(&self.config);
                    changed = true;
                    break;
                }
            }
        }

        if changed {
            Update::children(self.blueprints())
        } else if self.animating {
            Update::MEASURE
        } else {
            Update::NONE
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        let mut width = 0;
        let mut height = 0;
        for index in 0..children.len() {
            let size = children.measure(index, available);
            width = width.max(size.width);
            height += size.height;
        }
        if children.len() > 1 {
            height += self.config.gap * (children.len() as u16 - 1);
        }
        for entry in &self.entries {
            if let Some(motion) = &entry.motion {
                height = height.max(motion.y + motion.height);
            }
        }
        Size::new(width, height)
    }

    fn layout(
        &mut self,
        cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        let now = cx.time().elapsed;
        let duration = self.config.pace;
        let mut y = area.y;
        let mut animating = false;

        let prev_rects: Vec<Rect> =
            self.entries.iter().map(|e| e.rect).collect();

        for index in 0..children.len() {
            let height = children.size(index).height;

            let Some(entry) = self.entries.get_mut(index) else {
                let target = Rect::new(area.x, y, area.width, height);
                y = y.max(target.bottom() + self.config.gap);
                children.set(index, target);
                continue;
            };

            if entry.exiting {
                children.set(index, entry.rect);
                y = y.max(entry.rect.bottom() + self.config.gap);
                continue;
            }

            if entry.rect.height == 0 && index > 0 {
                if let Some(prev_rect) = prev_rects.get(index - 1) {
                    if prev_rect.height > 0 {
                        y = y.max(prev_rect.bottom() + self.config.gap);
                    }
                }
            }

            let target = Rect::new(area.x, y, area.width, height);
            y += height + self.config.gap;

            if let Some(motion) = &mut entry.motion {
                let motion_target_y =
                    (area.y as i32 + motion.y as i32 + motion.distance).max(0)
                        as u16;

                if target.y != motion_target_y {
                    let current_y = if now < motion.start {
                        area.y + motion.y
                    } else {
                        let pace = duration.as_secs_f32().max(0.001);
                        let travelled = ((now - motion.start).as_secs_f32()
                            / pace)
                            .min(motion.distance.abs() as f32);
                        let step = (travelled * motion.distance.signum() as f32)
                            .round() as i32;
                        (area.y as i32 + motion.y as i32 + step).max(0) as u16
                    };

                    if target.y != current_y && duration > Duration::ZERO {
                        let from_y = current_y.saturating_sub(area.y);
                        let distance = target.y as i32 - current_y as i32;
                        *motion = Motion {
                            y: from_y,
                            distance,
                            height: entry.rect.height,
                            start: now,
                        };
                    } else {
                        entry.motion = None;
                    }
                }
            } else if entry.rect != target {
                if entry.rect.height > 0 && duration > Duration::ZERO {
                    let from_y = entry.rect.y.saturating_sub(area.y);
                    let distance = target.y as i32 - entry.rect.y as i32;
                    entry.motion = Some(Motion {
                        y: from_y,
                        distance,
                        height: entry.rect.height,
                        start: now + self.config.delay,
                    });
                }
            }

            let rect = if let Some(motion) = &entry.motion {
                if now < motion.start {
                    animating = true;
                    Rect::new(
                        area.x,
                        area.y + motion.y,
                        area.width,
                        motion.height,
                    )
                } else {
                    let pace = duration.as_secs_f32().max(0.001);
                    let travelled = ((now - motion.start).as_secs_f32() / pace)
                        .min(motion.distance.abs() as f32);
                    if travelled >= motion.distance.abs() as f32 {
                        entry.motion = None;
                        target
                    } else {
                        animating = true;
                        let step = (travelled * motion.distance.signum() as f32)
                            .round() as i32;
                        let ny = area.y as i32 + motion.y as i32 + step;
                        Rect::new(area.x, ny.max(0) as u16, area.width, height)
                    }
                }
            } else {
                target
            };

            entry.rect = rect;
            children.set(index, rect);
        }

        self.animating = animating;
    }

    fn post_paint(
        &mut self,
        cx: &mut Cx,
        _props: &Self::Props,
        _canvas: &mut Canvas,
    ) -> bool {
        let any_exiting = self.entries.iter().any(|e| e.exiting);
        if !any_exiting && !self.animating {
            let next_deadline = self
                .entries
                .iter()
                .filter(|e| !e.exiting)
                .filter_map(|e| e.deadline)
                .min();

            if let Some(deadline) = next_deadline {
                let delay = deadline.saturating_sub(cx.time().elapsed);
                cx.wake_after(delay);
            }
        }

        any_exiting || self.animating
    }

    fn drop(&mut self, cx: &mut Cx) {
        if let Some(host) = cx.node {
            self.handle.detach(host);
        }
    }
}
