use std::collections::BTreeMap;

use crate::{
    layout::{rect::Rect, size::Size},
    render::cell::Cell,
    tree::id::NodeId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicsOp {
    pub key: u64,
    pub create: Vec<u8>,
    pub update: Vec<u8>,
    pub delete: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicsEntry {
    pub area: Rect,
    pub op: GraphicsOp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedGraphics {
    pub area: Rect,
    pub key: u64,
    pub delete: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphicsDiff {
    pub clears: Vec<GraphicsClear>,
    pub draws: Vec<GraphicsDraw>,
}

impl GraphicsDiff {
    pub fn is_empty(&self) -> bool {
        self.clears.is_empty() && self.draws.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicsClear {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    pub prelude: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicsDraw {
    pub x: u16,
    pub y: u16,
    pub data: Vec<u8>,
}

pub fn diff_graphics(
    current: &BTreeMap<NodeId, GraphicsEntry>,
    committed: &mut BTreeMap<NodeId, CommittedGraphics>,
    reset: bool,
) -> GraphicsDiff {
    let mut diff = GraphicsDiff::default();
    let mut cleared: Vec<Rect> = Vec::new();

    if reset {
        for old in committed.values() {
            diff.clears.push(GraphicsClear {
                x: old.area.x,
                y: old.area.y,
                w: old.area.width,
                h: old.area.height,
                prelude: old.delete.clone(),
            });
            cleared.push(old.area);
        }
        committed.clear();
    }

    for (node, old) in committed.iter() {
        match current.get(node) {
            Some(new) if new.area == old.area && new.op.key == old.key => {}
            Some(new) if new.op.key == old.key => {
                diff.clears.push(GraphicsClear {
                    x: old.area.x,
                    y: old.area.y,
                    w: old.area.width,
                    h: old.area.height,
                    prelude: Vec::new(),
                });
                cleared.push(old.area);
            }
            _ => {
                diff.clears.push(GraphicsClear {
                    x: old.area.x,
                    y: old.area.y,
                    w: old.area.width,
                    h: old.area.height,
                    prelude: old.delete.clone(),
                });
                cleared.push(old.area);
            }
        }
    }
    committed.retain(|node, _| current.contains_key(node));

    for (node, new) in current {
        let old = committed.get(node);
        let unchanged =
            old.is_some_and(|o| o.area == new.area && o.key == new.op.key);
        let overlaps_clear =
            cleared.iter().any(|area| area.intersects(&new.area));
        if unchanged && !overlaps_clear {
            committed.insert(
                *node,
                CommittedGraphics {
                    area: new.area,
                    key: new.op.key,
                    delete: new.op.delete.clone(),
                },
            );
            continue;
        }

        let data = if old.is_some_and(|o| o.key == new.op.key) {
            new.op.update.clone()
        } else {
            new.op.create.clone()
        };
        diff.draws.push(GraphicsDraw {
            x: new.area.x,
            y: new.area.y,
            data,
        });
        committed.insert(
            *node,
            CommittedGraphics {
                area: new.area,
                key: new.op.key,
                delete: new.op.delete.clone(),
            },
        );
    }

    diff
}

pub struct Buffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
    graphics: BTreeMap<NodeId, GraphicsEntry>,
    dirty_rect: Option<Rect>,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        Self {
            width: size.width,
            height: size.height,
            cells: vec![
                Cell::default();
                size.width as usize * size.height as usize
            ],
            graphics: BTreeMap::new(),
            dirty_rect: None,
        }
    }

    pub fn push_graphics(
        &mut self,
        node: NodeId,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        op: GraphicsOp,
    ) {
        self.graphics.insert(
            node,
            GraphicsEntry {
                area: Rect::new(x, y, w, h),
                op,
            },
        );
    }

    pub fn remove_graphics(&mut self, node: NodeId) {
        self.graphics.remove(&node);
    }

    pub fn graphics(&self) -> &BTreeMap<NodeId, GraphicsEntry> {
        &self.graphics
    }

    pub fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y as usize * self.width as usize + x as usize)
        } else {
            None
        }
    }

    pub fn cell(&self, x: u16, y: u16) -> Option<&Cell> {
        self.index(x, y).map(|i| &self.cells[i])
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = cell;
            if let Some(r) = self.dirty_rect
                && r.x == 0
                && r.y == 0
                && r.width == self.width
                && r.height == self.height
            {
                return;
            }
            self.dirty_rect = Some(match self.dirty_rect {
                None => Rect::new(x, y, 1, 1),
                Some(r) => {
                    let x0 = r.x.min(x);
                    let y0 = r.y.min(y);
                    let x1 = r.right().max(x + 1);
                    let y1 = r.bottom().max(y + 1);
                    Rect::new(x0, y0, x1 - x0, y1 - y0)
                }
            });
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
        self.graphics.clear();
        self.dirty_rect = Some(Rect::new(0, 0, self.width, self.height));
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn diff(&mut self, old: &Buffer) -> Vec<CellDiff> {
        let mut out = Vec::new();
        self.diff_into(old, &mut out);
        out
    }

    pub fn diff_into(&mut self, old: &Buffer, out: &mut Vec<CellDiff>) {
        if self.size() != old.size() {
            out.clear();
            out.extend(self.cells.iter().enumerate().map(|(i, &cell)| {
                CellDiff {
                    x: (i as u32 % self.width as u32) as u16,
                    y: (i as u32 / self.width as u32) as u16,
                    cell,
                }
            }));
            self.dirty_rect = None;
            return;
        }

        out.clear();
        let rect = match self.dirty_rect.take() {
            None => return,
            Some(r) => r,
        };

        let x0 = rect.x.min(self.width);
        let y0 = rect.y.min(self.height);
        let x1 = (rect.x + rect.width).min(self.width);
        let y1 = (rect.y + rect.height).min(self.height);

        for y in y0..y1 {
            let row = y as usize * self.width as usize;
            for x in x0..x1 {
                let i = row + x as usize;
                if self.cells[i] != old.cells[i] {
                    out.push(CellDiff {
                        x,
                        y,
                        cell: self.cells[i],
                    });
                }
            }
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub cell: Cell,
}
