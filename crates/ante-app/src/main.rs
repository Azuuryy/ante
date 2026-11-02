use ante_core::standard_deck;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ante".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    for (i, card) in standard_deck().into_iter().take(5).enumerate() {
        let x = (i as f32 - 2.0) * 110.0;
        commands.spawn((
            Sprite::from_color(Color::srgb(0.95, 0.92, 0.84), Vec2::new(96.0, 140.0)),
            Transform::from_xyz(x, 0.0, 0.0),
        ));
        let color = match card.suit {
            ante_core::Suit::Diamonds | ante_core::Suit::Hearts => Color::srgb(0.75, 0.15, 0.15),
            _ => Color::srgb(0.12, 0.12, 0.15),
        };
        commands.spawn((
            Sprite::from_color(color, Vec2::new(20.0, 20.0)),
            Transform::from_xyz(x, 0.0, 1.0),
        ));
    }
}
