//! Card positions of the original `CardArea:align_cards`, one place for every area type
//! Ante draws: the hand, the play area, title rows (deck view, collection), Jokers,
//! consumables and voucher stacks. Distances are in room units from the area's left edge;
//! `drop` is downward from the area's vertical centre. The time sway of the original
//! (`0.02 sin(2t + x)`, `0.03 sin(0.666t + x)`) is applied by the card's hover motion.

/// `config.type` of a card area. Title, Joker and Voucher rows are laid out by the deck
/// view and the collection; the inventories still use `tiles::spread_x`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum AreaKind {
    Hand,
    Play,
    Title,
    Joker,
    Consumeable,
    Voucher,
}

/// A card area: its type, width (`T.w`), slot width (`card_w`) and `card_limit`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardArea {
    pub(crate) kind: AreaKind,
    pub(crate) width: f32,
    pub(crate) card_w: f32,
    pub(crate) limit: usize,
}

/// Where one card rests in its area.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Slot {
    /// Left edge of the card.
    pub(crate) x: f32,
    /// Downward offset of the fan, before the highlight lift.
    pub(crate) drop: f32,
    /// Resting rotation in radians, before the sway.
    pub(crate) rotation: f32,
}

impl CardArea {
    pub(crate) fn new(kind: AreaKind, width: f32, card_w: f32, limit: usize) -> Self {
        Self {
            kind,
            width,
            card_w,
            limit,
        }
    }

    /// Slot of card `index` (0-based) among `count` cards drawn `card_width` wide.
    pub(crate) fn slot(&self, index: usize, count: usize, card_width: f32) -> Slot {
        let n = count.max(1) as f32;
        let k = index as f32 + 1.0;
        let centre = 0.5 * (self.card_w - card_width);
        // `(T.w - card_w) * ((k-1)/max(m-1,1) - 0.5*(n-m)/max(m-1,1))`, m = max(n, limit).
        let spread = |width: f32| {
            let m = count.max(self.limit).max(1) as f32;
            let span = (m - 1.0).max(1.0);
            (width - self.card_w) * ((k - 1.0) / span - 0.5 * (n - m) / span)
        };
        let fan = |turn: f32| Slot {
            x: spread(self.width) + centre,
            drop: (0.5 * (-n / 2.0 + k - 0.5) / n).abs() - if count > 1 { 0.2 } else { 0.0 },
            rotation: turn * (-n / 2.0 - 0.5 + k) / n,
        };
        match self.kind {
            AreaKind::Hand => Slot {
                drop: (0.5 * (-n / 2.0 + k - 0.5) / n).abs(),
                ..fan(0.2)
            },
            AreaKind::Title => fan(0.2),
            AreaKind::Play => Slot {
                // A one-card area centres its card.
                x: spread(self.width)
                    + centre
                    + if self.limit == 1 {
                        0.5 * (self.width - card_width)
                    } else {
                        0.0
                    },
                ..Slot::default()
            },
            AreaKind::Joker => Slot {
                x: if count > 2 {
                    (self.width - self.card_w) * (k - 1.0) / (n - 1.0) + centre
                } else if count > 1 {
                    (self.width - self.card_w) * (k - 0.5) / n + centre
                } else {
                    self.width / 2.0 - self.card_w / 2.0 + centre
                },
                drop: 0.0,
                rotation: 0.1 * (-n / 2.0 - 0.5 + k) / n,
            },
            AreaKind::Consumeable => Slot {
                x: if count > 1 {
                    (self.width - self.card_w) * (k - 1.0) / (n - 1.0) + centre
                } else {
                    self.width / 2.0 - self.card_w / 2.0 + centre
                },
                ..Slot::default()
            },
            AreaKind::Voucher if count > 1 => {
                // Pairs of a voucher and its upgrade overlap, alternating sides.
                let wide = self.width.max(3.2);
                let odd = index.is_multiple_of(2);
                Slot {
                    x: spread(wide)
                        + centre
                        + if odd { 0.27 } else { -0.27 }
                        + (self.width - wide) / 2.0,
                    rotation: 0.2 * (-n / 2.0 - 0.5 + k) / n + if odd { -0.08 } else { 0.08 },
                    ..fan(0.2)
                }
            }
            // A single voucher aligns like a title row.
            AreaKind::Voucher => fan(0.2),
        }
    }
}

#[cfg(test)]
mod tests;
