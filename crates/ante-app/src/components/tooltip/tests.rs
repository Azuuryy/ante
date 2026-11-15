use super::*;

#[test]
fn side_boxes_keep_their_order_without_repeats() {
    let tip = Tip::new("Wheel", "")
        .info(crate::components::info::references("c_wheel_of_fortune"))
        .info(crate::components::info::edition("Foil", false));
    let titles: Vec<_> = tip.side.iter().map(|info| info.title).collect();
    assert_eq!(titles, ["Foil", "Holographic", "Polychrome"]);
}

fn inks(line: &str) -> Vec<Ink> {
    let words: Vec<_> = line.split_whitespace().collect();
    let normalized: Vec<_> = words
        .iter()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '$')
                .to_ascii_lowercase()
        })
        .collect();
    words
        .iter()
        .enumerate()
        .map(|(index, word)| word_ink(word, &normalized[index], &normalized, index))
        .collect()
}

#[test]
fn chances_read_as_fractions() {
    assert_eq!(
        chance_fractions("1 in 4 chance to break"),
        "1/4 chance to break"
    );
    assert_eq!(chance_fractions("(one-in-two)"), "(1/2)");
    assert_eq!(
        chance_fractions("on a 1 in 15 chance;"),
        "on a 1/15 chance;"
    );
    assert_eq!(
        chance_fractions("9 in your full deck"),
        "9 in your full deck"
    );
    // One word, so a line can never break inside the fraction.
    let line = chance_fractions("1 in 5 chance");
    assert_eq!(line.split_whitespace().count(), 2);
    assert_eq!(inks(&line)[0], Ink::Chance);
    assert_eq!(inks("(1/2)")[0], Ink::Chance);
}

#[test]
fn chance_phrases_are_green() {
    assert_eq!(
        inks("1 in 5 chance"),
        vec![Ink::Chance, Ink::Chance, Ink::Chance, Ink::Chance]
    );
    assert_eq!(inks("one-in-two"), vec![Ink::Chance]);
}

#[test]
fn card_counts_are_not_mistaken_for_chances() {
    let ink = inks("9 in your full deck");
    assert_eq!(ink[0], Ink::Attention);
    assert!(!ink.contains(&Ink::Chance));
}

#[test]
fn money_and_scoring_values_use_balatro_colors() {
    assert_eq!(inks("Earn $4")[1], Ink::Money);
    assert_eq!(inks("+50 Chips")[0], Ink::Chips);
    assert_eq!(inks("+4 Mult")[0], Ink::Mult);
    assert_eq!(inks("×2")[0], Ink::Mult);
}

#[test]
fn ranks_and_counts_use_attention_orange() {
    assert_eq!(inks("Each Queen")[1], Ink::Attention);
    assert_eq!(inks("Discard 3 cards")[1], Ink::Attention);
}

#[test]
fn tooltip_follows_the_original_anchor_rule() {
    let size = Vec2::new(250.0, 150.0);
    let half = Vec2::new(60.0, 80.0);
    let window = Vec2::new(1460.0, 839.5);
    // Hand card near the bottom: centered above it.
    assert_eq!(
        position(Vec2::new(700.0, 700.0), half, size, window, TipAnchor::Auto),
        Vec2::new(575.0, 460.0)
    );
    // Joker at the top of the room: below it.
    assert_eq!(
        position(Vec2::new(700.0, 100.0), half, size, window, TipAnchor::Auto),
        Vec2::new(575.0, 190.0)
    );
    // Shop offer: left of it, flipped right at the left edge.
    assert_eq!(
        position(Vec2::new(700.0, 400.0), half, size, window, TipAnchor::Left),
        Vec2::new(380.0, 325.0)
    );
    assert_eq!(
        position(Vec2::new(100.0, 400.0), half, size, window, TipAnchor::Left),
        Vec2::new(170.0, 325.0)
    );
    // Always clamped inside the window.
    assert_eq!(
        position(Vec2::ZERO, half, size, Vec2::splat(100.0), TipAnchor::Auto),
        Vec2::ZERO
    );
}

#[test]
fn currently_lines_match_the_original_format() {
    assert_eq!(currently_text(CurrentValue::Mult(4)), "(Currently +4 Mult)");
    assert_eq!(
        currently_text(CurrentValue::Chips(30)),
        "(Currently +30 Chips)"
    );
    assert_eq!(
        currently_text(CurrentValue::XMult(150)),
        "(Currently X1.5 Mult)"
    );
    assert_eq!(
        currently_text(CurrentValue::XMult(125)),
        "(Currently X1.25 Mult)"
    );
    assert_eq!(
        currently_text(CurrentValue::XMult(300)),
        "(Currently X3 Mult)"
    );
    assert_eq!(currently_text(CurrentValue::Money(6)), "(Currently $6)");
    assert_eq!(currently_text(CurrentValue::Count(12)), "(Currently 12)");
    let line = currently_text(CurrentValue::Mult(4));
    assert_eq!(inks(&line), vec![Ink::Base, Ink::Mult, Ink::Base]);
}

#[test]
fn edition_badges_cycle_like_dark_edition() {
    // `game.lua`: R = 0.6 + 0.2 sin(1.3 t), B = 0.6 + 0.2 (1 - sin(1.3 t)), G = min(R, B).
    let start = dark_edition(0.0).to_srgba();
    assert!((start.red - 0.6).abs() < 1e-5 && (start.blue - 0.8).abs() < 1e-5);
    assert!((start.green - 0.6).abs() < 1e-5);
    let peak = dark_edition(std::f32::consts::FRAC_PI_2 / 1.3).to_srgba();
    assert!((peak.red - 0.8).abs() < 1e-5 && (peak.blue - 0.6).abs() < 1e-5);
    assert_ne!(badge_color("Foil"), Color::BLACK);
}

#[test]
fn card_sets_use_their_secondary_set_colours() {
    // `loc_colour`: tarot, planet and spectral are `G.C.SECONDARY_SET` colours.
    assert_eq!(named_color("tarot"), Some(TAROT));
    assert_eq!(named_color("planet"), Some(PLANET));
    assert_eq!(named_color("spectral"), Some(SPECTRAL));
    assert_eq!(named_color("chips"), None);
}

#[test]
fn the_shimmer_sweeps_each_letter_in_turn() {
    // The light enters from the left of a five-letter word and leaves on the right.
    let lit = |real: f32| (0..5).map(|i| shimmer_glow(i, 5, real)).collect::<Vec<_>>();
    let start = lit(0.0);
    assert!(start.iter().all(|&glow| glow == 0.0));
    let early = lit(0.6);
    let late = lit(1.6);
    let brightest = |glows: &[f32]| {
        (0..glows.len())
            .max_by(|&a, &b| glows[a].total_cmp(&glows[b]))
            .unwrap()
    };
    assert!(brightest(&early) < brightest(&late));
}
