use printpdf::*;

pub struct LayoutSettings {
    margin_x: Pt,
    margin_y: Pt,
    card_rows: usize,
    card_columns: usize,
    page_size: PaperSize,
    card_size: CardSize,
    dpi: f32,
}

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
        let l = self;
        let col = i % l.card_columns();
        let row = i / l.card_rows();

        let translate_x = (l.card_width() * (col as f32)) + l.margin_x();
        let translate_y = (l.card_height() * (row as f32)) + l.margin_y();
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
    }
}

impl LayoutSettings {
    pub fn new(paper: PaperSize, card: CardSize) -> Self {
        let dpi = 300.0;

        let page_width = paper.width().into_pt();
        let page_height = paper.height().into_pt();

        let card_width = card.width().into_pt();
        let card_height = card.height().into_pt();
        let card_columns = (page_width.into_px(dpi).0 / card_width.into_px(dpi).0);
        let margin_x = Pt((page_width.0 - (card_columns as f32 * card_width.0)) / 2.0);

        let card_rows = (page_height.into_px(dpi).0 / card_height.into_px(dpi).0);
        let margin_y = Pt((page_height.0 - (card_rows as f32 * card_height.0)) / 2.0);

        LayoutSettings {
            margin_x,
            margin_y,
            card_columns: card_columns as usize,
            card_rows: card_rows as usize,
            page_size: paper,
            card_size: card,
            dpi: dpi,
        }
    }
}

trait HasSize {
    fn height(&self) -> Mm;
    fn width(&self) -> Mm;
}

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
