#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && self.y < other.bottom()
            && other.x < self.right()
            && other.y < self.bottom()
    }

    pub fn intersection(&self, other: &Rect) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Some(Self::new(left, top, right - left, bottom - top))
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x
            && y >= self.y
            && x.saturating_sub(self.x) < self.width
            && y.saturating_sub(self.y) < self.height
    }

    pub fn left(&self) -> u16 {
        self.x
    }

    pub fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub fn top(&self) -> u16 {
        self.y
    }

    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }
}
