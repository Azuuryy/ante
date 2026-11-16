use super::*;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum CollectionView {
    #[default]
    Hub,
    Decks,
    /// One page of a content group.
    Items(ContentGroup),
}

#[derive(Clone, Copy, Message)]
pub(crate) enum Click {
    EditSeed,
    ToggleSeeded,
    PasteSeed,
    ShiftStake(isize),
    OpenPause,
    OpenRunInfo,
    RunInfoTab(u8),
    OpenDeck,
    HoverDeck(bool),
    SelectOffer {
        action: Command,
        revision: u64,
    },
    ShowRemaining(bool),
    MainMenu,
    ContinueRun,
    Start,
    Restart,
    Menu,
    OpenDecks,
    OpenOptions,
    Back,
    ShiftDeck(isize),
    ToggleVsync,
    Quit,
    Play(Command),
    SelectInventory {
        lane: crate::components::card_drag::Lane,
        index: usize,
        revision: u64,
    },
    ClearSelection,
    Collection(usize),
    CollectionJokers,
    CollectionItems(CollectionView),
    CollectionDecks,
    OptionsTab(u8),
    AssetEdit,
    AssetPaste,
    AssetSteam,
    AssetImport,
    AssetRemove,
    AssetCancelRemove,
    ChangeSpeed,
    /// Steps the master volume, or the game sounds volume when true.
    Skip,
    #[cfg_attr(not(feature = "dev-tools"), allow(dead_code))]
    UnlockAll,
}
