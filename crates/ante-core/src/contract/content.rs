#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentId(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub id: ContentId,
    pub name: String,
    pub description: String,
    pub cost: usize,
    pub supported: bool,
    /// Human-readable condition that unlocks this item in the collection.
    /// `None` means the item is available from the start.
    pub unlock: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentGroup {
    Jokers,
    Vouchers,
    Tags,
    Tarots,
    Planets,
    Spectrals,
    Boosters,
    Blinds,
    Enhancements,
    Editions,
    Seals,
}

impl ContentGroup {
    /// Every group the Collection lists, in hub order.
    pub const ALL: [Self; 11] = [
        Self::Jokers,
        Self::Vouchers,
        Self::Tarots,
        Self::Planets,
        Self::Spectrals,
        Self::Enhancements,
        Self::Seals,
        Self::Editions,
        Self::Boosters,
        Self::Tags,
        Self::Blinds,
    ];
}

/// Booster packs by collection id, with their art family and size.
pub const BOOSTERS: [(&str, &str, PackFamily, PackSize); 15] = [
    (
        "p_arcana_normal",
        "Arcana Pack",
        PackFamily::Arcana,
        PackSize::Normal,
    ),
    (
        "p_arcana_jumbo",
        "Jumbo Arcana Pack",
        PackFamily::Arcana,
        PackSize::Jumbo,
    ),
    (
        "p_arcana_mega",
        "Mega Arcana Pack",
        PackFamily::Arcana,
        PackSize::Mega,
    ),
    (
        "p_celestial_normal",
        "Celestial Pack",
        PackFamily::Celestial,
        PackSize::Normal,
    ),
    (
        "p_celestial_jumbo",
        "Jumbo Celestial Pack",
        PackFamily::Celestial,
        PackSize::Jumbo,
    ),
    (
        "p_celestial_mega",
        "Mega Celestial Pack",
        PackFamily::Celestial,
        PackSize::Mega,
    ),
    (
        "p_spectral_normal",
        "Spectral Pack",
        PackFamily::Spectral,
        PackSize::Normal,
    ),
    (
        "p_spectral_jumbo",
        "Jumbo Spectral Pack",
        PackFamily::Spectral,
        PackSize::Jumbo,
    ),
    (
        "p_spectral_mega",
        "Mega Spectral Pack",
        PackFamily::Spectral,
        PackSize::Mega,
    ),
    (
        "p_standard_normal",
        "Standard Pack",
        PackFamily::Standard,
        PackSize::Normal,
    ),
    (
        "p_standard_jumbo",
        "Jumbo Standard Pack",
        PackFamily::Standard,
        PackSize::Jumbo,
    ),
    (
        "p_standard_mega",
        "Mega Standard Pack",
        PackFamily::Standard,
        PackSize::Mega,
    ),
    (
        "p_buffoon_normal",
        "Buffoon Pack",
        PackFamily::Buffoon,
        PackSize::Normal,
    ),
    (
        "p_buffoon_jumbo",
        "Jumbo Buffoon Pack",
        PackFamily::Buffoon,
        PackSize::Jumbo,
    ),
    (
        "p_buffoon_mega",
        "Mega Buffoon Pack",
        PackFamily::Buffoon,
        PackSize::Mega,
    ),
];

/// Collection id of a booster pack.
pub fn booster_id(family: PackFamily, size: PackSize) -> &'static str {
    BOOSTERS
        .iter()
        .find(|(_, _, f, s)| *f == family && *s == size)
        .map_or("p_arcana_normal", |(id, ..)| id)
}

/// What is drawn over a playing card's base: its enhancement, edition and seal labels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CardLook {
    pub enhancement: Option<&'static str>,
    pub edition: &'static str,
    pub seal: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopArt {
    Card {
        rank: char,
        suit: &'static str,
        look: CardLook,
    },
    Joker(&'static str),
    Consumable(&'static str),
    Pack {
        family: PackFamily,
        size: PackSize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackFamily {
    Arcana,
    Celestial,
    Spectral,
    Standard,
    Buffoon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackSize {
    Normal,
    Jumbo,
    Mega,
}
