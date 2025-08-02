use js_sys::{Array, Uint8Array};
use printpdf::*;

fn generate_back_page(
    doc: &mut PdfDocument,
    layout: &LayoutSettings,
    back_image: &RawImage,
) -> PdfPage {
    let l = layout;
    let dpi = 300.0;
    let mut page_ops = vec![];

    let target_width_pixels = l.card_width.into_px(dpi);
    let target_height_pixels = l.card_height.into_px(dpi);
    let scale_x = target_width_pixels.0 as f32 / back_image.width as f32;
    let scale_y = target_height_pixels.0 as f32 / back_image.height as f32;

    let image_id = doc.add_image(&back_image);

    for i in 0..l.card_columns * l.card_rows {
        let col = i % l.card_columns;
        let row = i / l.card_rows;
        // Add the image to the document resources and get its ID

        // Place the same image again, but translated, rotated, and scaled
        page_ops.push(Op::UseXobject {
            id: image_id.clone(),
            transform: XObjectTransform {
                translate_x: Some((l.card_width * (col as f32)) + l.margin_x),
                translate_y: Some((l.card_height * (row as f32)) + l.margin_y),
                scale_x: Some(scale_x),
                scale_y: Some(scale_y),
                dpi: Some(dpi),
                rotate: None,
            },
        });
    }

    page_ops.extend_from_slice(&draw_lines(l));
    let page = PdfPage::new(l.page_width.into(), l.page_height.into(), page_ops);
    page
}

fn generate_pages(
    doc: &mut PdfDocument,
    loaded_images: Vec<RawImage>,
    layout: &LayoutSettings,
) -> Vec<PdfPage> {
    let mut pages = vec![];
    let l = layout;
    let dpi = 300.0;

    let target_width_pixels = l.card_width.into_px(dpi);

    let target_height_pixels = l.card_height.into_px(dpi);

    for chunk in loaded_images.chunks(l.card_columns * l.card_rows) {
        let mut page_ops = vec![];
        for (i, loaded_image) in chunk.iter().enumerate() {
            let scale_x = target_width_pixels.0 as f32 / loaded_image.width as f32;
            let scale_y = target_height_pixels.0 as f32 / loaded_image.height as f32;
            let col = i % l.card_columns;
            let row = i / l.card_rows;
            // Add the image to the document resources and get its ID
            let image_id = doc.add_image(&loaded_image);

            // Place the same image again, but translated, rotated, and scaled
            page_ops.push(Op::UseXobject {
                id: image_id.clone(),
                transform: XObjectTransform {
                    translate_x: Some((l.card_width * (col as f32)) + l.margin_x),
                    translate_y: Some((l.card_height * (row as f32)) + l.margin_y),
                    scale_x: Some(scale_x),
                    scale_y: Some(scale_y),
                    dpi: Some(dpi),
                    rotate: None,
                },
            });
        }
        pages.push(page_ops);
    }

    let mut final_pages = vec![];
    for mut page_ops in pages {
        page_ops.extend_from_slice(&draw_lines(l));
        let page = PdfPage::new(l.page_width.into(), l.page_height.into(), page_ops);
        final_pages.push(page);
    }
    final_pages
}

fn draw_lines(layout: &LayoutSettings) -> Vec<Op> {
    //Make this better later
    let zero = Mm(0.0).into_pt();
    let l = layout;

    let mut lines = vec![];

    for column in 0..=l.card_columns {
        lines.push((
            (l.margin_x + Pt(column as f32 * l.card_width.0), l.margin_y),
            (l.margin_x + Pt(column as f32 * l.card_width.0), zero),
        ));
        lines.push((
            (
                l.margin_x + Pt(column as f32 * l.card_width.0),
                l.page_height - l.margin_y,
            ),
            (
                l.margin_x + Pt(column as f32 * l.card_width.0),
                l.page_height,
            ),
        ));
    }
    for row in 0..=l.card_rows {
        lines.push((
            (zero, l.margin_y + Pt(row as f32 * l.card_height.0)),
            (l.margin_x, l.margin_y + Pt(row as f32 * l.card_height.0)),
        ));
        lines.push((
            (l.page_width, l.margin_y + Pt(row as f32 * l.card_height.0)),
            (
                l.page_width - l.margin_x,
                l.margin_y + Pt(row as f32 * l.card_height.0),
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

struct LayoutSettings {
    card_width: Pt,
    card_height: Pt,
    page_width: Pt,
    page_height: Pt,
    margin_x: Pt,
    margin_y: Pt,
    card_rows: usize,
    card_columns: usize,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        let card_width = Mm(63.0).into_pt();
        let card_height = Mm(88.0).into_pt();
        let page_width = Mm(210.0).into_pt();
        let page_height = Mm(297.0).into_pt();
        let card_columns = (page_width.0 / card_width.0).floor();
        let margin_x = Pt((page_width.0 - (card_columns * card_width.0)) / 2.0);

        let card_rows = (page_height.0 / card_height.0).floor();
        let margin_y = Pt((page_height.0 - (card_rows * card_height.0)) / 2.0);

        LayoutSettings {
            card_width,
            card_height,
            page_width,
            page_height,
            margin_x,
            margin_y,
            card_columns: card_columns as usize,
            card_rows: card_rows as usize,
        }
    }
}

use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn generate_pdf(pngs: Array) -> Uint8Array {
    console::log_1(&format!("JS passed {} buffers", pngs.length()).into());
    let mut images = Vec::new();

    for (i, entry) in pngs.iter().enumerate() {
        let u8arr = Uint8Array::new(&entry);
        console::log_1(&format!("Buffer #{} length = {}", i, u8arr.length()).into());
        let mut buf = vec![0; u8arr.length() as usize];
        u8arr.copy_to(&mut buf);
        match RawImage::decode_from_bytes(&buf, &mut Vec::new()) {
            Ok(img) => {
                console::log_1(&format!("✅ Decoded image #{}", i).into());
                images.push(img);
            }
            Err(e) => {
                console::warn_1(&format!("⚠️ Failed to decode #{}: {:?}", i, e).into());
            }
        }
    }

    console::log_1(&format!("Decoded {} images", images.len()).into());
    let mut doc = PdfDocument::new("Cards");
    let pages = generate_pages(&mut doc, images, &LayoutSettings::default());
    console::log_1(&format!("Generating {} pages", pages.len()).into());

    let bytes = doc.with_pages(pages)
                   .save(&PdfSaveOptions::default(), &mut Vec::new());
    console::log_1(&format!("Final PDF size: {} bytes", bytes.len()).into());
    Uint8Array::from(bytes.as_slice())
}