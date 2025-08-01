use std::path::{Path, PathBuf};

use printpdf::*;

fn main() {
    let mut doc = PdfDocument::new("Cards");
    let image_paths = get_all_pngs(Path::new("/home/philip/Pictures/DreadDomain"));

    let mut pages = vec![];
    // Create operations for our page
    let mut page_ops = vec![];

    let mut loaded_images = vec![];

    for image_path in image_paths {
        if let Ok(data) = std::fs::read(image_path) {
            if let Ok(image) = RawImage::decode_from_bytes(&data, &mut Vec::new()) {
                loaded_images.push(image);
            }
        }
    }
    let layout = LayoutSettings::default();
    let l = &layout;
    let dpi = 300.0;

    let target_width_pixels = l.card_width.into_px(dpi);

    let target_height_pixels = l.card_height.into_px(dpi);

    let mut cards_on_page = 0;
    for loaded_image in loaded_images.iter() {
        let scale_x = target_width_pixels.0 as f32 / loaded_image.width as f32;
        let scale_y = target_height_pixels.0 as f32 / loaded_image.height as f32;
        let col = cards_on_page % l.card_columns;
        let row = cards_on_page / l.card_rows;
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
        cards_on_page += 1;
        if cards_on_page >= 9 {
            pages.push(page_ops);
            page_ops = vec![];
            cards_on_page = 0;
        }
    }

    let mut final_pages = vec![];
    for mut page_ops in pages {
        page_ops.extend_from_slice(&draw_lines(l));
        let page = PdfPage::new(Mm(210.0), Mm(297.0), page_ops);
        final_pages.push(page);
    }
    // Save the PDF to a file
    let bytes = doc
        .with_pages(final_pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    std::fs::write("./image_example.pdf", bytes).unwrap();
    println!("Created image_example.pdf");
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
            (l.margin_x + Pt(column as f32 * l.card_width.0), l.page_height - l.margin_y),
            (l.margin_x + Pt(column as f32 * l.card_width.0), l.page_height ),
        ));
    }
    for row in 0..=l.card_rows {
        lines.push((
            (zero, l.margin_y + Pt(row as f32 * l.card_height.0)),
            (l.margin_x, l.margin_y + Pt(row as f32 * l.card_height.0)),
        ));
        lines.push((
            (l.page_width, l.margin_y + Pt(row as f32 * l.card_height.0)),
            (l.page_width-l.margin_x, l.margin_y + Pt(row as f32 * l.card_height.0)),
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

fn get_all_pngs(folder_path: &Path) -> Vec<PathBuf> {
    let mut pngs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "png" {
                    pngs.push(entry.path().to_path_buf());
                }
            }
        }
    }
    pngs
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
