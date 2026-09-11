#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
    Bmp,
    Tiff,
    Other,
}

/// terminal graphics protocol used to render the image.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum GraphicsProtocol {
    /// https://sw.kovidgoyal.net/kitty/graphics-protocol
    Kitty,
    /// https://en.wikipedia.org/wiki/Sixel
    Sixel,
    /// https://iterm2.com/documentation-images.html
    Iterm2,
    /// unicode half block characters `▀`, `▄`.
    Halfblocks,
}
