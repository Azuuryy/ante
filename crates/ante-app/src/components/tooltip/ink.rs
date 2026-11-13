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

pub(super) const TAROT: Color = Color::srgb(
    0xA7 as f32 / 255.0,
    0x82 as f32 / 255.0,
    0xD1 as f32 / 255.0,
);

pub(super) const PLANET: Color = Color::srgb(
    0x13 as f32 / 255.0,
    0xAF as f32 / 255.0,
    0xCE as f32 / 255.0,
);

pub(super) const SPECTRAL: Color = Color::srgb(
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

pub(super) const DARK_EDITION: Color = Color::BLACK;

pub(super) const LOCKED: Color = Color::srgb(
    0x4F as f32 / 255.0,
    0x56 as f32 / 255.0,
    0x58 as f32 / 255.0,
);

/// Chances read "1 / 4": "1 in 4" and "one-in-two" become a numerator over a total.
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
            out.push(format!("{lead}{a} / {b}{tail}"));
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
            out.push(format!("{lead}{a} / {b}{tail}"));
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
            let color = match word_key.as_str() {
                "heart" | "hearts" => Color::srgb(0.92, 0.06, 0.19),
                "diamond" | "diamonds" => Color::srgb(0.74, 0.32, 0.04),
                "club" | "clubs" => Color::srgb(0.00, 0.40, 0.27),
                "spade" | "spades" => Color::srgb(0.35, 0.20, 0.54),
                "planet" | "planets" => Color::srgb(0.00, 0.37, 0.55),
                _ => ink_color(word_ink(word, word_key, &normalized, index), base),
            };
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
            } else {
                label(row, word, size, color);
            }
        }
    });
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
