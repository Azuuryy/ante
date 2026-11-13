use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tip {
    pub title: String,
    pub(super) effect: String,
    pub(super) currently: Option<String>,
    pub(super) badges: Vec<String>,
    pub(super) details: Vec<String>,
    pub(super) side: Vec<Info>,
    /// Playing cards show their name in dark text on a white box (`name_from_rows`).
    pub(super) paper_name: bool,
    #[cfg(feature = "dev-tools")]
    pub(super) technical: Vec<String>,
}

impl Tip {
    pub fn new(title: impl Into<String>, effect: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            effect: effect.into(),
            currently: None,
            badges: Vec::new(),
            details: Vec::new(),
            side: Vec::new(),
            paper_name: false,
            #[cfg(feature = "dev-tools")]
            technical: Vec::new(),
        }
    }

    /// A playing card popup: the name sits on a white box.
    pub(crate) fn paper_name(mut self) -> Self {
        self.paper_name = true;
        self
    }

    pub fn currently(mut self, value: CurrentValue) -> Self {
        self.currently = Some(currently_text(value));
        self
    }

    pub fn badge(mut self, text: impl Into<String>) -> Self {
        self.badges.push(text.into());
        self
    }

    /// Adds side boxes, skipping one already shown.
    pub(crate) fn info(mut self, boxes: impl IntoIterator<Item = Info>) -> Self {
        for info in boxes {
            if !self.side.contains(&info) {
                self.side.push(info);
            }
        }
        self
    }

    pub fn detail(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        if !text.is_empty() {
            self.details.push(text);
        }
        self
    }

    pub fn technical(self, text: impl Into<String>) -> Self {
        #[cfg(feature = "dev-tools")]
        {
            let mut tip = self;
            tip.technical.push(text.into());
            tip
        }
        #[cfg(not(feature = "dev-tools"))]
        {
            let _ = text;
            self
        }
    }
}

/// A playing card popup as `generate_card_ui` builds it: the name on a white box, the chips
/// it scores and its enhancement rule. The edition and seal are badges with side boxes;
/// nothing is listed for a card without them.
pub(crate) fn playing_card(
    rank: char,
    suit: &str,
    enhancement: &str,
    edition: &str,
    seal: &str,
    debuffed: bool,
) -> Tip {
    let enhanced = crate::components::info::enhancement(enhancement);
    let stone = enhancement == "Stone";
    let (title, mut effect) = if stone {
        ("Stone Card".to_owned(), "+50 Chips".to_owned())
    } else {
        (
            format!("{} of {suit}", rank_name(rank)),
            format!("+{} chips", rank_chips(rank)),
        )
    };
    if let Some(rule) = enhanced.as_ref().filter(|_| !stone) {
        effect.push('\n');
        effect.push_str(rule.body);
    }
    let mut tip = Tip::new(title, effect).paper_name();
    if debuffed {
        tip = tip.badge("Debuffed");
    } else if let Some(rule) = &enhanced {
        tip = tip.badge(rule.title);
    }
    if let Some(info) = crate::components::info::edition(edition, false) {
        tip = tip.badge(info.title).info([info]);
    }
    if let Some(info) = crate::components::info::seal(seal) {
        tip = tip.badge(info.title).info([info]);
    }
    tip
}

/// `G.P_CARDS` rank names.
pub(super) fn rank_name(rank: char) -> String {
    match rank {
        'T' => "10".to_owned(),
        'J' => "Jack".to_owned(),
        'Q' => "Queen".to_owned(),
        'K' => "King".to_owned(),
        'A' => "Ace".to_owned(),
        other => other.to_string(),
    }
}

/// Base chips of a rank: its number, 10 for faces and 11 for the Ace.
pub(super) fn rank_chips(rank: char) -> u32 {
    match rank {
        'A' => 11,
        'T' | 'J' | 'Q' | 'K' => 10,
        other => other.to_digit(10).unwrap_or(0),
    }
}

/// Formats the grey "(Currently …)" line; `effect_line` colors the value like the original.
pub(super) fn currently_text(value: CurrentValue) -> String {
    match value {
        CurrentValue::Mult(amount) => format!("(Currently +{amount} Mult)"),
        CurrentValue::Chips(amount) => format!("(Currently +{amount} Chips)"),
        CurrentValue::XMult(hundredths) => {
            let whole = hundredths / 100;
            let fraction = hundredths % 100;
            let factor = match fraction {
                0 => whole.to_string(),
                f if f % 10 == 0 => format!("{whole}.{}", f / 10),
                f => format!("{whole}.{f:02}"),
            };
            format!("(Currently X{factor} Mult)")
        }
        CurrentValue::Money(amount) => format!("(Currently ${amount})"),
        CurrentValue::Count(amount) => format!("(Currently {amount})"),
    }
}

pub fn art(tip: Tip, image: Option<&ImageNode>) -> Tip {
    #[cfg(feature = "dev-tools")]
    {
        match image {
            Some(image) => match image.rect {
                Some(rect) => tip.technical(format!(
                    "Atlas pixels: {:.0}, {:.0} / {:.0} x {:.0}",
                    rect.min.x,
                    rect.min.y,
                    rect.width(),
                    rect.height()
                )),
                None => tip.technical("Full texture"),
            },
            None => tip.technical("Fallback: local texture or atlas entry unavailable"),
        }
    }
    #[cfg(not(feature = "dev-tools"))]
    {
        let _ = image;
        tip
    }
}

impl From<String> for Tip {
    fn from(text: String) -> Self {
        let (title, effect) = text.split_once('\n').unwrap_or((&text, ""));
        Self::new(title, effect)
    }
}
