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

/// A Joker's description with the round's target in it, like the original `loc_vars`:
/// Castle names its suit (`castle_card`), Mail-In Rebate its rank (`mail_card`). Before
/// a target is drawn, the general description.
pub(super) fn joker_text(joker: &balatro_rs::joker::Jokers) -> String {
    use balatro_types::joker::SelectorValue;
    match (joker.id(), joker.state().selector) {
        ("j_castle", Some(SelectorValue::Suit(suit))) => format!(
            "Gains 3 chips per discarded {} card; suit changes each round.",
            suit_name(suit).trim_end_matches('s')
        ),
        ("j_mail", Some(SelectorValue::Value(value))) => format!(
            "Each discarded {} pays $5; rank changes each round.",
            rank_word(char::from(value))
        ),
        ("j_todo_list", Some(SelectorValue::Hand(rank))) => format!(
            "Earns $4 when a {} is played; the hand changes each round.",
            hand_name(rank)
        ),
        ("j_ancient", Some(SelectorValue::Suit(suit))) => format!(
            "Each scored {} multiplies Mult by 1.5; suit changes each round.",
            suit_name(suit).trim_end_matches('s')
        ),
        ("j_idol", Some(SelectorValue::Card(value, suit))) => format!(
            "Each scored {} of {} multiplies Mult by 2; card changes each round.",
            rank_word(char::from(value)),
            suit_name(suit)
        ),
        (id, _) => joker_description(id),
    }
}

/// A poker hand's name, as the original writes it.
pub(super) fn hand_name(rank: HandRank) -> &'static str {
    match rank {
        HandRank::HighCard => "High Card",
        HandRank::OnePair => "Pair",
        HandRank::TwoPair => "Two Pair",
        HandRank::ThreeOfAKind => "Three of a Kind",
        HandRank::Straight => "Straight",
        HandRank::Flush => "Flush",
        HandRank::FullHouse => "Full House",
        HandRank::FourOfAKind => "Four of a Kind",
        HandRank::StraightFlush => "Straight Flush",
        HandRank::RoyalFlush => "Royal Flush",
        HandRank::FiveOfAKind => "Five of a Kind",
        HandRank::FlushHouse => "Flush House",
        HandRank::FlushFive => "Flush Five",
    }
}

/// A rank as the original names it: 2 to 10 in figures, then Jack, Queen, King, Ace.
pub(super) fn rank_word(rank: char) -> String {
    match rank {
        'T' => "10".to_owned(),
        'J' => "Jack".to_owned(),
        'Q' => "Queen".to_owned(),
        'K' => "King".to_owned(),
        'A' => "Ace".to_owned(),
        other => other.to_string(),
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
        "j_fibonacci" => "Each scored Ace, 2, 3, 5 or 8 adds 8 Mult.",
        "j_scary_face" => "Each scored face card adds 30 chips.",
        "j_abstract" => "Each owned joker adds 3 Mult.",
        "j_pareidolia" => "Treats every playing card as a face card.",
        "j_even_steven" => "Scored even ranks each add 4 Mult.",
        "j_odd_todd" => "Scored odd ranks each add 31 chips.",
        "j_scholar" => "Each scored Ace adds 20 chips and 4 Mult.",
        "j_blue_joker" => "Each card remaining in the deck adds 2 chips.",
        "j_baron" => "Each King held in hand multiplies Mult by 1.5.",
        "j_mime" => "Repeats abilities of cards held in hand.",
        "j_hack" => "Repeats each scored 2, 3, 4 or 5.",
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
        "j_walkie_talkie" => "Each scored 10 or 4 adds 10 chips and 4 Mult.",
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
        "j_wee" => "Permanently gains 8 chips for every scored 2.",
        "j_matador" => "Earns $8 when a played hand triggers the boss ability.",
        "j_hit_the_road" => "Gains a 0.5 Mult multiplier per Jack discarded this round.",
        "j_duo" => "A hand containing a pair multiplies Mult by 2.",
        "j_trio" => "A hand containing three of a kind multiplies Mult by 3.",
        "j_family" => "A hand containing four of a kind multiplies Mult by 4.",
        "j_order" => "A hand containing a straight multiplies Mult by 3.",
        "j_tribe" => "A hand containing a flush multiplies Mult by 2.",
        "j_shoot_the_moon" => "Each Queen held in hand adds 13 Mult.",
        "j_drivers_license" => "At least 16 enhanced cards in the deck multiply Mult by 3.",
        "j_yorick" => "Gains a Mult multiplier after every 23 cards discarded.",
        "j_chicot" => "Disables boss blind abilities.",
        "j_misprint" => "Adds a random amount of Mult when scoring.",
        "j_golden" => "Earns money at the end of a round.",
        "j_ice_cream" => "Adds chips when scoring; decreases after each hand.",
        "j_swashbuckler" => "Adds the sell value of every other owned joker to Mult.",
        "j_square" => "Gains 4 chips when the played hand has exactly 4 cards.",
        "j_runner" => "Gains 15 chips when the played hand contains a straight.",
        "j_throwback" => "Gains a 0.25 Mult multiplier for each blind skipped this run.",
        "j_fortune_teller" => "Adds 1 Mult for each Tarot card used this run.",
        "j_flash" => "Gains 2 Mult each time the shop is rerolled.",
        "j_red_card" => "Gains 3 Mult each time a Booster Pack is skipped.",
        "j_constellation" => "Gains a 0.1 Mult multiplier each time a Planet card is used.",
        "j_popcorn" => "Adds 20 Mult, 4 less after each round.",
        "j_turtle_bean" => "Hand size is 5 larger, 1 less after each round.",
        "j_ramen" => "Multiplies Mult by 2, 0.01 less for each card discarded.",
        "j_lucky_cat" => "Gains a 0.25 Mult multiplier each time a Lucky card triggers.",
        "j_glass" => "Gains a 0.75 Mult multiplier for each Glass card destroyed.",
        "j_hiker" => "Every scored card permanently gains 5 chips.",
        "j_oops" => "Doubles every listed probability.",
        "j_chaos" => "One free reroll in each shop.",
        "j_burglar" => "When a blind is selected, gain 3 hands and lose all discards.",
        "j_juggler" => "Hand size is 1 larger.",
        "j_drunkard" => "One more discard each round.",
        "j_troubadour" => "Hand size is 2 larger; one less hand each round.",
        "j_merry_andy" => "Three more discards each round; hand size is 1 smaller.",
        "j_mr_bones" => "Prevents a loss if the chips scored are at least 25% of the blind; destroyed.",
        "j_splash" => "Every played card counts in scoring.",
        "j_four_fingers" => "Flushes and straights can be made with 4 cards.",
        "j_8_ball" => "Each scored 8 has a 1 in 4 chance to create a Tarot card, if there is room.",
        "j_superposition" => "A hand containing an Ace and a straight creates a Tarot card, if there is room.",
        "j_seance" => "A hand containing a straight flush creates a Spectral card, if there is room.",
        "j_vagabond" => "A hand played with $4 or less creates a Tarot card, if there is room.",
        "j_loyalty_card" => "Multiplies Mult by 4 every 6 hands played.",
        "j_diet_cola" => "Sell this joker to create a free Double Tag.",
        "j_space" => "1 in 4 chance to upgrade the level of the played poker hand.",
        "j_hallucination" => "1 in 2 chance to create a Tarot card when a Booster Pack is opened.",
        "j_cartomancer" => "Creates a Tarot card when a blind is selected, if there is room.",
        "j_riff_raff" => "Creates 2 Common jokers when a blind is selected, if there is room.",
        "j_marble" => "Adds a Stone card to the deck when a blind is selected.",
        "j_burnt" => "Upgrades the level of the first discarded poker hand each round.",
        "j_shortcut" => "Straights can skip one rank between cards.",
        "j_smeared" => "Hearts and diamonds count as the same suit, and so do spades and clubs.",
        "j_delayed_grat" => "Earns $2 per discard left at the end of the round if no discard was used.",
        "j_egg" => "Gains $3 of sell value at the end of each round.",
        "j_todo_list" => "Earns $4 when the listed poker hand is played; the hand changes each round.",
        "j_to_the_moon" => "Earns $1 more interest for every $5 you hold at the end of the round.",
        "j_cloud_9" => "Earns $1 per 9 in your full deck at the end of the round.",
        "j_rocket" => "Earns $1 at the end of the round; the payout grows by $2 per Boss Blind defeated.",
        "j_satellite" => "Earns $1 per distinct Planet card used this run at the end of the round.",
        "j_vampire" => {
            "Gains a 0.1 Mult multiplier per scoring enhanced card, and removes the enhancement."
        }
        "j_selzer" => "Retriggers every played card for the next 10 hands.",
        "j_hologram" => "Gains a 0.25 Mult multiplier each time a playing card is added to your deck.",
        "j_campfire" => {
            "Gains a 0.25 Mult multiplier per card sold; resets when a Boss Blind is defeated."
        }
        "j_ceremonial" => {
            "When a blind is selected, destroys the joker to the right and gains twice its sell value as Mult."
        }
        "j_madness" => {
            "When a Small or Big Blind is selected, gains a 0.5 Mult multiplier and destroys a random joker."
        }
        "j_blackboard" => "Multiplies Mult by 3 if every card held in hand is a spade or a club.",
        "j_flower_pot" => {
            "Multiplies Mult by 3 if the scoring hand holds a diamond, a club, a heart and a spade."
        }
        "j_seeing_double" => {
            "Multiplies Mult by 2 if the scoring hand holds a club and a card of another suit."
        }
        "j_stuntman" => "Adds 250 chips; hand size is 2 smaller.",
        "j_bootstraps" => "Adds 2 Mult for every $5 you hold.",
        "j_gros_michel" => "Adds 15 Mult; 1 in 6 chance to go extinct at the end of the round.",
        "j_cavendish" => "Multiplies Mult by 3; 1 in 1000 chance to be destroyed at the end of the round.",
        "j_ancient" => "Each scored card of the current suit multiplies Mult by 1.5; suit changes each round.",
        "j_idol" => "Each scored card of the current rank and suit multiplies Mult by 2; card changes each round.",
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
        "j_swashbuckler" => CurrentValue::Mult(
            game.jokers
                .iter()
                .filter(|other| other.instance_id() != joker.instance_id())
                .map(|other| game.joker_sell_value(other))
                .sum(),
        ),
        "j_bootstraps" => CurrentValue::Mult(2 * (game.money / 5)),
        "j_square" | "j_runner" => CurrentValue::Chips(counter),
        "j_flash" | "j_red_card" => CurrentValue::Mult(counter),
        "j_fortune_teller" => CurrentValue::Mult(game.tarots_used()),
        "j_throwback" => CurrentValue::XMult(100 + 25 * game.skipped_count()),
        "j_constellation" | "j_lucky_cat" | "j_glass" => {
            CurrentValue::XMult(hundredths(1.0 + f64::from(joker.state().counter)))
        }
        "j_ramen" => CurrentValue::XMult(hundredths(2.0 - f64::from(joker.state().counter))),
        "j_popcorn" => CurrentValue::Mult(20usize.saturating_sub(counter)),
        "j_vampire" | "j_hologram" | "j_campfire" | "j_madness" => {
            CurrentValue::XMult(hundredths(1.0 + f64::from(joker.state().counter)))
        }
        "j_ceremonial" => CurrentValue::Mult(counter),
        "j_loyalty_card" => CurrentValue::Count(5 - counter % 6),
        "j_rocket" => CurrentValue::Money(1 + counter),
        "j_satellite" => CurrentValue::Money(game.planets_used()),
        "j_cloud_9" => CurrentValue::Money(
            full_deck()
                .iter()
                .filter(|card| card.value == balatro_rs::card::Value::Nine)
                .count(),
        ),
        "j_selzer" => CurrentValue::Count(10usize.saturating_sub(counter)),
        "j_turtle_bean" => CurrentValue::Count(5usize.saturating_sub(counter)),
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
