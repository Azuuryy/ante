use super::*;

pub(super) fn card_effect_summary(card: balatro_rs::card::Card) -> String {
    use balatro_rs::card::{Edition, Enhancement, Seal};
    let mut parts = Vec::new();
    match card.enhancement {
        Some(Enhancement::Bonus) => parts.push("+30 chips when scored"),
        Some(Enhancement::Mult) => parts.push("+4 Mult when scored"),
        Some(Enhancement::Wild) => parts.push("Counts as any suit"),
        Some(Enhancement::Glass) => parts.push("Doubles Mult when scored; may break"),
        Some(Enhancement::Steel) => parts.push("Mult x1.5 while held"),
        Some(Enhancement::Stone) => parts.push("Stone card: scores without its rank"),
        Some(Enhancement::Gold) => parts.push("Pays $3 while held"),
        Some(Enhancement::Lucky) => parts.push("Chance for Mult or money when scored"),
        None => {}
    }
    match card.edition {
        Edition::Foil => parts.push("Foil: +50 chips"),
        Edition::Holographic => parts.push("Holographic: +10 Mult"),
        Edition::Polychrome => parts.push("Polychrome: Mult x1.5"),
        Edition::Negative => parts.push("Negative edition"),
        Edition::Base => {}
    }
    match card.seal {
        Some(Seal::Gold) => parts.push("Gold seal: earns $3 when scored"),
        Some(Seal::Red) => parts.push("Red seal: repeats activation"),
        Some(Seal::Blue) => parts.push("Blue seal: may create a Planet at round end"),
        Some(Seal::Purple) => parts.push("Purple seal: creates a Tarot on discard if space allows"),
        None => {}
    }
    parts.join("; ")
}

pub(super) fn consumable_description(c: balatro_rs::consumable::Consumable) -> String {
    use balatro_rs::consumable::Consumable;
    match c {
        Consumable::Planet(p) => format!("Improves the {:?} hand.", p.hand_rank()),
        Consumable::Tarot(t) => {
            use balatro_rs::tarot::Tarot;
            let effect = match t {
                Tarot::Magician => "Adds Lucky to selected cards.",
                Tarot::Empress => "Adds Mult enhancement to selected cards.",
                Tarot::Hierophant => "Adds Bonus enhancement to selected cards.",
                Tarot::Lovers => "Adds Wild enhancement to selected cards.",
                Tarot::Chariot => "Adds Steel enhancement to selected cards.",
                Tarot::Justice => "Adds Glass enhancement to selected cards.",
                Tarot::Devil => "Adds Gold enhancement to selected cards.",
                Tarot::Tower => "Adds Stone enhancement to selected cards.",
                Tarot::Star => "Changes selected cards to diamonds.",
                Tarot::Moon => "Changes selected cards to clubs.",
                Tarot::Sun => "Changes selected cards to hearts.",
                Tarot::World => "Changes selected cards to spades.",
                Tarot::Strength => "Raises selected ranks by one.",
                Tarot::HangedMan => "Destroys the selected cards.",
                Tarot::Death => "Copies the second selected card onto the first.",
                Tarot::Hermit => "Doubles current money, adding at most $20.",
                Tarot::Temperance => "Pays the total sell value of jokers, up to $50.",
                Tarot::WheelOfFortune => "May add a random edition to an owned joker.",
                Tarot::HighPriestess => "Creates up to two random Planet cards if there is room.",
                Tarot::Emperor => "Creates up to two random Tarot cards if there is room.",
                Tarot::Judgement => "Creates a random joker if a slot is free.",
                Tarot::Fool => "Copies the last Tarot or Planet used, if there is room.",
            };
            with_selection(effect, t.requires_hand(), t.max_targets())
        }
        Consumable::Spectral(s) => {
            use balatro_rs::spectral::Spectral;
            let effect = match s {
                Spectral::Talisman => "Adds a gold seal to a selected card.",
                Spectral::DejaVu => "Adds a red seal to a selected card.",
                Spectral::Trance => "Adds a blue seal to a selected card.",
                Spectral::Medium => "Adds a purple seal to a selected card.",
                Spectral::Aura => "Adds a random edition to a selected card.",
                Spectral::Cryptid => "Creates two copies of a selected card.",
                Spectral::Familiar => "Destroys a random card and adds three enhanced face cards.",
                Spectral::Grim => "Destroys a random card and adds two enhanced aces.",
                Spectral::Incantation => {
                    "Destroys a random card and adds four enhanced numbered cards."
                }
                Spectral::Sigil => "Changes the entire hand to one random suit.",
                Spectral::Ouija => "Changes the hand to one random rank and reduces hand size.",
                Spectral::Immolate => "Destroys up to five random cards and grants $20.",
                Spectral::BlackHole => "Improves every poker hand level.",
                Spectral::Wraith => "Creates a rare joker if possible and sets money to zero.",
                Spectral::Soul => "Creates a legendary joker if a slot is free.",
                Spectral::Ectoplasm => {
                    "Adds a negative edition to a random joker and reduces hand size."
                }
                Spectral::Ankh => "Keeps a random joker, duplicates it and removes the others.",
                Spectral::Hex => {
                    "Gives a random joker a polychrome edition and removes the others."
                }
            };
            with_selection(effect, s.requires_hand(), s.max_targets())
        }
    }
}

/// A consumable effect with how many hand cards it takes: nothing when it takes none,
/// otherwise the most that can be selected.
pub(super) fn with_selection(effect: &str, requires_hand: bool, max: usize) -> String {
    match (requires_hand, max) {
        (false, _) | (_, 0) => effect.to_owned(),
        (true, 1) => format!("{effect} Select at most 1 card."),
        (true, max) => format!("{effect} Select at most {max} cards."),
    }
}

pub fn joker_description(id: &str) -> String {
    match id {
        "j_joker" => "Adds 4 Mult when a hand scores.",
        "j_greedy_joker" => "Each scored diamond adds 3 Mult.",
        "j_lusty_joker" => "Each scored heart adds 3 Mult.",
        "j_wrathful_joker" => "Each scored spade adds 3 Mult.",
        "j_gluttenous_joker" => "Each scored club adds 3 Mult.",
        "j_jolly" => "A hand containing a pair gains 8 Mult.",
        "j_zany" => "A hand containing three of a kind gains 12 Mult.",
        "j_mad" => "A hand containing two pairs gains 10 Mult.",
        "j_crazy" => "A hand containing a straight gains 12 Mult.",
        "j_droll" => "A hand containing a flush gains 10 Mult.",
        "j_sly" => "A hand containing a pair gains 50 chips.",
        "j_wily" => "A hand containing three of a kind gains 100 chips.",
        "j_clever" => "A hand containing two pairs gains 80 chips.",
        "j_devious" => "A hand containing a straight gains 100 chips.",
        "j_crafty" => "A hand containing a flush gains 80 chips.",
        "j_half" => "A played hand of at most three cards gains 20 Mult.",
        "j_banner" => "Each unused discard contributes 30 chips.",
        "j_mystic_summit" => "Adds 15 Mult when no discards remain.",
        "j_supernova" => "Adds Mult based on the number of times this hand was played.",
        "j_ride_the_bus" => "Gains Mult for hands without scoring face cards.",
        "j_fibonacci" => "Scored aces, twos, threes, fives and eights each add 8 Mult.",
        "j_scary_face" => "Each scored face card adds 30 chips.",
        "j_abstract" => "Each owned joker adds 3 Mult.",
        "j_pareidolia" => "Treats every playing card as a face card.",
        "j_even_steven" => "Scored even ranks each add 4 Mult.",
        "j_odd_todd" => "Scored odd ranks each add 31 chips.",
        "j_scholar" => "Each scored ace adds 20 chips and 4 Mult.",
        "j_blue_joker" => "Each card remaining in the deck adds 2 chips.",
        "j_baron" => "Each king held in hand multiplies Mult by 1.5.",
        "j_mime" => "Repeats abilities of cards held in hand.",
        "j_hack" => "Repeats scored twos, threes, fours and fives.",
        "j_dusk" => "Repeats played cards on the last hand of the round.",
        "j_hanging_chad" => "Repeats the first scoring card.",
        "j_raised_fist" => "Adds twice the lowest held rank to Mult.",
        "j_card_sharp" => "A hand type already played this round multiplies Mult by 3.",
        "j_stencil" => {
            "Mult is multiplied by the number of empty joker slots, counting this joker."
        }
        "j_steel_joker" => "Mult gains a 0.2 multiplier for each steel card in the deck.",
        "j_business" => "Each scored face card has a one-in-two chance to pay $2.",
        "j_faceless" => "Discard at least three face cards together to earn $5.",
        "j_green_joker" => "Gains 1 Mult per hand and loses 1 per discard.",
        "j_obelisk" => "Gains a 0.2 Mult multiplier when avoiding your most played hand type.",
        "j_midas_mask" => "Scored face cards become gold cards.",
        "j_luchador" => "Selling this joker disables the current boss.",
        "j_photograph" => "The first scored face card doubles Mult.",
        "j_erosion" => "Each card missing from the starting deck size adds 4 Mult.",
        "j_reserved_parking" => "Each held face card has a one-in-two chance to pay $1.",
        "j_mail" => "Each discarded card of the current rank pays $5; rank changes each round.",
        "j_stone" => "Each stone card in the deck contributes 25 chips.",
        "j_baseball" => "Each uncommon joker multiplies Mult by 1.5.",
        "j_bull" => "Each dollar held adds 2 chips.",
        "j_trading" => "A single-card first discard destroys the card and earns $3.",
        "j_trousers" => "Gains 2 Mult when a hand contains two pairs.",
        "j_walkie_talkie" => "Each scored four or ten adds 10 chips and 4 Mult.",
        "j_castle" => {
            "Gains 3 chips per discarded card of the current suit; suit changes each round."
        }
        "j_smiley" => "Each scored face card adds 5 Mult.",
        "j_ticket" => "Each scored gold card pays $4.",
        "j_acrobat" => "The last hand of the round multiplies Mult by 3.",
        "j_sock_and_buskin" => "Repeats each scored face card.",
        "j_rough_gem" => "Each scored diamond pays $1.",
        "j_bloodstone" => "Scored hearts have a one-in-two chance to multiply Mult by 1.5.",
        "j_arrowhead" => "Each scored spade adds 50 chips.",
        "j_onyx_agate" => "Each scored club adds 7 Mult.",
        "j_wee" => "Permanently gains 8 chips for every scored two.",
        "j_matador" => "Earns $8 when a played hand triggers the boss ability.",
        "j_hit_the_road" => "Gains a 0.5 Mult multiplier per jack discarded this round.",
        "j_duo" => "A hand containing a pair multiplies Mult by 2.",
        "j_trio" => "A hand containing three of a kind multiplies Mult by 3.",
        "j_family" => "A hand containing four of a kind multiplies Mult by 4.",
        "j_order" => "A hand containing a straight multiplies Mult by 3.",
        "j_tribe" => "A hand containing a flush multiplies Mult by 2.",
        "j_shoot_the_moon" => "Each queen held in hand adds 13 Mult.",
        "j_drivers_license" => "At least sixteen enhanced cards in the deck multiply Mult by 3.",
        "j_yorick" => "Gains a Mult multiplier after every 23 cards discarded.",
        "j_chicot" => "Disables boss blind abilities.",
        "j_misprint" => "Adds a random amount of Mult when scoring.",
        "j_golden" => "Earns money at the end of a round.",
        "j_ice_cream" => "Adds chips when scoring; decreases after each hand.",
        _ => "Joker effect: see supported rules in the engine.",
    }
    .into()
}

/// Value each scaling joker holds now, read from its engine state like the original `loc_vars`.
pub(super) fn joker_current(
    joker: &balatro_rs::joker::Jokers,
    game: &Game,
) -> Option<CurrentValue> {
    let counter = joker.state().counter.max(0.0) as usize;
    let full_deck = || game.full_deck();
    let enhanced = |kind: Enhancement| {
        full_deck()
            .iter()
            .filter(|card| card.enhancement == Some(kind))
            .count()
    };
    let discarded = || game.non_debuffed(game.discarded_this_round.iter());
    let hundredths = |factor: f64| (factor * 100.0).round() as usize;
    Some(match joker.id() {
        "j_green_joker" => CurrentValue::Mult(counter),
        "j_ride_the_bus" => CurrentValue::Mult(game.consecutive_hands_without_face_card),
        "j_trousers" => CurrentValue::Mult(counter),
        "j_abstract" => CurrentValue::Mult(game.jokers.len() * 3),
        "j_erosion" => CurrentValue::Mult(52usize.saturating_sub(full_deck().len()) * 4),
        "j_blue_joker" => CurrentValue::Chips(game.deck.cards().len() * 2),
        "j_bull" => CurrentValue::Chips(game.money * 2),
        "j_wee" => CurrentValue::Chips(counter),
        "j_stone" => CurrentValue::Chips(enhanced(Enhancement::Stone) * 25),
        "j_castle" => CurrentValue::Chips(counter),
        "j_ice_cream" => CurrentValue::Chips(100usize.saturating_sub(5 * counter)),
        "j_obelisk" => CurrentValue::XMult(hundredths(1.0 + joker.state().counter as f64)),
        "j_hit_the_road" => {
            let jacks = discarded()
                .iter()
                .filter(|card| card.value == balatro_rs::card::Value::Jack)
                .count();
            CurrentValue::XMult(hundredths(1.0 + 0.5 * jacks as f64))
        }
        "j_steel_joker" => {
            CurrentValue::XMult(hundredths(1.0 + 0.2 * enhanced(Enhancement::Steel) as f64))
        }
        "j_yorick" => CurrentValue::XMult(100 * (1 + counter / 23)),
        "j_stencil" => {
            // Empty slots plus every Stencil held, like the original `x_mult`.
            let empty = game.joker_slots().saturating_sub(game.jokers.len());
            let stencils = game
                .jokers
                .iter()
                .filter(|joker| joker.id() == "j_stencil")
                .count();
            CurrentValue::XMult(100 * (empty + stencils))
        }
        // Ante shows these too: their value follows the discards left this round.
        "j_banner" => CurrentValue::Chips(30 * game.discards()),
        "j_mystic_summit" => CurrentValue::Mult(if game.discards() == 0 { 15 } else { 0 }),
        "j_drivers_license" => CurrentValue::Count(
            full_deck()
                .iter()
                .filter(|card| card.enhancement.is_some())
                .count(),
        ),
        _ => return None,
    })
}

pub(super) fn sticker_summary(stickers: balatro_rs::joker::Stickers) -> String {
    let mut active = Vec::new();
    if stickers.eternal {
        active.push("Eternal");
    }
    if stickers.perishable {
        active.push("Perishable");
    }
    if stickers.rental {
        active.push("Rental");
    }
    active.join(" · ")
}

/// The starting rule of the deck with save id `id` (`b_red`, …), as the engine applies it.
pub fn deck_description(id: &str) -> Option<String> {
    balatro_types::DeckVariant::from_id(id).map(|deck| deck.description().to_owned())
}
