//! What a command did outside its score trace, named for run logs and messages.

use super::{CardKey, ContentId, JokerKey};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Note {
    /// A held card paid money at the end of the round (a Gold card).
    CardPaid {
        card: CardKey,
        label: String,
        amount: usize,
    },
    /// A seal created a consumable: Blue at the end of the round, Purple on a discard.
    SealCreated {
        card: CardKey,
        label: String,
        created: String,
        /// A Planet (Blue Seal) rather than a Tarot (Purple Seal).
        planet: bool,
    },
    /// A tag was gained; it joins the tag row or takes effect at once.
    TagGained(String),
    /// A Joker's message on a discard or at the end of the round, shown on the Joker.
    JokerMessage {
        id: JokerKey,
        kind: ContentId,
        name: &'static str,
        note: JokerNote,
    },
}

/// A Joker's message outside the score trace, as the original writes and colours it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JokerNote {
    /// "Upgrade!" in the chips colour.
    Upgrade,
    /// "Reset" in red.
    Reset,
    /// "$n" in the money colour.
    Dollars(usize),
    /// "Xn Mult" in red, the factor in hundredths.
    XMult(usize),
    /// "-n Mult" in red.
    MultMinus(usize),
    /// "Extinct!" in the default colour.
    Extinct,
    /// "Safe!" in the default colour.
    Safe,
    /// "+n Mult" in red.
    MultPlus(usize),
    /// "-Xn" in red, the factor in hundredths.
    XMultMinus(usize),
    /// "-n Hand Size".
    HandSizeMinus(usize),
    /// "Eaten!".
    Eaten,
    /// "Upgrade!" in the default colour.
    UpgradePlain,
    /// The hands left, as a bare number.
    Remaining(usize),
    /// "Drank!".
    Drank,
    /// "Value Up!" in the money colour.
    ValueUp,
    /// "Upgrade!" in the money colour.
    UpgradeMoney,
    /// "+n Hands".
    HandsPlus(usize),
    /// "Saved!".
    Saved,
    /// "+1 Tarot".
    PlusTarot,
    /// "+1 Spectral".
    PlusSpectral,
    /// "+1 Joker".
    PlusJoker,
    /// "+1 Stone".
    PlusStone,
    /// "Level Up!".
    LevelUp,
}

impl std::fmt::Display for JokerNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Upgrade => write!(f, "Upgrade!"),
            Self::Reset => write!(f, "Reset"),
            Self::Dollars(amount) => write!(f, "${amount}"),
            Self::XMult(hundredths) => {
                let text = format!("{}.{:02}", hundredths / 100, hundredths % 100);
                write!(
                    f,
                    "X{} Mult",
                    text.trim_end_matches('0').trim_end_matches('.')
                )
            }
            Self::MultMinus(amount) => write!(f, "-{amount} Mult"),
            Self::Extinct => write!(f, "Extinct!"),
            Self::Safe => write!(f, "Safe!"),
            Self::MultPlus(amount) => write!(f, "+{amount} Mult"),
            Self::XMultMinus(hundredths) => {
                write!(f, "-X{}.{:02}", hundredths / 100, hundredths % 100)
            }
            Self::HandSizeMinus(amount) => write!(f, "-{amount} Hand Size"),
            Self::Eaten => write!(f, "Eaten!"),
            Self::UpgradePlain => write!(f, "Upgrade!"),
            Self::Remaining(left) => write!(f, "{left}"),
            Self::Drank => write!(f, "Drank!"),
            Self::ValueUp => write!(f, "Value Up!"),
            Self::UpgradeMoney => write!(f, "Upgrade!"),
            Self::HandsPlus(amount) => write!(f, "+{amount} Hands"),
            Self::Saved => write!(f, "Saved!"),
            Self::PlusTarot => write!(f, "+1 Tarot"),
            Self::PlusSpectral => write!(f, "+1 Spectral"),
            Self::PlusJoker => write!(f, "+1 Joker"),
            Self::PlusStone => write!(f, "+1 Stone"),
            Self::LevelUp => write!(f, "Level Up!"),
        }
    }
}

/// Short name of a playing card: rank, suit initial, then its enhancement, edition and
/// seal in lower case, for example `KH steel` or `2C purple_seal`. Empty, `Base` and
/// `None` parts are left out.
pub fn card_label(rank: char, suit: &str, enhancement: &str, edition: &str, seal: &str) -> String {
    let mut label = String::from(rank);
    label.extend(suit.chars().next());
    for (part, suffix) in [(enhancement, ""), (edition, ""), (seal, "_seal")] {
        if !part.is_empty() && part != "Base" && part != "None" {
            label.push(' ');
            label.push_str(&part.to_lowercase());
            label.push_str(suffix);
        }
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_labels_are_short() {
        assert_eq!(
            card_label('K', "Hearts", "Steel", "Base", "None"),
            "KH steel"
        );
        assert_eq!(
            card_label('2', "Clubs", "", "Foil", "Purple"),
            "2C foil purple_seal"
        );
        assert_eq!(card_label('T', "Diamonds", "None", "", ""), "TD");
    }
}
