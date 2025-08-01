use std::path::{Path, PathBuf};

use printpdf::*;

fn main() {
    let mut doc = PdfDocument::new("Cards");
    let image_paths = get_all_pngs(Path::new("/home/philip/Pictures/DreadDomain"));

    // Create operations for our page
    let mut ops = Vec::new();

    let mut loaded_images = vec![];

    for image_path in image_paths {
        if let Ok(data) = std::fs::read(image_path) {
            if let Ok(image) = RawImage::decode_from_bytes(&data, &mut Vec::new()) {
                loaded_images.push(image);
            }
        }
    }
    let dpi = 300.0;

    let margin = Mm(10.0).into_pt();
    let card_width = Mm(63.0).into_pt();
    let card_height = Mm(88.0).into_pt();

    let target_width_pixels = card_width.into_px(dpi);

    let target_height_pixels = card_height.into_px(dpi);

    for (i, loaded_image) in loaded_images.iter().enumerate() {
        let scale_x = target_width_pixels.0 as f32 / loaded_image.width as f32;
        let scale_y = target_height_pixels.0 as f32 / loaded_image.height as f32;
        let col = i / 3;
        let row = i % 3;
        // Add the image to the document resources and get its ID
        let image_id = doc.add_image(&loaded_image);

        // Place the same image again, but translated, rotated, and scaled
        ops.push(Op::UseXobject {
            id: image_id.clone(),
            transform: XObjectTransform {
                translate_x: Some((card_width * (col as f32)) + margin),
                translate_y: Some((card_height * (row as f32)) + margin),
                scale_x: Some(scale_x),
                scale_y: Some(scale_y),
                dpi: Some(dpi),
                rotate: None,
            },
        });
    }
    // Create a page with our operations
    let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);

    // Save the PDF to a file
    let bytes = doc
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    std::fs::write("./image_example.pdf", bytes).unwrap();
    println!("Created image_example.pdf");
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
