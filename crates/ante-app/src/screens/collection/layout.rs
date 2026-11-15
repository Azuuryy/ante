//! Page layouts of the original `create_UIBox_your_collection_*` screens: the card areas of
//! each page (width, height, `card_limit`, type) and how many items a page shows.

use crate::components::card_area::AreaKind;
use ante_core::contract::ContentGroup;

/// One `CardArea` of a collection page, in room units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RowSpec {
    pub(crate) kind: AreaKind,
    /// `T.w` in card widths.
    pub(crate) width: f32,
    /// `T.h` in card heights.
    pub(crate) height: f32,
    pub(crate) limit: usize,
    /// Cards placed in the row.
    pub(crate) count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PageLayout {
    pub(crate) rows: Vec<RowSpec>,
    /// Card size in card units (1.27 for boosters).
    pub(crate) card_scale: f32,
    /// Padding around each row (0.07 for Jokers).
    pub(crate) row_padding: f32,
    /// Whether the page cycle is drawn below the cards.
    pub(crate) cycle: bool,
}

impl PageLayout {
    pub(crate) fn per_page(&self) -> usize {
        self.rows.iter().map(|row| row.count).sum()
    }

    /// Pages as the original counts them: `ceil` for most pages, `floor` for Tarots and
    /// Spectrals (22 / 11 and 18 / 9 divide exactly).
    pub(crate) fn pages(&self, items: usize) -> usize {
        items.div_ceil(self.per_page().max(1)).max(1)
    }
}

fn title(width: f32, height: f32, limit: usize) -> RowSpec {
    RowSpec {
        kind: AreaKind::Title,
        width,
        height,
        limit,
        count: limit,
    }
}

/// Layout of a card page; `None` for Tags and Blinds, which are grids of sprites.
pub(crate) fn page_layout(group: ContentGroup) -> Option<PageLayout> {
    use ContentGroup::*;
    let (rows, card_scale, row_padding, cycle) = match group {
        Jokers => (vec![title(5.0, 0.95, 5); 3], 1.0, 0.07, true),
        Tarots => (
            vec![title(5.25, 1.0, 5), title(6.25, 1.0, 6)],
            1.0,
            0.0,
            true,
        ),
        Planets => (vec![title(6.25, 1.0, 6); 2], 1.0, 0.0, false),
        Spectrals => (
            vec![title(4.25, 1.0, 4), title(5.25, 1.0, 5)],
            1.0,
            0.0,
            true,
        ),
        Boosters => (vec![title(5.25, 1.3, 4); 2], 1.27, 0.0, true),
        Vouchers => (
            vec![
                RowSpec {
                    kind: AreaKind::Voucher,
                    ..title(4.25, 1.0, 4)
                };
                2
            ],
            1.0,
            0.0,
            true,
        ),
        Enhancements => (vec![title(4.25, 1.03, 4); 2], 1.0, 0.0, false),
        Seals => (vec![title(4.25, 1.03, 4)], 1.0, 0.0, false),
        Editions => (vec![title(5.3, 1.03, 5)], 1.0, 0.0, false),
        Tags | Blinds => return None,
    };
    Some(PageLayout {
        rows,
        card_scale,
        row_padding,
        cycle,
    })
}

/// `get_blind_amount` at the base scaling: the eight listed amounts, then
/// `floor(a·(b + (k·c)^d)^c)` rounded down to its two leading digits.
pub(crate) fn blind_amount(ante: usize) -> f64 {
    const AMOUNTS: [f64; 8] = [
        300.0, 800.0, 2000.0, 5000.0, 11000.0, 20000.0, 35000.0, 50000.0,
    ];
    match ante {
        0 => 100.0,
        1..=8 => AMOUNTS[ante - 1],
        _ => {
            let k = 0.75_f64;
            let c = (ante - 8) as f64;
            let d = 1.0 + 0.2 * c;
            let amount = (AMOUNTS[7] * (1.6 + (k * c).powf(d)).powf(c)).floor();
            amount - amount % 10_f64.powf((amount.log10() - 1.0).floor())
        }
    }
}

/// `number_format`: digits grouped by commas, scientific notation from 1e11.
pub(crate) fn number_format(value: f64) -> String {
    if value >= 1e11 {
        let exponent = value.log10().floor();
        let mantissa = value / 10_f64.powf(exponent);
        return format!("{mantissa:.3}e{exponent}");
    }
    let digits = format!("{}", value.floor() as u64);
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}
