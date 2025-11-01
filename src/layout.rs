use std::fmt;

use printpdf::*;

#[derive(Debug, Clone)]
pub struct LayoutSettings {
    margin_x: Pt,
    margin_y: Pt,
    card_rows: usize,
    card_columns: usize,
    page_size: PaperSize,
    card_size: CardSize,
    dpi: f32,
}

#[derive(Debug)]
pub enum LayoutError {
    NoColumns { paper: PaperSize, card: CardSize },
    NoRows { paper: PaperSize, card: CardSize },
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutError::NoColumns { paper, card } => write!(
                f,
                "Card width ({card}) exceeds available width on {paper} paper. Try a smaller card size or a larger paper."
            ),
            LayoutError::NoRows { paper, card } => write!(
                f,
                "Card height ({card}) exceeds available height on {paper} paper. Try a smaller card size or a larger paper."
            ),
        }
    }
}

impl std::error::Error for LayoutError {}

impl LayoutSettings {
    pub fn dpi(&self) -> f32 {
        self.dpi
    }
    pub fn margin_x(&self) -> Pt {
        self.margin_x
    }
    pub fn margin_y(&self) -> Pt {
        self.margin_y
    }
    pub fn card_rows(&self) -> usize {
        self.card_rows
    }
    pub fn card_columns(&self) -> usize {
        self.card_columns
    }
    pub fn card_width(&self) -> Pt {
        self.card_size.width().into_pt()
    }

    pub fn card_height(&self) -> Pt {
        self.card_size.height().into_pt()
    }

    pub fn page_width(&self) -> Pt {
        self.page_size.width().into_pt()
    }

    pub fn page_height(&self) -> Pt {
        self.page_size.height().into_pt()
    }
}

impl LayoutSettings {
    pub fn card_position(&self, i: usize) -> (Pt, Pt) {
        let (row_from_top, col) = self.card_row_col(i);
        let row_from_bottom = self.card_rows() - row_from_top - 1;

        let translate_x = (self.card_width() * (col as f32)) + self.margin_x();
        let translate_y = (self.card_height() * (row_from_bottom as f32)) + self.margin_y();
        (translate_x, translate_y)
    }

    pub fn mirrored_card_position(&self, i: usize) -> (Pt, Pt) {
        let (row_from_top, col) = self.card_row_col(i);
        let mirrored_col = self.card_columns() - col - 1;
        let row_from_bottom = self.card_rows() - row_from_top - 1;

        let translate_x = (self.card_width() * (mirrored_col as f32)) + self.margin_x();
        let translate_y = (self.card_height() * (row_from_bottom as f32)) + self.margin_y();
        (translate_x, translate_y)
    }

    pub fn scale_card(&self, cur_size_pixels: (usize, usize)) -> (f32, f32) {
        let l = self;
        let dpi = l.dpi();
        let target_width_pixels = l.card_width().into_px(dpi);
        let target_height_pixels = l.card_height().into_px(dpi);
        let scale_x = target_width_pixels.0 as f32 / cur_size_pixels.0 as f32;
        let scale_y = target_height_pixels.0 as f32 / cur_size_pixels.1 as f32;

        (scale_x, scale_y)
    }
}

impl Default for LayoutSettings {
    fn default() -> Self {
        LayoutSettings::new(PaperSize::A4, CardSize::Tcg)
            .expect("Preset sizes should always fit on the selected paper")
    }
}

impl LayoutSettings {
    pub fn new(paper: PaperSize, card: CardSize) -> Result<Self, LayoutError> {
        let dpi = 300.0;

        let page_width = paper.width().into_pt();
        let page_height = paper.height().into_pt();

        let card_width = card.width().into_pt();
        let card_height = card.height().into_pt();
        let page_width_px = page_width.into_px(dpi).0;
        let page_height_px = page_height.into_px(dpi).0;
        let card_width_px = card_width.into_px(dpi).0;
        let card_height_px = card_height.into_px(dpi).0;

        let card_columns = page_width_px / card_width_px;
        if card_columns == 0 {
            return Err(LayoutError::NoColumns { paper, card });
        }

        let card_rows = page_height_px / card_height_px;
        if card_rows == 0 {
            return Err(LayoutError::NoRows { paper, card });
        }

        let margin_x = Pt((page_width.0 - (card_columns as f32 * card_width.0)) / 2.0);
        let margin_y = Pt((page_height.0 - (card_rows as f32 * card_height.0)) / 2.0);

        Ok(LayoutSettings {
            margin_x,
            margin_y,
            card_columns,
            card_rows,
            page_size: paper,
            card_size: card,
            dpi,
        })
    }

    fn card_row_col(&self, index: usize) -> (usize, usize) {
        let cols = self.card_columns();
        (index / cols, index % cols)
    }
}

trait HasSize {
    fn height(&self) -> Mm;
    fn width(&self) -> Mm;
}

#[derive(Debug, Clone, Copy)]
pub enum PaperSize {
    Letter,
    A4,
}

impl HasSize for PaperSize {
    fn height(&self) -> Mm {
        match self {
            PaperSize::Letter => Mm(279.4),
            PaperSize::A4 => Mm(297.0),
        }
    }
    fn width(&self) -> Mm {
        match self {
            PaperSize::Letter => Mm(215.9),
            PaperSize::A4 => Mm(210.0),
        }
    }
}

impl Default for PaperSize {
    fn default() -> Self {
        PaperSize::A4
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CardSize {
    Tcg,
    Tarrot,
}

impl HasSize for CardSize {
    fn height(&self) -> Mm {
        match self {
            CardSize::Tcg => Mm(88.0),
            CardSize::Tarrot => Mm(120.0),
        }
    }

    fn width(&self) -> Mm {
        match self {
            CardSize::Tcg => Mm(63.0),
            CardSize::Tarrot => Mm(70.0),
        }
    }
}

impl fmt::Display for PaperSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaperSize::Letter => write!(f, "Letter"),
            PaperSize::A4 => write!(f, "A4"),
        }
    }
}

impl fmt::Display for CardSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardSize::Tcg => write!(f, "TCG (63mm × 88mm)"),
            CardSize::Tarrot => write!(f, "Tarrot (70mm × 120mm)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_positions_progress_across_columns_then_rows() {
        let layout = LayoutSettings::new(PaperSize::A4, CardSize::Tcg).unwrap();

        let (x0, y0) = layout.card_position(0);
        let (x1, y1) = layout.card_position(1);
        assert!(x1.0 > x0.0);
        assert!((y1.0 - y0.0).abs() < f32::EPSILON);

        let (x_col_reset, y_next_row) = layout.card_position(layout.card_columns());
        assert!((x_col_reset.0 - x0.0).abs() < f32::EPSILON);
        assert!(y_next_row.0 < y0.0);
    }

    #[test]
    fn mirrored_position_flips_columns() {
        let layout = LayoutSettings::new(PaperSize::A4, CardSize::Tcg).unwrap();
        let last_col_index = layout.card_columns() - 1;

        let (front_x, front_y) = layout.card_position(last_col_index);
        let (back_x, back_y) = layout.mirrored_card_position(0);

        assert!((front_y.0 - back_y.0).abs() < f32::EPSILON);
        assert!((front_x.0 - back_x.0).abs() < f32::EPSILON);
    }
}
