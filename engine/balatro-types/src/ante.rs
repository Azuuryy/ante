use std::convert::TryFrom;

/// All ante levels.
// Goes above 8 for endless mode.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum Ante {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    BeyondEight(usize),
}

impl Ante {
    pub fn number(self) -> usize {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::BeyondEight(number) => number,
        }
    }

    pub fn next_supported(self) -> Option<Self> {
        Self::try_from(self.number().checked_add(1)?).ok()
    }

    // Base chip requirement.
    // Eventually this depends on deck and stake.
    pub fn base(&self) -> usize {
        match self {
            Self::Zero => 100,
            Self::One => 300,
            Self::Two => 800,
            Self::Three => 2000,
            Self::Four => 5000,
            Self::Five => 11000,
            Self::Six => 20000,
            Self::Seven => 35000,
            Self::Eight => 50000,
            Self::BeyondEight(number) => endless_base(*number).expect("validated Endless ante"),
        }
    }
    pub fn next(&self, max: Ante) -> Option<Self> {
        (*self < max).then(|| self.next_supported()).flatten()
    }

    /// One ante lower, `None` at `Zero`. The Hieroglyph/Petroglyph
    /// vouchers ("-1 Ante") move a run backwards along this.
    pub fn prev(&self) -> Option<Self> {
        Self::try_from(self.number().checked_sub(1)?).ok()
    }
}

impl TryFrom<usize> for Ante {
    type Error = ();

    fn try_from(i: usize) -> Result<Self, Self::Error> {
        match i {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            n if endless_base(n).is_some() => Ok(Self::BeyondEight(n)),
            _ => Err(()),
        }
    }
}

fn endless_base(ante: usize) -> Option<usize> {
    let steps = ante.checked_sub(8)? as f64;
    if steps < 1.0 {
        return None;
    }
    let scaled = 50_000.0 * (1.6 + (0.75 * steps).powf(1.0 + steps / 5.0)).powf(steps);
    if !scaled.is_finite() {
        return None;
    }
    let place = 10_f64.powf(scaled.log10().floor() - 1.0);
    let rounded = (scaled / place).floor() * place;
    let base = rounded as usize;
    base.checked_mul(6).map(|_| base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ante_base() {
        assert_eq!(Ante::Zero.base(), 100);
        assert_eq!(Ante::Eight.base(), 50000);
    }

    #[test]
    fn test_ante_next_stops_at_max() {
        assert_eq!(Ante::Zero.next(Ante::Eight), Some(Ante::One));
        assert_eq!(Ante::Eight.next(Ante::Eight), None);
    }
}
