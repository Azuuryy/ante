# Legal and assets

Ante is independent and unaffiliated with Balatro and its creators. Ante reproduces the original game's rules and behavior: logic, formulas, constants and event sequences are ported and adapted to Rust and Bevy. The original files themselves (Lua sources, images, fonts, audio, `.fs` shaders, localization text and archives) are never committed to this repository nor shipped in a release package.

The game can use those files at runtime only from the user's own installation, through the optional Local Assets importer. The importer copies an allowlist of files into local application data, validates paths and resource limits, never executes scripts or bypasses access controls, and keeps a usable fallback when a file is missing. Imported copies and reference captures stay outside version control (`captures/` is git-ignored). MIT applies only to Ante's own code; dependencies retain their respective licenses.
