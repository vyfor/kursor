use crate::{layout::size::Size, render::cell::Cell};

pub struct Buffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        Self {
            width: size.width,
            height: size.height,
            cells: vec![Cell::default(); size.width as usize * size.height as usize],
        }
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
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn diff(&self, old: &Buffer) -> Vec<CellDiff> {
        if self.size() != old.size() {
            return self
                .cells
                .iter()
                .enumerate()
                .map(|(i, &cell)| CellDiff {
                    x: (i as u32 % self.width as u32) as u16,
                    y: (i as u32 / self.width as u32) as u16,
                    cell,
                })
                .collect();
        }

        self.cells
            .iter()
            .zip(&old.cells)
            .enumerate()
            .filter(|(_, (old, new))| new != old)
            .map(|(i, (&cell, _))| CellDiff {
                x: (i as u32 % self.width as u32) as u16,
                y: (i as u32 / self.width as u32) as u16,
                cell,
            })
            .collect()
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub cell: Cell,
}
