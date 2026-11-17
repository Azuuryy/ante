//! Content the player has not unlocked yet stays out of the shop and packs, like
//! `get_current_pool` skipping `unlocked = false` centers. Legendary Jokers are never
//! locked out: The Soul finds them whatever the profile.

use super::*;

impl Game {
    /// Sets the locked Joker and voucher ids for this run (`j_…`, `v_…`).
    pub fn set_locked(&mut self, ids: Vec<String>) {
        self.locked = ids;
        self.backend.set_locked(self.locked.clone());
    }

    /// Whether `id` is locked for this run.
    pub fn is_locked(&self, id: &str) -> bool {
        self.locked.iter().any(|locked| locked == id)
    }

    /// The redeemed vouchers, with the locked ones counted as taken so no offer draws
    /// them.
    pub(crate) fn voucher_pool(&self) -> Vouchers {
        let mut pool = self.vouchers.clone();
        for voucher in Voucher::iter() {
            if self.is_locked(voucher.id()) {
                pool.redeem(voucher);
            }
        }
        pool
    }
}
