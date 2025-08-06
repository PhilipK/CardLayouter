mod layout;
mod pdf_generation;

use js_sys::{Array, Uint8Array};

use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::pdf_generation::generate_from_bytes;

#[wasm_bindgen]
pub fn generate_pdf(pngs: Array, back: JsValue, paper_size: u32, card_size: u32) -> Uint8Array {
    console::log_1(&format!("JS passed {} buffers", pngs.length()).into());

    let mut image_bytes = vec![];
    for (i, entry) in pngs.iter().enumerate() {
        let u8arr = Uint8Array::new(&entry);
        console::log_1(&format!("Buffer #{} length = {}", i, u8arr.length()).into());
        let mut buf = vec![0; u8arr.length() as usize];
        u8arr.copy_to(&mut buf);
        image_bytes.push(buf);
    }

    let back = if back.is_undefined() {
        None
    } else {
        let u8arr = Uint8Array::new(&back);
        let mut buf = vec![0; u8arr.length() as usize];
        u8arr.copy_to(&mut buf);
        Some(buf)
    };

    let paper_size = match paper_size {
        0 => layout::PaperSize::A4,
        _ => layout::PaperSize::Letter,
    };

    let card_size = match card_size {
        0 => layout::CardSize::Tcg,
        _ => layout::CardSize::Tarrot,
    };

    let bytes = generate_from_bytes(image_bytes, back, paper_size, card_size);

    console::log_1(&format!("Final PDF size: {} bytes", bytes.len()).into());
    Uint8Array::from(bytes.as_slice())
}
