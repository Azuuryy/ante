use super::*;

pub fn load(
    source: Res<AtlasSource>,
    mut atlases: ResMut<Atlases>,
    mut images: ResMut<Assets<Image>>,
    mut fonts: ResMut<Assets<Font>>,
    mut metrics: ResMut<FrameMetrics>,
) {
    let Some(root) = &source.0 else { return };
    let start = crate::services::diagnostics::measurement();
    let mut load_one = |name| {
        let path = format!("resources/textures/1x/{name}");
        let dimensions = LOCAL_FILES
            .iter()
            .find(|(file, _)| *file == path)
            .and_then(|(_, dimensions)| *dimensions)
            .expect("known atlas");
        match read_atlas(root, name, dimensions) {
            Ok(image) => {
                crate::journal_event!("atlas_loaded", json!({"file": name}));
                Some(images.add(image))
            }
            Err(_error) => {
                crate::journal_event!(
                    "atlas_fallback",
                    json!({"file": name, "error": _error.to_string()})
                );
                None
            }
        }
    };
    atlases.cards = load_one("8BitDeck.png");
    atlases.card_surfaces = load_one("Enhancers.png");
    atlases.boosters = load_one("boosters.png");
    atlases.vouchers = load_one("Vouchers.png");
    atlases.jokers = load_one("Jokers.png");
    atlases.consumables = load_one("Tarots.png");
    atlases.shop_sign = load_one("ShopSignAnimation.png");
    atlases.blind_chips = load_one("BlindChips.png");
    atlases.tags = load_one("tags.png");
    atlases.stake_chips = load_one("chips.png");
    atlases.logo = load_one("balatro.png");
    atlases.ui_assets = load_one("ui_assets.png");
    atlases.font = match read_local_font(root) {
        Ok(font) => Some(fonts.add(font)),
        Err(_error) => {
            crate::journal_event!(
                "asset_fallback",
                json!({"file": "m6x11plus.ttf", "error": _error.to_string()}),
            );
            None
        }
    };
    atlases.status = Some(match (atlases.cards.is_some(), atlases.jokers.is_some()) {
        (true, true) => "Local card faces and joker atlas loaded",
        (true, false) => "Local card faces loaded - joker placeholders",
        (false, true) => "Card placeholders - local joker atlas loaded",
        (false, false) => "Local atlases unavailable - using placeholders",
    });
    metrics.record_assets(start.elapsed());
}

pub(crate) const LOCAL_FILES: &[(&str, Option<(u32, u32)>)] = &[
    ("resources/textures/1x/8BitDeck.png", Some((923, 380))),
    ("resources/textures/1x/Enhancers.png", Some((497, 475))),
    ("resources/textures/1x/boosters.png", Some((284, 855))),
    ("resources/textures/1x/Vouchers.png", Some((639, 380))),
    ("resources/textures/1x/Jokers.png", Some((710, 1520))),
    ("resources/textures/1x/Tarots.png", Some((710, 570))),
    (
        "resources/textures/1x/ShopSignAnimation.png",
        Some((452, 57)),
    ),
    ("resources/textures/1x/BlindChips.png", Some((714, 1054))),
    ("resources/textures/1x/tags.png", Some((204, 170))),
    ("resources/textures/1x/chips.png", Some((145, 58))),
    ("resources/textures/1x/balatro.png", Some((333, 216))),
    ("resources/textures/1x/ui_assets.png", Some((72, 36))),
    ("resources/fonts/m6x11plus.ttf", None),
];

pub(crate) fn read_local_file(root: &Path, relative: &str) -> io::Result<Vec<u8>> {
    let root = root.canonicalize()?;
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err(io::Error::other(
            "Asset must be a regular file inside the selected folder",
        ));
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::other("Asset exceeds the file size limit"));
    }
    Ok(bytes)
}

pub(crate) fn validate_local_file(bytes: &[u8], dimensions: Option<(u32, u32)>) -> io::Result<()> {
    match dimensions {
        Some(dimensions) => decode_atlas(bytes, dimensions).map(|_| ()),
        None => validate_font(bytes),
    }
}

pub(crate) fn read_local_font(root: &Path) -> io::Result<Font> {
    let bytes = read_local_file(root, "resources/fonts/m6x11plus.ttf")?;
    validate_font(&bytes)?;
    Ok(Font::from_bytes(bytes))
}

pub(super) fn validate_font(bytes: &[u8]) -> io::Result<()> {
    if bytes.len() as u64 > MAX_BYTES || skrifa::FontRef::new(bytes).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid local font",
        ));
    }
    Ok(())
}

pub(super) fn read_atlas(root: &Path, name: &str, dimensions: (u32, u32)) -> io::Result<Image> {
    let bytes = read_local_file(root, &format!("resources/textures/1x/{name}"))?;
    decode_atlas(&bytes, dimensions)
}

pub(super) fn decode_atlas(bytes: &[u8], (width, height): (u32, u32)) -> io::Result<Image> {
    validate_png(bytes, width, height)?;
    Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::default(),
    )
    .map_err(io::Error::other)
}

pub(super) fn validate_png(bytes: &[u8], width: u32, height: u32) -> io::Result<()> {
    if bytes.len() as u64 > MAX_BYTES
        || bytes.get(..16) != Some(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR")
        || bytes.get(16..20) != Some(&width.to_be_bytes())
        || bytes.get(20..24) != Some(&height.to_be_bytes())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unexpected PNG dimensions or format",
        ));
    }
    Ok(())
}
