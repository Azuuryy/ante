use super::*;

#[cfg(feature = "serde")]
impl Game {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        #[derive(serde::Deserialize)]
        struct TagFields {
            skipped_count: Option<usize>,
        }
        let legacy_tags = serde_json::from_str::<TagFields>(s)?
            .skipped_count
            .is_none();
        let mut game: Self = serde_json::from_str(s)?;
        for ante in [game.ante_start, game.ante_end, game.ante_current] {
            if Ante::try_from(ante.number()) != Ok(ante) {
                return Err(serde::de::Error::custom("invalid ante in snapshot"));
            }
        }
        let extinct = game.gros_michel_extinct;
        game.backend.set_gros_michel_extinct(extinct);
        let jokers = game.jokers.clone();
        game.effect_registry.register_jokers(jokers, &game.clone());

        // Deserializing doesn't allocate ids, so Card::new() calls after this
        // point would otherwise collide with ids already present in `game`.
        let pack_contents = game
            .shop
            .packs
            .iter()
            .flat_map(|pack| &pack.contents)
            .chain(game.open_pack.iter().flat_map(|pack| &pack.contents));
        let max_id = game
            .deck
            .cards()
            .iter()
            .chain(game.available.cards().iter())
            .chain(game.discarded.iter())
            .chain(game.shop.cards.iter())
            .chain(pack_contents.clone().filter_map(|content| match content {
                PackContent::PlayingCard(card) => Some(card),
                _ => None,
            }))
            .map(|c| c.id)
            .max();
        if let Some(max_id) = max_id {
            crate::card::ensure_id_counter_past(max_id);
        }

        // Same fixup for joker instance ids, minted ones sit in `game.jokers`
        // and `game.shop.jokers`, and 0 means "never minted" so it's excluded.
        let max_joker_id = game
            .jokers
            .iter()
            .chain(game.shop.jokers.iter())
            .chain(pack_contents.filter_map(|content| match content {
                PackContent::Joker(joker) => Some(joker),
                _ => None,
            }))
            .map(|j| j.instance_id())
            .filter(|&id| id > 0)
            .max();
        if game.current_boss.is_none() {
            game.draw_ante_boss();
        }

        if let Some(max_joker_id) = max_joker_id {
            crate::joker::ensure_joker_id_counter_past(max_joker_id);
        }

        if legacy_tags {
            game.skipped_count = game
                .action_history
                .iter()
                .filter(|action| matches!(action, Action::SkipBlind(_)))
                .count();
            for tag in std::mem::take(&mut game.tags) {
                game.acquire_tag(tag, None);
            }
            if game.stage == Stage::PreBlind() {
                game.resolve_blind_choice_tags();
            }
        }

        Ok(game)
    }
}
