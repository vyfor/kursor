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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum GraphicsProtocol {
    Kitty,
    Sixel,
    Iterm2,
    Halfblocks,
}
