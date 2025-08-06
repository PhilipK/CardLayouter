use printpdf::*;

use crate::layout::{CardSize, LayoutSettings, PaperSize};

fn generate_back_page(
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
        let (x, y) = l.card_position(i);
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

pub fn generate_from_bytes(images_bytes: Vec<Vec<u8>>, back: Option<Vec<u8>>, paper_size:PaperSize, card_size:CardSize) -> Vec<u8> {
    let mut images = vec![];
    for img in images_bytes {
        match RawImage::decode_from_bytes(&img, &mut Vec::new()) {
            Ok(img) => {
                images.push(img);
            }
            Err(_e) => {
                //console::warn_1(&format!("⚠️ Failed to decode #{}: {:?}", i, e).into());
            }
        }
    }
    let l = LayoutSettings::new(paper_size, card_size);

    let mut doc = PdfDocument::new("Cards");
    let mut pages = generate_pages(&mut doc, images, &l);

    if let Some(img) = back {
        match RawImage::decode_from_bytes(&img, &mut Vec::new()) {
            Ok(img) => {
                let page = generate_back_page(&mut doc, &l, &img);
                pages.push(page);
            }
            Err(_e) => {
                //console::warn_1(&format!("⚠️ Failed to decode #{}: {:?}", i, e).into());
            }
        }
    }

    let bytes = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());
    bytes
}
