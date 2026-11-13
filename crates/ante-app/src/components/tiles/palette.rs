use super::*;

pub const RED: Color = Color::srgb(
    0xFE as f32 / 255.0,
    0x5F as f32 / 255.0,
    0x55 as f32 / 255.0,
);

pub const BLUE: Color = Color::srgb(
    0x00 as f32 / 255.0,
    0x9D as f32 / 255.0,
    0xFF as f32 / 255.0,
);

pub const ORANGE: Color = Color::srgb(
    0xFD as f32 / 255.0,
    0xA2 as f32 / 255.0,
    0x00 as f32 / 255.0,
);

pub const PALE_GREEN: Color = Color::srgb(
    0x56 as f32 / 255.0,
    0xA8 as f32 / 255.0,
    0x87 as f32 / 255.0,
);

pub const L_BLACK: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x63 as f32 / 255.0,
    0x67 as f32 / 255.0,
);

/// `G.C.GREEN`, used by Sell, Redeem, Open and Reroll.
pub const GREEN: Color = Color::srgb(
    0x4B as f32 / 255.0,
    0xC2 as f32 / 255.0,
    0x92 as f32 / 255.0,
);

/// `G.C.UI.BACKGROUND_INACTIVE`.
pub const INACTIVE: Color = Color::srgb(0.4, 0.4, 0.4);

/// `G.C.MONEY`, the colour of prices.
pub const MONEY: Color = Color::srgb(
    0xF3 as f32 / 255.0,
    0xB9 as f32 / 255.0,
    0x58 as f32 / 255.0,
);

/// `G.C.BLACK` and `G.C.GOLD`.
pub const BLACK: Color = Color::srgb(
    0x37 as f32 / 255.0,
    0x42 as f32 / 255.0,
    0x44 as f32 / 255.0,
);

pub const GOLD: Color = Color::srgb(
    0xEA as f32 / 255.0,
    0xC0 as f32 / 255.0,
    0x58 as f32 / 255.0,
);

/// `G.C.UI.TEXT_INACTIVE`.
pub const TEXT_INACTIVE: Color = Color::srgba(
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x88 as f32 / 255.0,
    0x99 as f32 / 255.0,
);

/// `G.C.GREY`.
pub const GREY: Color = Color::srgb(
    0x5F as f32 / 255.0,
    0x73 as f32 / 255.0,
    0x77 as f32 / 255.0,
);

/// `G.C.IMPORTANT` and `G.C.UI.TRANSPARENT_DARK`.
pub const IMPORTANT: Color = Color::srgb(1.0, 0x9A as f32 / 255.0, 0.0);

pub const TRANSPARENT_DARK: Color = Color::srgba(
    0x22 as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x22 as f32 / 255.0,
    0x22 as f32 / 255.0,
);

/// `mix_colours(a, b, p)`: `p` of `a` and the rest of `b`.
pub fn mix(a: Color, b: Color, p: f32) -> Color {
    let (a, b) = (a.to_srgba(), b.to_srgba());
    let m = |x: f32, y: f32| x * p + y * (1.0 - p);
    Color::srgba(
        m(a.red, b.red),
        m(a.green, b.green),
        m(a.blue, b.blue),
        m(a.alpha, b.alpha),
    )
}

/// `darken(colour, amount)`.
pub fn darken(color: Color, amount: f32) -> Color {
    shade(color, -amount)
}

/// `lighten(colour, amount)`.
pub fn lighten(color: Color, amount: f32) -> Color {
    shade(color, amount)
}

/// Lighten (positive) or darken (negative) like the original `lighten`/`darken`.
pub fn shade(color: Color, amount: f32) -> Color {
    let c = color.to_srgba();
    let mix = |v: f32| {
        if amount >= 0.0 {
            v + (1.0 - v) * amount
        } else {
            v * (1.0 + amount)
        }
    };
    Color::srgba(mix(c.red), mix(c.green), mix(c.blue), c.alpha)
}

pub(super) fn scale_rgb(color: Color, factor: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgba(
        srgba.red * factor,
        srgba.green * factor,
        srgba.blue * factor,
        srgba.alpha,
    )
}
