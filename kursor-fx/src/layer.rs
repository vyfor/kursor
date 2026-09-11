use kursor_core::{
    layout::rect::Rect,
    render::{canvas::Canvas, cell::Cell},
};

#[derive(Clone, Copy)]
pub struct LayerCell {
    pub x: u16,
    pub y: u16,
    pub cell: Cell,
}

#[derive(Clone, Default)]
pub struct EffectLayer {
    area: Rect,
    /// what was already on the canvas (i.e. before children's `paint`).
    underlay: Vec<Cell>,
    /// what the underlying children drew on the canvas.
    source: Vec<Cell>,
    /// what the effect actually writes to the canvas.
    output: Vec<Cell>,
}

impl EffectLayer {
    pub fn new() -> Self {
        Self {
            area: Rect::new(0, 0, 0, 0),
            underlay: Vec::new(),
            source: Vec::new(),
            output: Vec::new(),
        }
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn width(&self) -> u16 {
        self.area.width
    }

    pub fn height(&self) -> u16 {
        self.area.height
    }

    pub fn source(&self, x: u16, y: u16) -> Cell {
        self.source
            .get(self.index(x, y))
            .copied()
            .unwrap_or_default()
    }

    pub fn underlay(&self, x: u16, y: u16) -> Cell {
        self.underlay
            .get(self.index(x, y))
            .copied()
            .unwrap_or_default()
    }

    pub fn get(&self, x: u16, y: u16) -> Cell {
        self.output
            .get(self.index(x, y))
            .copied()
            .unwrap_or_default()
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        let index = self.index(x, y);
        if let Some(output) = self.output.get_mut(index) {
            *output = cell;
        }
    }

    pub fn clear(&mut self) {
        if self.underlay.len() == self.output.len() {
            self.output.copy_from_slice(&self.underlay);
        } else {
            self.output.fill(Cell::default());
        }
    }

    pub(crate) fn advance(&mut self) {
        self.source.copy_from_slice(&self.output);
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(LayerCell),
    {
        for y in 0..self.height() {
            for x in 0..self.width() {
                f(LayerCell {
                    x,
                    y,
                    cell: self.get(x, y),
                });
            }
        }
    }

    pub(crate) fn capture(&mut self, canvas: &Canvas<'_>, area: Rect) {
        self.area = Rect::new(0, 0, area.width, area.height);

        let len = area.width as usize * area.height as usize;
        self.source.resize(len, Cell::default());
        self.output.resize(len, Cell::default());

        if self.underlay.len() != len {
            self.underlay.resize(len, Cell::default());
        }

        for y in 0..area.height {
            for x in 0..area.width {
                let index = self.index(x, y);
                let cell = canvas.cell(x, y).unwrap_or_default();

                self.source[index] = cell;
                self.output[index] = cell;
            }
        }
    }

    pub(crate) fn capture_underlay(&mut self, canvas: &Canvas<'_>, area: Rect) {
        self.area = Rect::new(0, 0, area.width, area.height);
        let len = area.width as usize * area.height as usize;
        self.underlay.resize(len, Cell::default());

        for y in 0..area.height {
            for x in 0..area.width {
                let index = self.index(x, y);
                self.underlay[index] = canvas.cell(x, y).unwrap_or_default();
            }
        }
    }

    pub(crate) fn write(&self, canvas: &mut Canvas<'_>) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                canvas.set_cell(x, y, self.output[self.index(x, y)]);
            }
        }
    }

    pub(crate) fn write_underlay(&self, canvas: &mut Canvas<'_>) {
        if self.underlay.len() != self.output.len() {
            return;
        }

        for y in 0..self.height() {
            for x in 0..self.width() {
                canvas.set_cell(x, y, self.underlay[self.index(x, y)]);
            }
        }
    }

    fn index(&self, x: u16, y: u16) -> usize {
        y as usize * self.width() as usize + x as usize
    }
}
