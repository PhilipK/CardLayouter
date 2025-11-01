use std::fmt;

use printpdf::*;

use crate::layout::{CardSize, LayoutError, LayoutSettings, PaperSize};

#[derive(Debug)]
pub enum GenerateError {
    NoImages,
    Layout(LayoutError),
    MissingImage { name: String },
    DecodeFront { name: String, reason: String },
    DecodeBack { reason: String },
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerateError::NoImages => write!(f, "No front card images were provided."),
            GenerateError::Layout(err) => write!(f, "{}", err),
            GenerateError::MissingImage { name } => write!(
                f,
                "Could not find image bytes for '{name}' after sorting. Please retry the upload."
            ),
            GenerateError::DecodeFront { name, reason } => {
                write!(f, "Failed to decode front image '{name}': {reason}")
            }
            GenerateError::DecodeBack { reason } => {
                write!(f, "Failed to decode back image: {reason}")
            }
        }
    }
}

impl std::error::Error for GenerateError {}

fn generate_back_page_mirrored(
    doc: &mut PdfDocument,
    layout: &LayoutSettings,
    back_image: &RawImage,
) -> PdfPage {
    let l = layout;
    let dpi = l.dpi();
    let mut page_ops = vec![];

    let image_id = doc.add_image(&back_image);
    let (scale_x, scale_y) = l.scale_card((back_image.width, back_image.height));

    for i in 0..l.card_columns() * l.card_rows() {
        let (x, y) = l.mirrored_card_position(i);
        // Place the same image again, but translated, rotated, and scaled
        page_ops.push(Op::UseXobject {
            id: image_id.clone(),
            transform: XObjectTransform {
                translate_x: Some(x),
                translate_y: Some(y),
                scale_x: Some(scale_x),
                scale_y: Some(scale_y),
                dpi: Some(dpi),
                rotate: None,
            },
        });
    }

    page_ops.extend_from_slice(&draw_lines(l));
    let page = PdfPage::new(l.page_width().into(), l.page_height().into(), page_ops);
    page
}

fn generate_pages(
    doc: &mut PdfDocument,
    loaded_images: Vec<RawImage>,
    layout: &LayoutSettings,
) -> Vec<PdfPage> {
    let mut pages = vec![];
    let l = layout;

    for chunk in loaded_images.chunks(l.card_columns() * l.card_rows()) {
        let mut page_ops = vec![];
        for (i, loaded_image) in chunk.iter().enumerate() {
            // Add the image to the document resources and get its ID
            let image_id = doc.add_image(&loaded_image);

            let (scale_x, scale_y) = l.scale_card((loaded_image.width, loaded_image.height));
            let (x, y) = l.card_position(i);

            // Place the same image again, but translated, rotated, and scaled
            page_ops.push(Op::UseXobject {
                id: image_id.clone(),
                transform: XObjectTransform {
                    translate_x: Some(x),
                    translate_y: Some(y),
                    scale_x: Some(scale_x),
                    scale_y: Some(scale_y),
                    dpi: Some(l.dpi()),
                    rotate: None,
                },
            });
        }
        pages.push(page_ops);
    }

    let mut final_pages = vec![];
    for mut page_ops in pages {
        page_ops.extend_from_slice(&draw_lines(l));
        let page = PdfPage::new(l.page_width().into(), l.page_height().into(), page_ops);
        final_pages.push(page);
    }
    final_pages
}

fn draw_lines(layout: &LayoutSettings) -> Vec<Op> {
    //Make this better later
    let zero = Mm(0.0).into_pt();
    let l = layout;

    let mut lines = vec![];

    for column in 0..=l.card_columns() {
        lines.push((
            (
                l.margin_x() + Pt(column as f32 * l.card_width().0),
                l.margin_y(),
            ),
            (l.margin_x() + Pt(column as f32 * l.card_width().0), zero),
        ));
        lines.push((
            (
                l.margin_x() + Pt(column as f32 * l.card_width().0),
                l.page_height() - l.margin_y(),
            ),
            (
                l.margin_x() + Pt(column as f32 * l.card_width().0),
                l.page_height(),
            ),
        ));
    }
    for row in 0..=l.card_rows() {
        lines.push((
            (zero, l.margin_y() + Pt(row as f32 * l.card_height().0)),
            (
                l.margin_x(),
                l.margin_y() + Pt(row as f32 * l.card_height().0),
            ),
        ));
        lines.push((
            (
                l.page_width(),
                l.margin_y() + Pt(row as f32 * l.card_height().0),
            ),
            (
                l.page_width() - l.margin_x(),
                l.margin_y() + Pt(row as f32 * l.card_height().0),
            ),
        ));
    }

    let mut ops = vec![];

    for (p1, p2) in lines {
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point { x: p1.0, y: p1.1 },
                        bezier: false,
                    },
                    LinePoint {
                        p: Point { x: p2.0, y: p2.1 },
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
    }

    ops
}

pub fn generate_from_bytes(
    images_bytes: Vec<Vec<u8>>,
    back: Option<Vec<u8>>,
    paper_size: PaperSize,
    card_size: CardSize,
    file_names: Vec<String>,
    mirror_back: bool,
) -> Result<Vec<u8>, GenerateError> {
    // Sort by names first, then use indexes to decode images in that order
    let mut indexed_names: Vec<_> = file_names.into_iter().enumerate().collect();
    indexed_names.sort_by(|a, b| a.1.cmp(&b.1));

    let mut images = Vec::with_capacity(indexed_names.len());
    for (original_index, name) in indexed_names {
        let bytes = images_bytes
            .get(original_index)
            .ok_or_else(|| GenerateError::MissingImage { name: name.clone() })?;

        match RawImage::decode_from_bytes(bytes, &mut Vec::new()) {
            Ok(img) => images.push(img),
            Err(err) => {
                return Err(GenerateError::DecodeFront {
                    name,
                    reason: err.to_string(),
                });
            }
        }
    }

    if images.is_empty() {
        return Err(GenerateError::NoImages);
    }

    let layout = LayoutSettings::new(paper_size, card_size).map_err(GenerateError::Layout)?;

    let mut doc = PdfDocument::new("Cards");
    let mut pages = generate_pages(&mut doc, images, &layout);

    if let Some(img) = back {
        match RawImage::decode_from_bytes(&img, &mut Vec::new()) {
            Ok(img) => {
                let page = if mirror_back {
                    generate_back_page_mirrored(&mut doc, &layout, &img)
                } else {
                    generate_back_page_straight(&mut doc, &layout, &img)
                };
                pages.push(page);
            }
            Err(err) => {
                return Err(GenerateError::DecodeBack {
                    reason: err.to_string(),
                });
            }
        }
    }

    let bytes = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());
    Ok(bytes)
}

fn generate_back_page_straight(
    doc: &mut PdfDocument,
    layout: &LayoutSettings,
    back_image: &RawImage,
) -> PdfPage {
    let dpi = layout.dpi();
    let mut page_ops = vec![];
    let image_id = doc.add_image(back_image);
    let (scale_x, scale_y) = layout.scale_card((back_image.width, back_image.height));

    for i in 0..layout.card_columns() * layout.card_rows() {
        let (x, y) = layout.card_position(i);
        page_ops.push(Op::UseXobject {
            id: image_id.clone(),
            transform: XObjectTransform {
                translate_x: Some(x),
                translate_y: Some(y),
                scale_x: Some(scale_x),
                scale_y: Some(scale_y),
                dpi: Some(dpi),
                rotate: None,
            },
        });
    }

    page_ops.extend_from_slice(&draw_lines(layout));
    PdfPage::new(
        layout.page_width().into(),
        layout.page_height().into(),
        page_ops,
    )
}
