use super::*;

pub(super) const MULT: Color = Color::srgb(
    0xFE as f32 / 255.0,
    0x5F as f32 / 255.0,
    0x55 as f32 / 255.0,
);

pub(super) const CHIPS: Color = Color::srgb(
    0x00 as f32 / 255.0,
    0x9D as f32 / 255.0,
    0xFF as f32 / 255.0,
);

pub(super) const MONEY: Color = Color::srgb(
    0xF3 as f32 / 255.0,
    0xB9 as f32 / 255.0,
    0x58 as f32 / 255.0,
);

pub(super) const CHANCE: Color = Color::srgb(
    0x4B as f32 / 255.0,
    0xC2 as f32 / 255.0,
    0x92 as f32 / 255.0,
);

pub(super) const ATTENTION: Color = Color::srgb(
    0xFF as f32 / 255.0,
    0x9A as f32 / 255.0,
    0x00 as f32 / 255.0,
);

pub(super) const RARITY_COMMON: Color = Color::srgb(
    0x00 as f32 / 255.0,
    0x9D as f32 / 255.0,
    0xFF as f32 / 255.0,
);

pub(super) const RARITY_UNCOMMON: Color = Color::srgb(
    0x4B as f32 / 255.0,
    0xC2 as f32 / 255.0,
    0x92 as f32 / 255.0,
);

pub(super) const RARITY_RARE: Color = Color::srgb(
    0xFE as f32 / 255.0,
    0x5F as f32 / 255.0,
    0x55 as f32 / 255.0,
);

pub(super) const RARITY_LEGENDARY: Color = Color::srgb(
    0xB2 as f32 / 255.0,
    0x6C as f32 / 255.0,
    0xBB as f32 / 255.0,
);

pub(crate) const TAROT: Color = Color::srgb(
    0xA7 as f32 / 255.0,
    0x82 as f32 / 255.0,
    0xD1 as f32 / 255.0,
);

pub(crate) const PLANET: Color = Color::srgb(
    0x13 as f32 / 255.0,
    0xAF as f32 / 255.0,
    0xCE as f32 / 255.0,
);

pub(crate) const SPECTRAL: Color = Color::srgb(
    0x45 as f32 / 255.0,
    0x84 as f32 / 255.0,
    0xFA as f32 / 255.0,
);

pub(super) const VOUCHER: Color = Color::srgb(
    0xFD as f32 / 255.0,
    0x68 as f32 / 255.0,
    0x2B as f32 / 255.0,
);

pub(super) const BOOSTER: Color = Color::srgb(
    0x64 as f32 / 255.0,
    0x6E as f32 / 255.0,
    0xB7 as f32 / 255.0,
);

pub(super) const EDITION: Color = Color::srgb(
    0x4C as f32 / 255.0,
    0xA8 as f32 / 255.0,
    0x93 as f32 / 255.0,
);

/// `G.C.DARK_EDITION`, recomputed every frame by `Game:update` from real time: the badge
/// colour of Foil, Holographic, Polychrome and Negative cards.
pub(crate) fn dark_edition(real: f32) -> Color {
    let wave = (real * 1.3).sin();
    let red = 0.6 + 0.2 * wave;
    let blue = 0.6 + 0.2 * (1.0 - wave);
    Color::srgb(red, red.min(blue), blue)
}

/// An edition badge whose colour follows `dark_edition`.
#[derive(Component)]
pub(crate) struct DarkEditionBadge;

pub(crate) fn animate_edition_badges(
    time: Res<Time<Real>>,
    mut badges: Query<&mut PixelBox, With<DarkEditionBadge>>,
) {
    let colour = dark_edition(time.elapsed_secs());
    for mut badge in &mut badges {
        badge.fill = colour;
    }
}

pub(super) const LOCKED: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x56 as f32 / 255.0,
    0x58 as f32 / 255.0,
);

/// Chances read "1/4": "1 in 4" and "one-in-two" become one word, numerator over total,
/// so a wrapped line never splits the fraction.
pub(crate) fn chance_fractions(line: &str) -> String {
    let number = |word: &str| -> Option<String> {
        let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
        if !trimmed.is_empty() && trimmed.bytes().all(|c| c.is_ascii_digit()) {
            return Some(trimmed.to_owned());
        }
        let value = match trimmed.to_ascii_lowercase().as_str() {
            "one" => 1,
            "two" => 2,
            "three" => 3,
            "four" => 4,
            "five" => 5,
            "six" => 6,
            "seven" => 7,
            "eight" => 8,
            "nine" => 9,
            "ten" => 10,
            _ => return None,
        };
        Some(value.to_string())
    };
    let words: Vec<&str> = line.split_whitespace().collect();
    let mut out: Vec<String> = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let word = words[index];
        // "one-in-two" in a single token.
        if let Some((left, right)) = word.split_once("-in-")
            && let (Some(a), Some(b)) = (number(left), number(right))
        {
            let tail: String = right.chars().skip_while(|c| c.is_alphanumeric()).collect();
            let lead: String = left.chars().take_while(|c| !c.is_alphanumeric()).collect();
            out.push(format!("{lead}{a}/{b}{tail}"));
            index += 1;
            continue;
        }
        if index + 2 < words.len()
            && words[index + 1].eq_ignore_ascii_case("in")
            && let (Some(a), Some(b)) = (number(word), number(words[index + 2]))
        {
            let lead: String = word.chars().take_while(|c| !c.is_alphanumeric()).collect();
            let tail: String = words[index + 2]
                .chars()
                .skip_while(|c| !c.is_alphanumeric())
                .skip_while(|c| c.is_alphanumeric())
                .collect();
            out.push(format!("{lead}{a}/{b}{tail}"));
            index += 3;
            continue;
        }
        out.push(word.to_owned());
        index += 1;
    }
    out.join(" ")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Ink {
    Base,
    Mult,
    Chips,
    Money,
    Chance,
    Attention,
}

/// Suits and card sets keep their own colour in descriptions, like `{C:hearts}`,
/// `{C:tarot}`, `{C:planet}` and `{C:spectral}` in the original text (`loc_colour`).
pub(super) fn named_color(word: &str) -> Option<Color> {
    Some(match word {
        "heart" | "hearts" => Color::srgb(0.92, 0.06, 0.19),
        "diamond" | "diamonds" => Color::srgb(0.74, 0.32, 0.04),
        "club" | "clubs" => Color::srgb(0.00, 0.40, 0.27),
        "spade" | "spades" => Color::srgb(0.35, 0.20, 0.54),
        "tarot" | "tarots" => TAROT,
        "planet" | "planets" => PLANET,
        "spectral" | "spectrals" => SPECTRAL,
        _ => return None,
    })
}

pub(super) fn effect_line(ui: &mut ChildSpawnerCommands, line: &str, base: Color, size: f32) {
    let line = chance_fractions(line);
    let words: Vec<_> = line.split_whitespace().collect();
    let normalized: Vec<_> = words
        .iter()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '$')
                .to_ascii_lowercase()
        })
        .collect();
    ui.spawn((
        Node {
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(size * 0.3),
            row_gap: px(size * 0.1),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|row| {
        for (index, word) in words.into_iter().enumerate() {
            let word_key = &normalized[index];
            let multiplier = is_multiplier(&normalized, index, word);
            let color = named_color(word_key)
                .unwrap_or_else(|| ink_color(word_ink(word, word_key, &normalized, index), base));
            if multiplier {
                let explicit_times =
                    word_key.starts_with('x') || word_key.ends_with('x') || word.contains('×');
                // Trailing punctuation stays outside the box.
                let figure = word.trim_end_matches(|c: char| !c.is_alphanumeric());
                let trailing = &word[figure.len()..];
                let display = if explicit_times {
                    figure.to_owned()
                } else {
                    format!("×{figure}")
                };
                // `{X:mult}`: a box with `r = 0.05` corners, far smaller than the panels'
                // stepped corners, so it is drawn as a plain rectangle.
                row.spawn((
                    Node {
                        column_gap: px(0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|group| {
                    group
                        .spawn((
                            Node {
                                padding: UiRect::axes(tiles(0.04), tiles(0.01)),
                                ..default()
                            },
                            BackgroundColor(color),
                            Pickable::IGNORE,
                        ))
                        .with_children(|pill| label(pill, display, size, Color::WHITE));
                    if !trailing.is_empty() {
                        label(group, trailing, size, base);
                    }
                });
            } else if matches!(
                word_key.as_str(),
                "tarot" | "tarots" | "spectral" | "spectrals"
            ) {
                shimmer_word(row, word, size, color);
            } else {
                label(row, word, size, color);
            }
        }
    });
}

/// Seconds between two sweeps of the shimmer, and its half-width in letters.
const SHIMMER_PERIOD: f32 = 2.4;
const SHIMMER_WIDTH: f32 = 2.0;

/// A word whose letters catch a light sweeping from left to right. An Ante addition for
/// "Tarot" and "Spectral"; the original draws them in their plain set colour.
#[derive(Component)]
pub(crate) struct Shimmer {
    base: Color,
    len: usize,
}

#[derive(Component)]
pub(crate) struct ShimmerGlyph(usize);

/// Light on letter `index` of a `len`-letter word at real time `real`, from 0 to 1.
pub(super) fn shimmer_glow(index: usize, len: usize, real: f32) -> f32 {
    let travel = len as f32 + 2.0 * SHIMMER_WIDTH;
    let head = (real / SHIMMER_PERIOD).fract() * travel - SHIMMER_WIDTH;
    (1.0 - (index as f32 - head).abs() / SHIMMER_WIDTH).clamp(0.0, 1.0)
}

fn shimmer_word(ui: &mut ChildSpawnerCommands, word: &str, size: f32, color: Color) {
    let font = || TextFont {
        font_size: FontSize::Px(size),
        ..default()
    };
    ui.spawn((
        Text::new(""),
        font(),
        TextColor(color),
        Shimmer {
            base: color,
            len: word.chars().count(),
        },
        Pickable::IGNORE,
    ))
    .with_children(|text| {
        for (index, letter) in word.chars().enumerate() {
            text.spawn((
                TextSpan::new(letter.to_string()),
                font(),
                TextColor(color),
                ShimmerGlyph(index),
            ));
        }
    });
}

pub(crate) fn animate_shimmer(
    time: Res<Time<Real>>,
    words: Query<(&Shimmer, &Children)>,
    mut glyphs: Query<(&ShimmerGlyph, &mut TextColor)>,
) {
    let real = time.elapsed_secs();
    for (shimmer, children) in &words {
        let base = shimmer.base.to_srgba();
        for child in children {
            if let Ok((glyph, mut colour)) = glyphs.get_mut(*child) {
                let light = 0.6 * shimmer_glow(glyph.0, shimmer.len, real);
                let lit = Color::srgba(
                    base.red + (1.0 - base.red) * light,
                    base.green + (1.0 - base.green) * light,
                    base.blue + (1.0 - base.blue) * light,
                    base.alpha,
                );
                if colour.0 != lit {
                    colour.0 = lit;
                }
            }
        }
    }
}

pub(super) fn word_ink(word: &str, key: &str, normalized: &[String], index: usize) -> Ink {
    if word.contains('$') {
        return Ink::Money;
    }
    if key.contains("chance")
        || is_chance_token(normalized, index)
        || is_fraction(normalized, index)
    {
        return Ink::Chance;
    }
    let next = normalized.get(index + 1).map_or("", String::as_str);
    let previous = index.checked_sub(1).map_or("", |i| normalized[i].as_str());
    let previous_unit = index.checked_sub(2).and_then(|i| unit_ink(&normalized[i]));
    if numeric_word(word) {
        if let Some(unit) = unit_ink(next)
            .or_else(|| unit_ink(previous))
            .or(previous_unit)
        {
            return unit;
        }
        if is_multiplier(normalized, index, word) {
            return Ink::Mult;
        }
        return Ink::Attention;
    }
    if is_number_word(key) || is_rank_word(key) {
        return Ink::Attention;
    }
    Ink::Base
}

pub(super) fn is_multiplier(normalized: &[String], index: usize, word: &str) -> bool {
    let key = &normalized[index];
    let next = normalized.get(index + 1).map_or("", String::as_str);
    let previous = index.checked_sub(1).map_or("", |i| normalized[i].as_str());
    let previous_unit = index.checked_sub(2).and_then(|i| unit_ink(&normalized[i]));
    let explicit_times = key.starts_with('x') || key.ends_with('x') || word.contains('×');
    numeric_word(word)
        && (explicit_times || next == "multiplier" || (previous == "by" && previous_unit.is_some()))
}

pub(super) fn ink_color(ink: Ink, base: Color) -> Color {
    match ink {
        Ink::Base => base,
        Ink::Mult => MULT,
        Ink::Chips => CHIPS,
        Ink::Money => MONEY,
        Ink::Chance => CHANCE,
        Ink::Attention => ATTENTION,
    }
}

pub(super) fn unit_ink(word: &str) -> Option<Ink> {
    match word {
        "mult" => Some(Ink::Mult),
        "chip" | "chips" => Some(Ink::Chips),
        _ => None,
    }
}

pub(super) fn numeric_word(word: &str) -> bool {
    let value = word
        .trim_matches(|c: char| !c.is_alphanumeric())
        .trim_start_matches(['x', 'X'])
        .trim_end_matches(['x', 'X']);
    !value.is_empty()
        && value.bytes().any(|c| c.is_ascii_digit())
        && value.bytes().all(|c| c.is_ascii_digit() || c == b'.')
}

pub(super) fn is_number_word(word: &str) -> bool {
    matches!(
        word,
        "one"
            | "two"
            | "three"
            | "four"
            | "five"
            | "six"
            | "seven"
            | "eight"
            | "nine"
            | "ten"
            | "eleven"
            | "twelve"
            | "thirteen"
            | "fourteen"
            | "fifteen"
            | "twenty"
            | "thirty"
            | "forty"
            | "fifty"
            | "hundred"
            | "thousand"
    )
}

pub(super) fn is_rank_word(word: &str) -> bool {
    matches!(
        word,
        "ace" | "aces" | "jack" | "jacks" | "queen" | "queens" | "king" | "kings"
    )
}

/// "1 / 4": the two numbers and the slash of a chance fraction.
pub(super) fn is_fraction(normalized: &[String], index: usize) -> bool {
    let at = |i: usize| normalized.get(i).map(String::as_str);
    let digits = |word: Option<&str>| {
        word.is_some_and(|w| !w.is_empty() && w.bytes().all(|c| c.is_ascii_digit()))
    };
    let raw = |i: usize| at(i).is_some_and(str::is_empty);
    if let Some((numerator, total)) = at(index).and_then(|word| word.split_once('/'))
        && digits(Some(numerator))
        && digits(Some(total))
    {
        return true;
    }
    // The slash normalizes to an empty word.
    (raw(index) && index > 0 && digits(at(index - 1)) && digits(at(index + 1)))
        || (digits(at(index)) && raw(index + 1) && digits(at(index + 2)))
        || (index >= 2 && digits(at(index)) && raw(index - 1) && digits(at(index - 2)))
}

pub(super) fn is_chance_token(normalized: &[String], index: usize) -> bool {
    let key = &normalized[index];
    if key.contains("-in-") {
        let mut parts = key.split("-in-");
        if let (Some(left), Some(right), None) = (parts.next(), parts.next(), parts.next())
            && (is_number_word(left) || numeric_word(left))
            && (is_number_word(right) || numeric_word(right))
        {
            return true;
        }
    }
    let previous = index.checked_sub(1).map_or("", |i| normalized[i].as_str());
    let next = normalized.get(index + 1).map_or("", String::as_str);
    if key == "in" {
        return is_number_word(next) || numeric_word(next);
    }
    if is_number_word(key) || numeric_word(key) {
        if next == "in" {
            return normalized
                .get(index + 2)
                .is_some_and(|value| is_number_word(value) || numeric_word(value));
        }
        if previous == "in" && index >= 2 {
            let before = &normalized[index - 2];
            return is_number_word(before) || numeric_word(before);
        }
    }
    false
}
