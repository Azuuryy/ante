#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(u64);

impl Score {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_scale(self, numerator: u64, denominator: u64) -> Option<Self> {
        let value = (self.0 as u128 * numerator as u128).checked_div(denominator as u128)?;
        u64::try_from(value).ok().map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::Score;

    #[test]
    fn ratio_uses_wide_intermediates_and_floors() {
        assert_eq!(Score::new(3).checked_scale(3, 2), Some(Score::new(4)));
        assert_eq!(
            Score::new(u64::MAX).checked_scale(2, 2),
            Some(Score::new(u64::MAX))
        );
        assert_eq!(Score::new(u64::MAX).checked_scale(2, 1), None);
        assert_eq!(Score::new(5).checked_scale(1, 0), None);
    }
}
