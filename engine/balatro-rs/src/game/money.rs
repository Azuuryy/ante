//! Spending, with Credit Card's debt: the original lets money go down to -$20
//! (`G.GAME.bankrupt_at`). Ante keeps `money` unsigned and holds what is owed in `debt`;
//! the balance is `money - debt`, and the two are settled before each action, so money
//! earned anywhere pays the debt back first.

use super::*;

/// How far below $0 Credit Card lets money go.
pub(crate) const CREDIT_LIMIT: usize = 20;

impl Game {
    /// What can be spent now: the money, plus Credit Card's room below $0.
    pub fn spendable(&self) -> usize {
        let credit = if self.count_jokers(|j| matches!(j, Jokers::CreditCard(_))) > 0 {
            CREDIT_LIMIT
        } else {
            0
        };
        (self.money + credit).saturating_sub(self.debt)
    }

    /// Pays `cost`, going into debt for what the money does not cover.
    pub(crate) fn pay(&mut self, cost: usize) {
        if cost <= self.money {
            self.money -= cost;
        } else {
            self.debt += cost - self.money;
            self.money = 0;
        }
    }

    /// Money earned since the last action pays the debt back.
    pub(crate) fn settle_debt(&mut self) {
        let repaid = self.money.min(self.debt);
        self.money -= repaid;
        self.debt -= repaid;
    }

    /// The balance as the HUD shows it: (dollars, below zero).
    pub fn balance(&self) -> (usize, bool) {
        if self.debt > self.money {
            (self.debt - self.money, true)
        } else {
            (self.money - self.debt, false)
        }
    }
}
