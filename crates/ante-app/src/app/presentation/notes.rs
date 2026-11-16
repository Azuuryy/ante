//! What a command did outside its score trace, shown as popups once the score animation
//! is over: a Joker's message on a discard or at the end of the round (Castle "Upgrade!",
//! Hit the Road "X1.5 Mult", Green Joker "-1 Mult", Mail-In Rebate "$5"…), a Gold card's
//! "$3" and a seal's "+1 Tarot" or "+1 Planet" on the card, one after the other like the
//! original's `card_eval_status_text` events.

use super::*;
use ante_core::contract::{JokerNote, MessageInk, Note};

/// How long each note shows, in game seconds.
pub(crate) const NOTE_TIME: f32 = 0.6;

/// A note's popup, removed when the next note shows or its time is up.
#[derive(Component)]
pub(crate) struct NotePopup;

/// The Joker or card a note shows on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NoteTarget {
    Joker(JokerKey),
    Card(CardKey),
}

/// The popup of a note, or `None` for a note with nothing to show (a tag gained).
pub(crate) fn note_popup(note: &Note) -> Option<(NoteTarget, Popup)> {
    Some(match note {
        Note::JokerMessage { id, note, .. } => (
            NoteTarget::Joker(*id),
            match *note {
                JokerNote::Upgrade => Popup::Message("Upgrade!", MessageInk::Chips),
                JokerNote::Reset => Popup::Message("Reset", MessageInk::Mult),
                JokerNote::Dollars(amount) => Popup::Money(amount as i64),
                JokerNote::XMult(hundredths) => Popup::XMult(hundredths),
                JokerNote::MultMinus(amount) => Popup::Mult(-(amount as i64)),
                JokerNote::Extinct => Popup::Message("Extinct!", MessageInk::Filter),
                JokerNote::Safe => Popup::Message("Safe!", MessageInk::Filter),
                JokerNote::MultPlus(amount) => Popup::Mult(amount as i64),
                JokerNote::XMultMinus(_) => Popup::Message("-X0.01", MessageInk::Mult),
                JokerNote::HandSizeMinus(_) => Popup::Message("-1 Hand Size", MessageInk::Filter),
                JokerNote::Eaten => Popup::Message("Eaten!", MessageInk::Filter),
                JokerNote::UpgradePlain => Popup::Message("Upgrade!", MessageInk::Filter),
                JokerNote::Remaining(left) => Popup::Message(
                    ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"][left.min(9)],
                    MessageInk::Filter,
                ),
                JokerNote::Drank => Popup::Message("Drank!", MessageInk::Filter),
                JokerNote::ValueUp => Popup::Message("Value Up!", MessageInk::Money),
                JokerNote::UpgradeMoney => Popup::Message("Upgrade!", MessageInk::Money),
                JokerNote::HandsPlus(_) => Popup::Message("+3 Hands", MessageInk::Filter),
                JokerNote::Saved => Popup::Message("Saved!", MessageInk::Filter),
                JokerNote::PlusTarot => Popup::Message("+1 Tarot", MessageInk::Tarot),
                JokerNote::PlusSpectral => Popup::Message("+1 Spectral", MessageInk::Spectral),
                JokerNote::PlusJoker => Popup::Message("+1 Joker", MessageInk::Blue),
                JokerNote::PlusStone => Popup::Message("+1 Stone", MessageInk::Filter),
                JokerNote::LevelUp => Popup::Message("Level Up!", MessageInk::Filter),
            },
        ),
        Note::CardPaid { card, amount, .. } => {
            (NoteTarget::Card(*card), Popup::Money(*amount as i64))
        }
        Note::SealCreated { card, planet, .. } => (
            NoteTarget::Card(*card),
            if *planet {
                Popup::Message("+1 Planet", MessageInk::Planet)
            } else {
                Popup::Message("+1 Tarot", MessageInk::Tarot)
            },
        ),
        Note::TagGained(_) => return None,
    })
}

/// The held Jokers and the cards on screen a note can show on.
type NoteTargets<'w, 's> = (
    Query<
        'w,
        's,
        (
            Entity,
            &'static ScoredJoker,
            Option<&'static mut HoverMotion>,
        ),
        Without<ScoredCard>,
    >,
    Query<
        'w,
        's,
        (
            Entity,
            &'static ScoredCard,
            Option<&'static mut HoverMotion>,
        ),
        Without<ScoredJoker>,
    >,
);

/// Shows the waiting notes one at a time while no score animation plays. A note whose
/// Joker or card is no longer on screen is skipped.
pub(crate) fn show_notes(
    mut commands: Commands,
    mut session: ResMut<Session>,
    time: Res<Time<Real>>,
    mut timer: Local<f32>,
    shown: Query<Entity, With<NotePopup>>,
    (mut jokers, mut cards): NoteTargets,
    mut sounds: MessageWriter<Sfx>,
) {
    if session.playback.is_some() {
        return;
    }
    *timer -= time.delta_secs() * session.speed.factor();
    if *timer > 0.0 {
        return;
    }
    for popup in &shown {
        commands.entity(popup).despawn();
    }
    if session.notes.is_empty() {
        return;
    }
    while let Some(note) = session.bypass_change_detection().notes.pop_front() {
        let Some((target, popup)) = note_popup(&note) else {
            continue;
        };
        let found = match target {
            NoteTarget::Joker(id) => jokers
                .iter_mut()
                .find(|(_, joker, _)| joker.id == id)
                .map(|(entity, _, motion)| (entity, motion, PopupSide::Below)),
            NoteTarget::Card(id) => cards
                .iter_mut()
                .find(|(_, card, _)| card.id == id)
                .map(|(entity, _, motion)| (entity, motion, PopupSide::Above)),
        };
        let Some((entity, motion, side)) = found else {
            continue;
        };
        if let Some(mut motion) = motion {
            motion.juice_up(0.6, 0.1, rand::random());
        }
        sounds.write(popup_sound(popup));
        commands
            .entity(entity)
            .with_child((NotePopup, score_popup(popup, side)));
        *timer = NOTE_TIME;
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ante_core::contract::ContentId;

    #[test]
    fn notes_become_the_original_popups() {
        let joker = |note| Note::JokerMessage {
            id: JokerKey(3),
            kind: ContentId("j_castle"),
            name: "Castle",
            note,
        };
        assert_eq!(
            note_popup(&joker(JokerNote::Upgrade)),
            Some((
                NoteTarget::Joker(JokerKey(3)),
                Popup::Message("Upgrade!", MessageInk::Chips)
            ))
        );
        assert_eq!(
            note_popup(&joker(JokerNote::XMult(150))).unwrap().1,
            Popup::XMult(150)
        );
        assert_eq!(
            note_popup(&joker(JokerNote::MultMinus(1))).unwrap().1,
            Popup::Mult(-1)
        );
        let paid = Note::CardPaid {
            card: CardKey(9),
            label: "KH gold".into(),
            amount: 3,
        };
        assert_eq!(
            note_popup(&paid),
            Some((NoteTarget::Card(CardKey(9)), Popup::Money(3)))
        );
        assert_eq!(note_popup(&Note::TagGained("Speed Tag".into())), None);
    }
}
