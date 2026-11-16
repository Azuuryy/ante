use ante_core::contract::Deck;

pub(crate) const STAKES: [&str; 8] = [
    "White Stake",
    "Red Stake",
    "Green Stake",
    "Black Stake",
    "Blue Stake",
    "Purple Stake",
    "Orange Stake",
    "Gold Stake",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Main,
    DeckSelect,
    Options,
    Pause,
    RunInfo,
    DeckInspect,
    Collection,
    Run,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeckChoice {
    Red,
    Blue,
    Yellow,
    Green,
    Black,
    Magic,
    Nebula,
    Ghost,
    Abandoned,
    Checkered,
    Zodiac,
    Painted,
    Anaglyph,
    Plasma,
    Erratic,
}

impl DeckChoice {
    pub const ALL: [Self; 15] = [
        Self::Red,
        Self::Blue,
        Self::Yellow,
        Self::Green,
        Self::Black,
        Self::Magic,
        Self::Nebula,
        Self::Ghost,
        Self::Abandoned,
        Self::Checkered,
        Self::Zodiac,
        Self::Painted,
        Self::Anaglyph,
        Self::Plasma,
        Self::Erratic,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Red => "Red Deck",
            Self::Blue => "Blue Deck",
            Self::Yellow => "Yellow Deck",
            Self::Green => "Green Deck",
            Self::Black => "Black Deck",
            Self::Magic => "Magic Deck",
            Self::Nebula => "Nebula Deck",
            Self::Ghost => "Ghost Deck",
            Self::Abandoned => "Abandoned Deck",
            Self::Checkered => "Checkered Deck",
            Self::Zodiac => "Zodiac Deck",
            Self::Painted => "Painted Deck",
            Self::Anaglyph => "Anaglyph Deck",
            Self::Plasma => "Plasma Deck",
            Self::Erratic => "Erratic Deck",
        }
    }

    pub fn deck(self) -> Option<Deck> {
        match self {
            Self::Red => Some(Deck::Red),
            Self::Blue => Some(Deck::Blue),
            _ => None,
        }
    }

    /// The deck's starting rule (`b_*` description), read from the engine.
    pub fn effect(self) -> String {
        ante_core::engine::deck_description(self.id()).unwrap_or_default()
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Red => "b_red",
            Self::Blue => "b_blue",
            Self::Yellow => "b_yellow",
            Self::Green => "b_green",
            Self::Black => "b_black",
            Self::Magic => "b_magic",
            Self::Nebula => "b_nebula",
            Self::Ghost => "b_ghost",
            Self::Abandoned => "b_abandoned",
            Self::Checkered => "b_checkered",
            Self::Zodiac => "b_zodiac",
            Self::Painted => "b_painted",
            Self::Anaglyph => "b_anaglyph",
            Self::Plasma => "b_plasma",
            Self::Erratic => "b_erratic",
        }
    }

    /// Collection unlock condition, `None` when the deck starts unlocked.
    pub fn unlock(self) -> Option<&'static str> {
        match self {
            Self::Red => None,
            Self::Blue => Some("Discover 20 items."),
            Self::Yellow => Some("Discover 50 items."),
            Self::Green => Some("Discover 75 items."),
            Self::Black => Some("Discover 100 items."),
            Self::Magic => Some("Win a run with the Red Deck."),
            Self::Nebula => Some("Win a run with the Blue Deck."),
            Self::Ghost => Some("Win a run with the Yellow Deck."),
            Self::Abandoned => Some("Win a run with the Green Deck."),
            Self::Checkered => Some("Win a run with the Black Deck."),
            Self::Zodiac => Some("Win a run at Red Stake."),
            Self::Painted => Some("Win a run at Green Stake."),
            Self::Anaglyph => Some("Win a run at Black Stake."),
            Self::Plasma => Some("Win a run at Blue Stake."),
            Self::Erratic => Some("Win a run at Purple Stake."),
        }
    }

    pub fn shifted(self, offset: isize) -> Self {
        let index = Self::ALL.iter().position(|choice| *choice == self).unwrap();
        Self::ALL[(index as isize + offset).rem_euclid(Self::ALL.len() as isize) as usize]
    }
}

#[derive(Debug, Default)]
pub struct Navigation {
    pub screen: Screen,
    previous: Screen,
    deck_return: Screen,
    pub deck: Option<DeckChoice>,
    pub collection_page: usize,
}

impl Navigation {
    pub fn open_decks(&mut self) {
        self.deck_return = match self.screen {
            Screen::Pause | Screen::Run => self.screen,
            _ => Screen::Main,
        };
        self.screen = Screen::DeckSelect;
        self.deck.get_or_insert(DeckChoice::Red);
    }

    pub fn open_run_info(&mut self) {
        self.previous = self.screen;
        self.screen = Screen::RunInfo;
    }

    pub fn open_options(&mut self) {
        self.previous = self.screen;
        self.screen = Screen::Options;
    }

    pub fn open_collection(&mut self, page: usize) {
        if self.screen != Screen::Collection {
            self.previous = self.screen;
        }
        self.screen = Screen::Collection;
        self.collection_page = page;
    }

    pub fn overlays_run(&self) -> bool {
        match self.screen {
            Screen::Pause | Screen::RunInfo | Screen::DeckInspect => true,
            Screen::DeckSelect => matches!(self.deck_return, Screen::Pause | Screen::Run),
            Screen::Options | Screen::Collection => {
                matches!(self.previous, Screen::Pause | Screen::Run)
            }
            _ => false,
        }
    }

    pub fn back(&mut self) {
        self.screen = match self.screen {
            Screen::Options | Screen::Collection | Screen::RunInfo => self.previous,
            Screen::DeckSelect => self.deck_return,
            Screen::Pause | Screen::DeckInspect => Screen::Run,
            _ => self.screen,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_selector_wraps_and_only_supported_decks_can_start() {
        assert_eq!(DeckChoice::Red.shifted(-1), DeckChoice::Erratic);
        assert_eq!(DeckChoice::Erratic.shifted(1), DeckChoice::Red);
        assert_eq!(DeckChoice::Red.deck(), Some(Deck::Red));
        assert_eq!(DeckChoice::Blue.deck(), Some(Deck::Blue));
        assert!(
            DeckChoice::ALL[2..]
                .iter()
                .all(|deck| deck.deck().is_none())
        );
    }

    #[test]
    fn overlays_return_to_the_original_screen() {
        let mut navigation = Navigation::default();
        navigation.open_decks();
        navigation.open_options();
        navigation.back();
        assert_eq!(navigation.screen, Screen::DeckSelect);
        navigation.screen = Screen::Run;
        navigation.open_collection(0);
        navigation.open_collection(1);
        navigation.back();
        assert_eq!(navigation.screen, Screen::Run);
    }
}
