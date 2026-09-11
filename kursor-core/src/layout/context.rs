use crate::layout::{offset::Offset, rect::Rect, size::Size};

pub struct MeasureCx<'a> {
    measure: &'a mut dyn FnMut(usize, Size) -> Size,
    len: usize,
    available: Size,
}

impl<'a> MeasureCx<'a> {
    pub fn new(
        measure: &'a mut dyn FnMut(usize, Size) -> Size,
        len: usize,
        available: Size,
    ) -> Self {
        Self {
            measure,
            len,
            available,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn size(&mut self, index: usize) -> Size {
        (self.measure)(index, self.available)
    }

    pub fn measure(&mut self, index: usize, available: Size) -> Size {
        (self.measure)(index, available)
    }
}

pub struct LayoutCx<'a> {
    sizes: &'a [Size],
    rects: &'a mut [Rect],
    offsets: &'a mut [Offset],
}

impl<'a> LayoutCx<'a> {
    pub(crate) fn new(
        sizes: &'a [Size],
        rects: &'a mut [Rect],
        offsets: &'a mut [Offset],
    ) -> Self {
        assert_eq!(sizes.len(), rects.len());
        assert_eq!(rects.len(), offsets.len());
        Self {
            sizes,
            rects,
            offsets,
        }
    }

    pub fn len(&self) -> usize {
        self.sizes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sizes.is_empty()
    }

    pub fn size(&self, index: usize) -> Size {
        self.sizes[index]
    }

    pub fn rect(&self, index: usize) -> Rect {
        self.rects[index]
    }

    pub fn set(&mut self, index: usize, rect: Rect) {
        self.rects[index] = rect;
    }

    pub fn translate(&mut self, index: usize, offset: Offset) {
        self.offsets[index] = offset;
    }

    pub fn rects(&self) -> &[Rect] {
        self.rects
    }

    pub fn offsets(&self) -> &[Offset] {
        self.offsets
    }
}
