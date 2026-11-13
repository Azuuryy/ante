use super::*;

pub(crate) fn parse_seed(backend: &dyn GameBackend, text: &str) -> Option<u64> {
    let text = text.trim();
    if text.is_empty() || text.len() > 20 {
        return None;
    }
    if text.bytes().all(|byte| byte.is_ascii_digit()) {
        return text.parse().ok();
    }
    (text.len() <= 8 && text.bytes().all(|byte| byte.is_ascii_alphanumeric()))
        .then(|| backend.parse_seed(&text.to_ascii_uppercase()))
}

pub(crate) fn edit_seed(
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    primary: Query<Entity, With<PrimaryWindow>>,
    mut session: ResMut<Session>,
    mut clicks: MessageWriter<Click>,
) {
    let Ok(window) = primary.single() else {
        return;
    };
    for key in keys.read() {
        if key.state != ButtonState::Pressed || key.window != window {
            continue;
        }
        if session.navigation.screen == Page::Options
            && session.options_tab == 2
            && session.asset_settings.editing
        {
            if key.key_code == KeyCode::Escape || key.key_code == KeyCode::Enter {
                session.asset_settings.editing = false;
            } else if held.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                if key.key_code == KeyCode::KeyV && !key.repeat {
                    clicks.write(Click::AssetPaste);
                }
            } else {
                match &key.logical_key {
                    Key::Character(text)
                        if session.asset_settings.source.len() + text.len() <= 4096 =>
                    {
                        session.asset_settings.source.push_str(text)
                    }
                    Key::Backspace => {
                        session.asset_settings.source.pop();
                    }
                    Key::Space => session.asset_settings.source.push(' '),
                    _ => {}
                }
            }
            continue;
        }
        if key.key_code == KeyCode::Escape && !key.repeat {
            if session.editing_seed {
                session.editing_seed = false;
            } else if session.navigation.screen == Page::Run {
                clicks.write(Click::OpenPause);
            } else if session.navigation.screen != Page::Main {
                clicks.write(Click::Back);
            }
            continue;
        }
        if !session.editing_seed || session.navigation.screen != Page::DeckSelect {
            continue;
        }
        if held.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            if key.key_code == KeyCode::KeyV && !key.repeat {
                clicks.write(Click::PasteSeed);
            }
            continue;
        }
        match &key.logical_key {
            Key::Character(text) if text.bytes().all(|byte| byte.is_ascii_alphanumeric()) => {
                let numeric = session
                    .seed
                    .bytes()
                    .chain(text.bytes())
                    .all(|byte| byte.is_ascii_digit());
                let limit = if numeric { 20 } else { 8 };
                if session.seed.len() + text.len() <= limit {
                    session.seed.push_str(&text.to_ascii_uppercase());
                    session.error = None;
                }
            }
            Key::Backspace => {
                session.seed.pop();
                session.error = None;
            }
            Key::Enter => {
                clicks.write(Click::Start);
            }
            _ => {}
        }
    }
}
