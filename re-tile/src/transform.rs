//! Transform image.

use ::std::{num::NonZero, ops::DerefMut};

use ::clap::ValueEnum;
use ::color_eyre::eyre::{bail, ensure};
use ::image::{DynamicImage, GenericImage, GenericImageView};
use ::imageproc::drawing::{draw_text_mut, text_size};

/// Convert a NonZero u16 to a u32.
fn to_u32(i: NonZero<u16>) -> u32 {
    i.get().into()
}

/// How to align the image on an axis.
#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum Align {
    /// Align at start.
    Start,
    /// Align at center.
    #[default]
    Center,
    /// Align at end.
    End,
}

impl Align {
    /// Align a length.
    pub const fn offset(self, from_len: u32, to_len: u32) -> u32 {
        assert!(from_len <= to_len);

        match self {
            Align::Start => 0,
            Align::Center => (to_len - from_len) / 2,
            Align::End => to_len - from_len,
        }
    }
}

/// Current state used for index text.
#[derive(Debug)]
struct TextState {
    /// Current index.
    index: isize,
    /// Current scale divisor.
    divisor: NonZero<u32>,
    /// Font to use.
    font: ab_glyph::FontRef<'static>,
    /// Color to draw with.
    color: image::Rgba<u8>,
}

/// Re-tile image.
///
/// # Errors
/// If transform cannot be performed.
#[expect(clippy::too_many_arguments)]
pub fn transform(
    image: DynamicImage,
    from: [NonZero<u16>; 2],
    to: [NonZero<u16>; 2],
    align_x: Align,
    align_y: Align,
    index_start: Option<isize>,
    align_x_text: Align,
    align_y_text: Align,
) -> ::color_eyre::Result<DynamicImage> {
    let [from_w, from_h] = from.map(to_u32);
    let [to_w, to_h] = to.map(to_u32);

    ensure!(
        from_w <= to_w,
        "from width {from_w} is higher than to width {to_w}"
    );
    ensure!(
        from_h <= to_h,
        "from height {from_h} is higher than to height {to_h}"
    );

    let off_w = align_x.offset(from_w, to_w);
    let off_h = align_y.offset(from_h, to_h);

    let cols = {
        let width = image.width();

        let div = width / from_w;
        let rem = width % from_w;
        if rem != 0 {
            bail!("input image width {width} is not divisible by from tile width {from_w}")
        }
        div
    };
    let rows = {
        let height = image.height();

        let div = height / from_h;
        let rem = height % from_h;
        if rem != 0 {
            bail!("input image height {height} is not divisible by from tile height {from_h}")
        }

        div
    };

    let width = to_w * cols;
    let height = to_h * rows;

    let mut output = DynamicImage::new(width, height, image.color());

    let tiles = (0..rows).flat_map(move |y| (0..cols).map(move |x| (x, y)));

    let mut text_config = if let Some(index) = index_start {
        Some(TextState {
            index,
            divisor: const {
                if let Some(value) = NonZero::new(1) {
                    value
                } else {
                    unreachable!()
                }
            },
            font: ab_glyph::FontRef::try_from_slice(include_bytes!(
                "../font/NotoSansMono-Bold.ttf"
            ))?,
            color: ::image::Rgba([0, 0, 0, 255]),
        })
    } else {
        None
    };

    for (x, y) in tiles {
        let from_x = x * from_w;
        let from_y = y * from_h;

        let to_x = x * to_w + off_w;
        let to_y = y * to_h + off_h;

        let view = image.view(from_x, from_y, from_w, from_h);
        let view_img = view.to_image();

        output.copy_from(&view_img, to_x, to_y)?;

        if let Some(TextState {
            index,
            font,
            color,
            divisor,
        }) = &mut text_config
        {
            let text = index.to_string();
            let (text_w, text_h, scale) = 'brk: {
                for d in divisor.get()..1024 {
                    let scale = to_w as f32 / d as f32;
                    let (text_w, text_h) = text_size(scale, font, &text);

                    if text_w <= to_w / 2 && text_h <= to_h / 2 {
                        *divisor = NonZero::new(d).unwrap_or_else(|| {
                            unreachable!("first value of d is from a range starting with a nonzero")
                        });
                        break 'brk (text_w, text_h, scale);
                    }
                }
                bail!("maximum scale division reached for text (1024)");
            };

            let text_x = align_x_text.offset(text_w, to_w);
            let text_y = align_y_text.offset(text_h, to_h);

            let mut view = output.sub_image(x * to_w, y * to_h, to_w, to_h);
            draw_text_mut(
                view.deref_mut(),
                *color,
                text_x.try_into()?,
                text_y.try_into()?,
                scale,
                font,
                &index.to_string(),
            );
            *index += 1;
        }
    }

    Ok(output)
}
