//! Blueprint and Brainstorm: finding the Joker they copy and running its effects as a
//! copy.

use super::*;

/// Copies of copies stop after as many steps as there are Jokers, like the original's
/// `context.blueprint > #G.jokers.cards + 1`.
const MAX_COPY_DEPTH: usize = 16;

impl Game {
    /// The Joker that copier `id` copies now: the one to its right (Blueprint) or the
    /// leftmost (Brainstorm), if it is another Joker and `blueprint_compatible`.
    pub(crate) fn copy_target(&self, id: usize, leftmost: bool) -> Option<Jokers> {
        let position = self.jokers.iter().position(|j| j.instance_id() == id)?;
        let target = if leftmost {
            self.jokers.first()?
        } else {
            self.jokers.get(position + 1)?
        };
        (target.instance_id() != id && target.blueprint_compatible()).then(|| target.clone())
    }

    /// Runs `apply` on each effect of copier `id`'s target, then gives the target back
    /// the state it had: a copy adds its effect, it does not make the target grow.
    pub(crate) fn run_copied(
        &mut self,
        id: usize,
        leftmost: bool,
        mut apply: impl FnMut(&mut Game, Effects),
    ) {
        if self.copy_depth >= MAX_COPY_DEPTH.min(self.jokers.len() + 1) {
            return;
        }
        let Some(target) = self.copy_target(id, leftmost) else {
            return;
        };
        let state = target.state();
        self.copy_depth += 1;
        for effect in target.effects(self) {
            apply(self, effect);
        }
        self.copy_depth -= 1;
        if let Some(held) = self
            .jokers
            .iter_mut()
            .find(|j| j.instance_id() == target.instance_id())
        {
            held.set_state(state);
        }
    }

    /// A face card destroyed: each Canio gains X1 Mult (`cards_destroyed`).
    pub(crate) fn face_card_destroyed(&mut self) {
        self.grow_jokers(
            |j| matches!(j, Jokers::Canio(_)),
            1.0,
            |value| crate::event::JokerNote::XMult(100 + (value * 100.0).round() as usize),
        );
    }
}
