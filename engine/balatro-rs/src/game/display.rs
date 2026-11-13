use super::*;

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hand_str: String = self
            .available
            .cards_and_selected()
            .iter()
            .map(|(c, sel)| {
                if *sel {
                    format!("[{}]", card_display(c))
                } else {
                    card_display(c)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(f, "hand: {}", hand_str)?;
        writeln!(f, "discard pile: {}", self.discarded.len())?;
        let deck_cards = self.deck.cards();
        let modified: Vec<String> = deck_cards
            .iter()
            .filter(|c| c.enhancement.is_some() || c.edition != Edition::Base || c.seal.is_some())
            .map(card_display)
            .collect();
        if modified.is_empty() {
            writeln!(f, "deck: {}", deck_cards.len())?;
        } else {
            writeln!(
                f,
                "deck: {} ({} modified: {})",
                deck_cards.len(),
                modified.len(),
                modified.join(" ")
            )?;
        }
        if self.jokers.is_empty() {
            writeln!(f, "jokers: (none)")?;
        } else {
            writeln!(f, "jokers:")?;
            for j in self.jokers.clone() {
                writeln!(f, "  {}", joker_display(&j))?;
            }
        }
        if self.consumables.is_empty() {
            writeln!(f, "consumables: (none)")?;
        } else {
            let parts: Vec<String> = self
                .consumables
                .iter()
                .map(|c| format!("[{}] {}", c.type_label(), c.name()))
                .collect();
            writeln!(f, "consumables: {}", parts.join(", "))?;
        }
        if self.vouchers.is_empty() {
            writeln!(f, "vouchers: (none)")?;
        } else {
            let names: Vec<&str> = self.vouchers.owned().iter().map(|v| v.name()).collect();
            writeln!(f, "vouchers: {}", names.join(", "))?;
        }
        writeln!(f, "planetarium: {}", self.planetarium)?;
        writeln!(f, "stage: {:?}", self.stage)?;
        writeln!(f, "ante: {:?}", self.ante_current)?;
        writeln!(f, "blind: {:?}", self.blind)?;
        writeln!(f, "round: {}", self.round)?;
        writeln!(f, "hands remaining: {}", self.plays)?;
        writeln!(f, "discards remaining: {}", self.discards())?;
        writeln!(f, "money: {}", self.money)?;
        if matches!(self.stage, Stage::Blind(_)) {
            writeln!(
                f,
                "score: {}  target: {}",
                self.score,
                self.required_score()
            )
        } else {
            writeln!(f, "score: {}", self.score)
        }
    }
}
