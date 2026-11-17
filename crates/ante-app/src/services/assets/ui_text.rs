//! A few interface strings read from the user's local `localization/en-us.lua`.
//!
//! Only a closed list of keys is looked up, by pattern: the Lua file is never executed or
//! interpreted, and the text never ships with Ante. Each key has Ante's own fallback.

use std::{collections::HashMap, path::Path};

use bevy::prelude::*;

use super::catalog::{AtlasSource, read_local_file};

/// Imported text files, relative to the selected installation.
pub const TEXT_FILES: &[&str] = &["localization/en-us.lua"];
const MAX_TEXT: usize = 2 * 1024 * 1024;

/// Keys Ante reads, with the wording used when no local file is available.
const KEYS: &[(&str, &str)] = &[
    ("ph_improve_run", "Upgrade your run"),
    ("ante_x_voucher", "Ante #1# vouchers"),
    ("ph_blind_score_at_least", "Score at least"),
    ("ph_blind_reward", "Reward: "),
    ("b_cash_out", "Cash Out"),
    ("ph_score_at_least", "Score at least"),
    ("remaining_hand_money", "Remaining Hands ($#1# each)"),
    ("remaining_discard_money", "Remaining Discards ($#1# each)"),
    ("interest", "#1# interest per $#2# (#3# max)"),
];

/// Booster families and sizes whose `p_<family>_<size>` description is read.
const PACK_FAMILIES: [&str; 5] = ["arcana", "celestial", "spectral", "standard", "buffoon"];
const PACK_SIZES: [&str; 3] = ["normal", "jumbo", "mega"];

/// Booster description used when no local file is available.
const PACK_FALLBACK: &str = "Choose #1# of up to #2# cards";

#[derive(Resource, Default, Debug)]
pub struct UiText {
    values: HashMap<&'static str, String>,
    /// Multi-line `text = { ... }` descriptions by key, with formatting tags removed.
    texts: HashMap<String, Vec<String>>,
}

impl UiText {
    /// The local string for `key`, or Ante's fallback.
    pub fn get(&self, key: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| {
            KEYS.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, fallback)| (*fallback).to_owned())
                .unwrap_or_default()
        })
    }

    /// `get` with `#1#` replaced, like `localize{type = 'variable'}`.
    pub fn with_var(&self, key: &str, value: impl ToString) -> String {
        self.get(key).replace("#1#", &value.to_string())
    }

    /// `get` with `#1#`, `#2#`, … replaced in order.
    pub fn with_vars(&self, key: &str, values: &[usize]) -> String {
        values
            .iter()
            .enumerate()
            .fold(self.get(key), |text, (i, value)| {
                text.replace(&format!("#{}#", i + 1), &value.to_string())
            })
    }

    /// A booster description: `p_<family>_<size>` with `#1#` (choices) and `#2#` (cards).
    pub fn pack(&self, family: &str, size: &str, choose: usize, extra: usize) -> String {
        let key = format!("p_{family}_{size}");
        let lines = self
            .texts
            .get(&key)
            .map_or_else(|| PACK_FALLBACK.to_owned(), |lines| lines.join(" "));
        lines
            .replace("#1#", &choose.to_string())
            .replace("#2#", &extra.to_string())
    }

    fn from_source(source: &str) -> Self {
        let values = KEYS
            .iter()
            .filter_map(|&(key, _)| find_string(source, key).map(|value| (key, value)))
            .collect();
        let texts = PACK_FAMILIES
            .iter()
            .flat_map(|family| {
                PACK_SIZES
                    .iter()
                    .map(move |size| format!("p_{family}_{size}"))
            })
            .filter_map(|key| find_text_block(source, &key).map(|lines| (key, lines)))
            .collect();
        Self { values, texts }
    }
}

/// Startup: reads the local file when it was imported.
pub fn load(mut commands: Commands, source: Res<AtlasSource>) {
    let text = source
        .0
        .as_deref()
        .and_then(|root| read_text(root).ok())
        .map(|source| UiText::from_source(&source))
        .unwrap_or_default();
    commands.insert_resource(text);
}

fn read_text(root: &Path) -> std::io::Result<String> {
    let bytes = read_local_file(root, TEXT_FILES[0])?;
    String::from_utf8(bytes).map_err(std::io::Error::other)
}

/// Import check: a UTF-8 text file of a sensible size.
pub fn validate(bytes: &[u8]) -> std::io::Result<()> {
    if bytes.len() > MAX_TEXT {
        return Err(std::io::Error::other("Text file exceeds the size limit"));
    }
    std::str::from_utf8(bytes)
        .map(|_| ())
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

/// The double-quoted value assigned to `key` (`key = "value"`), with `\"` and `\\` escapes.
fn find_string(source: &str, key: &str) -> Option<String> {
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut rest = source;
    while let Some(index) = rest.find(key) {
        let before = rest[..index].chars().next_back();
        let after = &rest[index + key.len()..];
        rest = after;
        if before.is_some_and(is_ident) {
            continue;
        }
        let after = after.trim_start();
        let Some(after) = after.strip_prefix('=') else {
            continue;
        };
        let Some(body) = after.trim_start().strip_prefix('"') else {
            continue;
        };
        let mut value = String::new();
        let mut chars = body.chars();
        while let Some(c) = chars.next() {
            match c {
                '"' => return Some(value),
                '\\' => value.push(chars.next()?),
                '\n' => return None,
                c => value.push(c),
            }
        }
        return None;
    }
    None
}

/// The quoted lines of `key = { ... text = { "a", "b" } }`, without `{C:...}` style tags.
fn find_text_block(source: &str, key: &str) -> Option<Vec<String>> {
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut rest = source;
    while let Some(index) = rest.find(key) {
        let before = rest[..index].chars().next_back();
        let after = &rest[index + key.len()..];
        rest = after;
        if before.is_some_and(is_ident) {
            continue;
        }
        let Some(body) = after
            .trim_start()
            .strip_prefix('=')
            .and_then(|body| body.trim_start().strip_prefix('{'))
        else {
            continue;
        };
        // The entry ends at its matching brace; `text` must be inside it.
        let mut depth = 1usize;
        let mut end = None;
        let mut quoted = false;
        let mut escaped = false;
        for (offset, c) in body.char_indices() {
            match c {
                _ if escaped => escaped = false,
                '\\' if quoted => escaped = true,
                '"' => quoted = !quoted,
                '{' if !quoted => depth += 1,
                '}' if !quoted => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let entry = &body[..end?];
        let text = entry.find("text")?;
        let list = entry[text + 4..].trim_start().strip_prefix('=')?;
        let list = list.trim_start().strip_prefix('{')?;
        let mut lines = Vec::new();
        let mut rest = list;
        loop {
            let rest_trim = rest.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
            if rest_trim.starts_with('}') {
                break;
            }
            let inner = rest_trim.strip_prefix('"')?;
            let mut value = String::new();
            let mut chars = inner.char_indices();
            let mut close = None;
            while let Some((offset, c)) = chars.next() {
                match c {
                    '"' => {
                        close = Some(offset);
                        break;
                    }
                    '\\' => value.push(chars.next()?.1),
                    '\n' => return None,
                    c => value.push(c),
                }
            }
            lines.push(strip_tags(&value));
            rest = &inner[close? + 1..];
        }
        return (!lines.is_empty()).then_some(lines);
    }
    None
}

/// Removes `{...}` formatting tags, keeping the text between them.
fn strip_tags(line: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in line.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Written for Ante: the same shape as a localization table, with made-up text.
    const SAMPLE: &str = r#"
        misc = {
            dictionary = {
                xph_improve_run="wrong key",
                ph_improve_run = "Sample \"line\"",
                ante_x_voucher="Sample #1# box",
            },
        }
        Other = {
            p_arcana_normal={
                name="Sample Pack",
                text={
                    "Pick {C:attention}#1#{} from",
                    "{C:attention}#2#{C:tarot} sample{} things",
                },
            },
        }
    "#;

    #[test]
    fn reads_pack_descriptions_without_tags() {
        let text = UiText::from_source(SAMPLE);
        assert_eq!(
            text.pack("arcana", "normal", 1, 3),
            "Pick 1 from 3 sample things"
        );
        assert_eq!(
            text.pack("buffoon", "mega", 2, 4),
            "Choose 2 of up to 4 cards"
        );
    }

    #[test]
    fn reads_listed_keys_by_pattern() {
        let text = UiText::from_source(SAMPLE);
        assert_eq!(text.get("ph_improve_run"), "Sample \"line\"");
        assert_eq!(text.with_var("ante_x_voucher", 3), "Sample 3 box");
    }

    #[test]
    fn falls_back_without_a_local_file() {
        let text = UiText::default();
        assert_eq!(text.get("ph_improve_run"), "Upgrade your run");
        assert_eq!(text.get("unknown"), "");
    }

    #[test]
    fn rejects_unterminated_values() {
        assert_eq!(
            find_string("ph_improve_run = \"open\n\"", "ph_improve_run"),
            None
        );
    }
}
