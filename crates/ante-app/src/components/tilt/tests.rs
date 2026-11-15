use super::*;

#[test]
fn only_a_hovered_card_under_the_cursor_tilts() {
    let screen = Vec2::new(1920.0, 1080.0);
    let point = Vec2::new(300.0, 400.0);
    let cursor = Some(point);
    assert_eq!(
        tilt_params(false, cursor, screen, 70.0),
        TiltParams::default()
    );
    assert_eq!(tilt_params(true, None, screen, 70.0), TiltParams::default());
    let tilted = tilt_params(true, cursor, screen, 70.0);
    assert_eq!(tilted.hovering, 1.0);
    assert_eq!(tilted.cursor, point);
    assert_eq!(tilted.screen_scale, 70.0);
}

/// The headless app needs no GPU: a card sprite becomes a tilt material once its image
/// is known, and a sprite outside any card keeps its `ImageNode`.
#[test]
fn card_sprites_switch_to_the_tilt_material() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<TextureAtlasLayout>()
        .init_asset::<TiltImage>()
        .add_systems(Update, tilt_card_sprites);
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let card = app
        .world_mut()
        .spawn((Node::default(), HoverMotion::default()))
        .id();
    let face = app
        .world_mut()
        .spawn((
            Node::default(),
            ImageNode::new(image.clone()),
            ChildOf(card),
        ))
        .id();
    let loose = app
        .world_mut()
        .spawn((Node::default(), ImageNode::new(image)))
        .id();
    app.update();
    let world = app.world();
    assert!(world.get::<ImageNode>(face).is_none());
    assert!(world.get::<MaterialNode<TiltImage>>(face).is_some());
    assert_eq!(world.get::<TiltRoot>(face).map(|root| root.0), Some(card));
    assert!(world.get::<ImageNode>(loose).is_some());
    assert!(world.get::<NoTilt>(loose).is_some());
}

/// Ante's WGSL is only compiled by the GPU at run time; naga checks it here.
#[test]
fn tilt_shaders_parse_and_validate() {
    for (name, source) in [
        ("tilt.wgsl", include_str!("../tilt.wgsl")),
        ("tilt_image.wgsl", include_str!("../tilt_image.wgsl")),
    ] {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("{name}: {}", error.emit_to_string(source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|error| panic!("{name}: {error:?}"));
    }
}
