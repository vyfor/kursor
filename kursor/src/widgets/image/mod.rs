pub mod builder;
pub use builder::ImageBuilder;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use kursor_core::{
    component::{Component, Update, blueprint::Blueprint, context::Cx},
    layout::{context::MeasureCx, size::Size},
    render::{buffer::GraphicsOp, canvas::Canvas},
    state::Value,
    term_info::TermInfo,
};
use kursor_image::{
    GraphicsProtocol, Halfblocks, ImageData, ImageEncoder, ImageFit,
    ImageSource, ImageTarget, Iterm2, Kitty, Sixel, Transmission, fit_cells,
};

#[derive(Clone, PartialEq)]
pub struct ImageProps {
    pub source: Value<ImageSource>,
    pub fit: Value<ImageFit>,
    pub prefer: Value<Option<GraphicsProtocol>>,
    pub transmission: Value<Transmission>,
}

pub struct Image {
    source: ImageSource,
    fit: ImageFit,
    prefer: Option<GraphicsProtocol>,
    transmission: Transmission,
}

impl Image {
    pub fn new(source: impl Into<ImageSource>) -> ImageBuilder {
        ImageBuilder::new(source)
    }

    pub fn file(path: impl Into<PathBuf>) -> Blueprint {
        Self::new(ImageSource::file(path)).build()
    }

    pub fn data(image: ImageData) -> Blueprint {
        Self::new(ImageSource::Data(image)).build()
    }

    pub fn with(props: ImageProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }
}

impl Component for Image {
    type Props = ImageProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            source: ImageSource::file(PathBuf::new()),
            fit: ImageFit::Contain,
            prefer: None,
            transmission: Transmission::Direct,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let source = props.source.get();
        let fit = props.fit.get();
        let prefer = props.prefer.get();
        let transmission = props.transmission.get();
        let changed = self.source != source
            || self.fit != fit
            || self.prefer != prefer
            || self.transmission != transmission;
        self.source = source;
        self.fit = fit;
        self.prefer = prefer;
        self.transmission = transmission;
        if changed { Update::PAINT } else { Update::NONE }
    }

    fn measure(
        &mut self,
        cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        _children: &mut MeasureCx,
    ) -> Size {
        let Some((width, height)) = pixel_size(&self.source, cx) else {
            return Size::default();
        };
        let cell = cell_size(cx);
        fit_cells(width, height, available, cell, self.fit).unwrap_or(available)
    }

    fn paint(&self, cx: &mut Cx, _props: &Self::Props, canvas: &mut Canvas) {
        let area = cx.rect;
        if area.width == 0 || area.height == 0 {
            return;
        }
        let Some(node) = cx.node else {
            return;
        };
        let info = cx.get::<TermInfo>().copied().unwrap_or_default();
        let protocol = self.prefer.unwrap_or_else(|| pick_protocol(&info));
        let cell = cell_size(cx).unwrap_or(Size::new(8, 16));
        let target = ImageTarget::with_cell_size(
            Size::new(area.width, area.height),
            cell,
        );
        let key = gkey(&self.source, protocol, self.transmission);

        match protocol {
            GraphicsProtocol::Halfblocks => {
                let Ok(cells) = Halfblocks.encode(&self.source, &target) else {
                    return;
                };
                for (i, cell) in cells.iter().enumerate() {
                    canvas.set_cell(
                        (i as u16) % area.width,
                        (i as u16) / area.width,
                        *cell,
                    );
                }
            }
            GraphicsProtocol::Kitty => {
                let encoder = match self.transmission {
                    Transmission::TempFile => Kitty::temp_file(),
                    Transmission::Direct => Kitty::new(),
                };
                let id = node.as_u32();
                let size = Size::new(area.width, area.height);
                let display = encoder.display(id, size);
                let Ok(transmit) = encoder.transmit(&self.source, id) else {
                    return;
                };
                let mut create = transmit;
                create.extend_from_slice(&display);
                let mut update = encoder.undisplay(id);
                update.extend_from_slice(&display);
                canvas.push_graphics(GraphicsOp {
                    key,
                    create,
                    update,
                    delete: encoder.delete(id),
                });
            }
            GraphicsProtocol::Iterm2 => {
                let Ok(data) = Iterm2.encode(&self.source, &target) else {
                    return;
                };
                canvas.push_graphics(GraphicsOp {
                    key,
                    create: data.clone(),
                    update: data,
                    delete: Vec::new(),
                });
            }
            GraphicsProtocol::Sixel => {
                let Ok(data) = Sixel::new().encode(&self.source, &target)
                else {
                    return;
                };
                canvas.push_graphics(GraphicsOp {
                    key,
                    create: data.clone(),
                    update: data,
                    delete: Vec::new(),
                });
            }
        }
    }
}

fn pick_protocol(info: &TermInfo) -> GraphicsProtocol {
    if info.graphics.kitty {
        GraphicsProtocol::Kitty
    } else if info.graphics.iterm2 {
        GraphicsProtocol::Iterm2
    } else if info.graphics.sixel {
        GraphicsProtocol::Sixel
    } else {
        GraphicsProtocol::Halfblocks
    }
}

fn cell_size(cx: &Cx) -> Option<Size> {
    let info = cx.get::<TermInfo>().copied().unwrap_or_default();
    info.cell_size.map(|(w, h)| Size::new(w, h))
}

fn pixel_size(source: &ImageSource, cx: &Cx) -> Option<(u32, u32)> {
    match source {
        ImageSource::Data(image) => Some(image.size()),
        ImageSource::File { .. } | ImageSource::Encoded { .. } => {
            let info = cx.get::<TermInfo>().copied().unwrap_or_default();
            let cell = info.cell_size.map(|(w, h)| Size::new(w, h))?;
            Some((
                u32::from(cx.rect.width) * u32::from(cell.width),
                u32::from(cx.rect.height) * u32::from(cell.height),
            ))
        }
    }
}

fn gkey(
    source: &ImageSource,
    protocol: GraphicsProtocol,
    transmission: Transmission,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    protocol.hash(&mut hasher);
    transmission.hash(&mut hasher);
    hasher.finish()
}
