//! Headless layout checks: menus are laid out by Bevy UI without a window or GPU, and
//! sibling controls must not overlap.

use crate::{
    app::session::Session,
    components::tiles::ROOM,
    services::assets::{catalog::Atlases, import::AssetSettings},
};
use ante_core::contract::CampaignGoal;
use bevy::{prelude::*, ui::ComputedNode};

fn layout(
    content: impl FnOnce(&mut ChildSpawnerCommands, &Session, &Atlases) + Send + 'static,
) -> App {
    let mut app = harness();
    let world = app.world_mut();
    let session = world.remove_resource::<Session>().unwrap();
    let atlases = Atlases::default();
    let mut content = Some(content);
    world
        .commands()
        .spawn(Node {
            width: px(ROOM.x),
            height: px(ROOM.y),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|ui| (content.take().unwrap())(ui, &session, &atlases));
    world.flush();
    world.insert_resource(session);
    for _ in 0..3 {
        app.update();
    }
    app
}

/// A headless app with UI layout, the real font when imported, and no content.
fn harness() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<bevy::shader::Shader>();
    app.add_plugins((
        bevy::window::WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(1920, 1056)
                    .with_scale_factor_override(
                        crate::components::tiles::viewport_fit(Vec2::new(1920.0, 1056.0)).1,
                    ),
                ..default()
            }),
            ..default()
        },
        bevy::input::InputPlugin,
        bevy::image::ImagePlugin::default(),
        bevy::transform::TransformPlugin,
        bevy::camera::CameraPlugin,
        bevy::text::TextPlugin,
        bevy::picking::DefaultPickingPlugins,
        bevy::ui::UiPlugin,
        crate::components::pixel_rect::PixelRectPlugin,
    ))
    .insert_resource(Session::new(CampaignGoal::Standard, AssetSettings::load()))
    .init_resource::<Atlases>()
    .init_asset::<TextureAtlasLayout>()
    .init_asset::<Mesh>()
    .init_asset::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>();
    app.add_systems(Update, crate::app::presentation::apply_local_font);
    app.world_mut().spawn(Camera2d);
    // Text only has a size with the real font: load it from the local import when present.
    let font = AssetSettings::load()
        .active
        .and_then(|root| crate::services::assets::catalog::read_local_font(&root).ok());
    let world = app.world_mut();
    if let Some(font) = font {
        let handle = world.resource_mut::<Assets<Font>>().add(font);
        world.resource_mut::<Atlases>().font = Some(handle);
    }
    app
}

/// Rects of nodes that have text children directly, i.e. labelled controls.
fn sibling_overlaps(app: &mut App) -> Vec<String> {
    let world = app.world_mut();
    let mut query = world.query::<(
        Entity,
        &ComputedNode,
        &UiGlobalTransform,
        Option<&Children>,
        &Node,
    )>();
    // Absolute nodes are layers placed on purpose; only flow siblings must not overlap.
    let rects: Vec<(Entity, Rect, Vec<Entity>)> = query
        .iter(world)
        .filter(|(.., layout)| layout.position_type != PositionType::Absolute)
        .map(|(e, node, transform, children, _)| {
            let size = node.size();
            let center = transform.translation;
            (
                e,
                Rect::from_center_size(center, size),
                children.map(|c| c.to_vec()).unwrap_or_default(),
            )
        })
        .collect();
    if std::env::var("ANTE_LAYOUT_DUMP").is_ok() {
        let mut texts = world.query::<(&Text, &ComputedNode, &UiGlobalTransform)>();
        for (text, node, transform) in texts.iter(world) {
            eprintln!(
                "{:?} size {:?} at {:?}",
                text.0,
                node.size(),
                transform.translation
            );
        }
    }
    let laid_out = rects
        .iter()
        .filter(|r| r.1.width() > 10.0 && r.1.height() > 10.0)
        .count();
    assert!(laid_out > 10, "layout did not run: {laid_out} sized nodes");
    let find = |e: Entity| rects.iter().find(|r| r.0 == e).map(|r| r.1);
    let mut overlaps = Vec::new();
    for (_, _, children) in &rects {
        let boxes: Vec<_> = children
            .iter()
            .filter_map(|&c| find(c).map(|r| (c, r)))
            .collect();
        for (i, a) in boxes.iter().enumerate() {
            for b in &boxes[i + 1..] {
                let inter = a.1.intersect(b.1);
                if a.1.width() > 1.0
                    && b.1.width() > 1.0
                    && inter.width() > 1.0
                    && inter.height() > 1.0
                {
                    overlaps.push(format!("{:?} {:?} / {:?} {:?}", a.0, a.1, b.0, b.1));
                }
            }
        }
    }
    overlaps
}

/// Texts wider than the button box that holds them.
fn spilled_texts(app: &mut App) -> Vec<String> {
    let world = app.world_mut();
    let mut boxes = world.query::<(
        &ComputedNode,
        &UiGlobalTransform,
        &crate::components::pixel_rect::PixelBox,
    )>();
    let boxes: Vec<_> = boxes
        .iter(world)
        .map(|(node, t, _)| Rect::from_center_size(t.translation, node.size()))
        .collect();
    let mut texts = world.query::<(&Text, &ComputedNode, &UiGlobalTransform, &ChildOf)>();
    let mut parents = world.query::<&crate::components::pixel_rect::PixelBox>();
    let mut spilled = Vec::new();
    let found: Vec<_> = texts
        .iter(world)
        .map(|(text, node, t, parent)| {
            (
                text.0.clone(),
                Rect::from_center_size(t.translation, node.size()),
                parent.parent(),
            )
        })
        .collect();
    for (text, rect, parent) in found {
        if parents.get(world, parent).is_err() {
            continue;
        }
        let holder = boxes.iter().find(|b| b.contains(rect.center()));
        if let Some(holder) = holder
            && (rect.width() > holder.width() + 1.0 || rect.height() > holder.height() + 1.0)
        {
            spilled.push(format!("{text:?} {rect:?} in {holder:?}"));
        }
    }
    spilled
}

fn check(app: &mut App) {
    let overlaps = sibling_overlaps(app);
    let spilled = spilled_texts(app);
    assert!(
        overlaps.is_empty() && spilled.is_empty(),
        "overlaps {overlaps:#?}
spilled {spilled:#?}"
    );
}

#[test]
fn main_menu_lays_out_cleanly() {
    let mut app = layout(|ui, _, atlases| crate::screens::menu::main_menu(ui, atlases));
    check(&mut app);
}

#[test]
fn pause_lays_out_cleanly() {
    let mut app = layout(|ui, _, _| crate::screens::run::pause(ui, false));
    check(&mut app);
}

#[test]
fn collection_hub_lays_out_cleanly() {
    let mut app = layout(|ui, session, atlases| {
        crate::screens::collection::show_collection(
            ui,
            session,
            &crate::services::profile::Profile::load(),
            atlases,
            &ante_core::native::NativeBackend,
        );
    });
    check(&mut app);
}

#[test]
fn options_lay_out_cleanly() {
    let mut app = layout(|ui, session, _| {
        crate::screens::settings::show_options(
            ui,
            session,
            &crate::services::diagnostics::FramePacing::default(),
            &crate::services::profile::Profile::load(),
        );
    });
    check(&mut app);
}

#[test]
fn deck_select_controls_do_not_overlap() {
    let mut app = layout(|ui, session, atlases| {
        crate::screens::run::deck_select(ui, session, atlases, &ante_core::native::NativeBackend);
    });
    check(&mut app);
}

#[test]
fn hud_column_fits_the_window() {
    let mut app = harness();
    let world = app.world_mut();
    let atlases = Atlases::default();
    let board = crate::screens::run::RunBoard::spawn(
        &mut world.commands(),
        &atlases,
        crate::app::navigation::DeckChoice::Red,
        1,
        1,
    );
    world.flush();
    for _ in 0..3 {
        app.update();
    }
    let world = app.world_mut();
    let height = world
        .query::<&Window>()
        .single(world)
        .unwrap()
        .physical_height() as f32;
    // The counters, the round score and the hand box: every embossed BOSS_MAIN box.
    let mut boxes = world.query::<(
        &ComputedNode,
        &UiGlobalTransform,
        &crate::components::pixel_rect::DynRole,
    )>();
    let mut counted = 0;
    for (node, transform, role) in boxes.iter(world) {
        if *role != crate::components::pixel_rect::DynRole::BossMain {
            continue;
        }
        counted += 1;
        let rect = Rect::from_center_size(transform.translation, node.size());
        assert!(
            rect.min.y >= 0.0 && rect.max.y <= height,
            "HUD box {rect:?} leaves the {height} px window"
        );
    }
    assert!(counted >= 5, "only {counted} HUD boxes laid out");
    drop(board);
}
