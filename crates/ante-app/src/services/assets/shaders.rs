//! Converts the user's local LÖVE fragment shaders to WGSL when they are loaded.
//!
//! The original `.fs` files are read from local application data only; they are never
//! committed or shipped. This module rewrites them in memory: `extern` uniforms move into
//! one block of `vec4` slots, LÖVE names map to GLSL 450, and an entry point feeds
//! `effect()` the atlas coordinates of the drawn sprite. naga validates the result and
//! emits WGSL for a Bevy `UiMaterial`.

use std::{collections::HashMap, fmt, path::Path};

use bevy::{prelude::*, shader::Shader};
#[cfg(feature = "dev-tools")]
use serde_json::json;

use super::catalog::{AtlasSource, read_local_file};

/// Uniform slots available to a converted shader; each `extern` takes one `vec4`.
pub const SLOTS: usize = 16;

/// Fragment shaders the importer may copy; `skew` and `vortex` are vertex-only.
pub const SHADER_FILES: &[&str] = &[
    "background",
    "booster",
    "CRT",
    "debuff",
    "dissolve",
    "flame",
    "flash",
    "foil",
    "gold_seal",
    "holo",
    "hologram",
    "negative",
    "negative_shine",
    "played",
    "polychrome",
    "splash",
    "voucher",
];

/// Gamma of the original shaders' colour space: their inputs are decoded and their output
/// encoded with it. 2.2 is LÖVE's; raise it to darken the converted shaders (background,
/// editions), lower it to brighten them.
pub const LOVE_GAMMA: f32 = 2.2;

pub fn shader_path(name: &str) -> String {
    format!("resources/shaders/{name}.fs")
}

/// Shaders converted from the local installation, by original file name.
#[derive(Resource, Default)]
pub struct LoveShaders {
    loaded: HashMap<&'static str, (Handle<Shader>, Vec<Uniform>)>,
}

impl LoveShaders {
    pub fn get(&self, name: &str) -> Option<&(Handle<Shader>, Vec<Uniform>)> {
        self.loaded.get(name)
    }
}

/// Startup: converts every available shader; a missing or failing one keeps the plain sprite.
pub fn load(mut commands: Commands, source: Res<AtlasSource>, mut assets: ResMut<Assets<Shader>>) {
    let mut shaders = LoveShaders::default();
    if let Some(root) = source.0.as_deref() {
        for &name in SHADER_FILES {
            match read_shader(root, name) {
                Ok(converted) => {
                    let handle = assets.add(Shader::from_wgsl(
                        converted.wgsl,
                        format!("local-assets/{}", shader_path(name)),
                    ));
                    shaders.loaded.insert(name, (handle, converted.uniforms));
                    crate::journal_event!("shader_loaded", json!({"file": name}));
                }
                Err(_error) => {
                    crate::journal_event!(
                        "shader_fallback",
                        json!({"file": name, "error": _error})
                    );
                }
            }
        }
    }
    commands.insert_resource(shaders);
}

fn read_shader(root: &Path, name: &str) -> Result<Converted, String> {
    let bytes = read_local_file(root, &shader_path(name)).map_err(|error| error.to_string())?;
    let source = String::from_utf8(bytes).map_err(|error| error.to_string())?;
    convert(&source).map_err(|error| error.to_string())
}

/// Import check: the file is text and converts.
pub fn validate(bytes: &[u8]) -> std::io::Result<()> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    convert(source)
        .map(|_| ())
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniformKind {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Bool,
}

/// Where a shader's named uniform lives in the slot array.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Uniform {
    pub name: String,
    pub slot: usize,
    pub kind: UniformKind,
}

#[derive(Clone, Debug)]
pub struct Converted {
    pub wgsl: String,
    pub uniforms: Vec<Uniform>,
}

#[derive(Debug)]
pub enum ShaderError {
    Unsupported(String),
    Parse(String),
    Validation(String),
    Output(String),
}

impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(detail) => write!(f, "unsupported shader: {detail}"),
            Self::Parse(detail) => write!(f, "GLSL parse failed: {detail}"),
            Self::Validation(detail) => write!(f, "validation failed: {detail}"),
            Self::Output(detail) => write!(f, "WGSL output failed: {detail}"),
        }
    }
}

/// Converts one LÖVE fragment shader to WGSL with a `fragment` entry point.
pub fn convert(source: &str) -> Result<Converted, ShaderError> {
    let (body, uniforms) = rewrite(source)?;
    let glsl = assemble(&body, &uniforms);
    let mut module = naga::front::glsl::Frontend::default()
        .parse(
            &naga::front::glsl::Options::from(naga::ShaderStage::Fragment),
            &glsl,
        )
        .map_err(|error| ShaderError::Parse(error.emit_to_string(&glsl)))?;
    for entry in &mut module.entry_points {
        entry.name = "fragment".to_owned();
    }
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|error| ShaderError::Validation(error.emit_to_string(&glsl)))?;
    let wgsl =
        naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
            .map_err(|error| ShaderError::Output(error.to_string()))?;
    Ok(Converted { wgsl, uniforms })
}

/// Drops the vertex part, collects `extern` uniforms and renames LÖVE identifiers.
fn rewrite(source: &str) -> Result<(String, Vec<Uniform>), ShaderError> {
    let mut uniforms = Vec::new();
    let mut body = String::new();
    let mut skipping = 0usize;
    for line in source.lines() {
        let trimmed = line.trim();
        if skipping > 0 {
            if trimmed.starts_with("#if") {
                skipping += 1;
            } else if trimmed.starts_with("#endif") {
                skipping -= 1;
            }
            continue;
        }
        if trimmed.starts_with("#if") && trimmed.contains("VERTEX") {
            skipping = 1;
            continue;
        }
        if let Some(declaration) = trimmed.strip_prefix("extern ") {
            let uniform = parse_extern(declaration, uniforms.len())?;
            uniforms.push(uniform);
            continue;
        }
        body.push_str(line);
        body.push('\n');
    }
    if body.contains("love_ScreenSize") {
        uniforms.push(Uniform {
            name: "love_ScreenSize".to_owned(),
            slot: uniforms.len(),
            kind: UniformKind::Vec4,
        });
    }
    if uniforms.len() > SLOTS {
        return Err(ShaderError::Unsupported(format!(
            "{} uniforms, at most {SLOTS}",
            uniforms.len()
        )));
    }
    if !body.contains("effect") {
        return Err(ShaderError::Unsupported("no effect() function".to_owned()));
    }
    for (from, to) in [
        ("MY_HIGHP_OR_MEDIUMP", ""),
        ("number", "float"),
        ("Image", "int"),
        ("texture", "love_image"),
        ("Texel", "love_texel"),
    ] {
        body = replace_word(&body, from, to);
    }
    for uniform in &uniforms {
        body = replace_word(&body, &uniform.name, &slot_expression(uniform));
    }
    Ok((body, uniforms))
}

fn parse_extern(declaration: &str, slot: usize) -> Result<Uniform, ShaderError> {
    let words: Vec<_> = declaration
        .trim_end_matches(';')
        .split_whitespace()
        .filter(|word| {
            !matches!(
                *word,
                "MY_HIGHP_OR_MEDIUMP" | "highp" | "mediump" | "lowp" | "uniform"
            )
        })
        .collect();
    let [kind, name] = words.as_slice() else {
        return Err(ShaderError::Unsupported(format!("extern {declaration}")));
    };
    let kind = match *kind {
        "float" | "number" => UniformKind::Float,
        "vec2" => UniformKind::Vec2,
        "vec3" => UniformKind::Vec3,
        "vec4" => UniformKind::Vec4,
        "bool" => UniformKind::Bool,
        other => {
            return Err(ShaderError::Unsupported(format!("extern of type {other}")));
        }
    };
    Ok(Uniform {
        name: (*name).to_owned(),
        slot,
        kind,
    })
}

fn slot_expression(uniform: &Uniform) -> String {
    let slot = format!("love_slots[{}]", uniform.slot);
    match uniform.kind {
        UniformKind::Float => format!("{slot}.x"),
        UniformKind::Vec2 => format!("{slot}.xy"),
        UniformKind::Vec3 => format!("{slot}.xyz"),
        UniformKind::Vec4 => slot,
        UniformKind::Bool => format!("({slot}.x != 0.0)"),
    }
}

fn assemble(body: &str, uniforms: &[Uniform]) -> String {
    let lookup = |name: &str| {
        uniforms
            .iter()
            .find(|uniform| uniform.name == name)
            .map(slot_expression)
    };
    // Atlas sprites: the original passes the sprite cell and pixel size in
    // texture_details and the atlas size in image_details. LÖVE shades gamma-encoded
    // colours while Bevy samples and writes linear ones, so both ends are converted.
    let coordinates = match (lookup("texture_details"), lookup("image_details")) {
        (Some(details), Some(image)) => {
            format!("({details}.xy * {details}.zw + love_uv * {details}.zw) / {image}")
        }
        _ => "love_uv".to_owned(),
    };
    format!(
        "#version 450\n\
         layout(set = 1, binding = 0) uniform LoveUniforms {{ vec4 love_slots[{SLOTS}]; }};\n\
         layout(set = 1, binding = 1) uniform texture2D love_texture;\n\
         layout(set = 1, binding = 2) uniform sampler love_sampler;\n\
         vec4 love_texel(int image, vec2 coords) {{\n\
             vec4 linear = textureLod(sampler2D(love_texture, love_sampler), coords, 0.0);\n\
             return vec4(pow(linear.rgb, vec3(1.0 / {LOVE_GAMMA:.4})), linear.a);\n\
         }}\n\
         {body}\n\
         layout(location = 0) in vec2 love_uv;\n\
         layout(location = 0) out vec4 love_out;\n\
         void main() {{\n\
             vec4 gamma = effect(vec4(1.0), 0, {coordinates}, gl_FragCoord.xy);\n\
             love_out = vec4(pow(max(gamma.rgb, vec3(0.0)), vec3({LOVE_GAMMA:.4})), gamma.a);\n\
         }}\n"
    )
}

/// Replaces whole identifiers only, so `dissolve` does not touch `dissolve_mask`.
fn replace_word(source: &str, from: &str, to: &str) -> String {
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(index) = rest.find(from) {
        let before = rest[..index].chars().next_back();
        let after = rest[index + from.len()..].chars().next();
        out.push_str(&rest[..index]);
        if before.is_some_and(is_ident) || after.is_some_and(is_ident) {
            out.push_str(from);
        } else {
            out.push_str(to);
        }
        rest = &rest[index + from.len()..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Written for Ante in the LÖVE dialect; exercises every construct the converter handles.
    const SAMPLE: &str = r#"
#if defined(VERTEX) || __VERSION__ > 100 || defined(GL_FRAGMENT_PRECISION_HIGH)
    #define MY_HIGHP_OR_MEDIUMP highp
#else
    #define MY_HIGHP_OR_MEDIUMP mediump
#endif

extern MY_HIGHP_OR_MEDIUMP vec2 shine;
extern MY_HIGHP_OR_MEDIUMP number dissolve;
extern MY_HIGHP_OR_MEDIUMP vec4 texture_details;
extern MY_HIGHP_OR_MEDIUMP vec2 image_details;
extern bool shadow;

vec4 dissolve_mask(vec4 tex)
{
    return vec4(shadow ? vec3(0.) : tex.rgb, tex.a * (1. - dissolve));
}

vec4 effect(vec4 colour, Image texture, vec2 texture_coords, vec2 screen_coords)
{
    vec4 tex = Texel(texture, texture_coords);
    number glow = 0.5 + 0.5 * sin(shine.x + texture_coords.x * 10.);
    tex.rgb += vec3(glow * 0.2);
    return dissolve_mask(tex * colour);
}

extern MY_HIGHP_OR_MEDIUMP float hovering;

#ifdef VERTEX
vec4 position(mat4 transform_projection, vec4 vertex_position)
{
    return transform_projection * vertex_position + vec4(0, 0, 0, hovering);
}
#endif
"#;

    #[test]
    fn love_shader_converts_to_validated_wgsl() {
        let converted = convert(SAMPLE).unwrap();
        assert!(converted.wgsl.contains("fn fragment("));
        assert!(converted.wgsl.contains("@group(1) @binding(0)"));
        assert!(!converted.wgsl.contains("position("));
        let names: Vec<_> = converted.uniforms.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "shine",
                "dissolve",
                "texture_details",
                "image_details",
                "shadow",
                "hovering"
            ]
        );
        let shadow = converted
            .uniforms
            .iter()
            .find(|u| u.name == "shadow")
            .unwrap();
        assert_eq!(shadow.kind, UniformKind::Bool);
    }

    #[test]
    fn identifiers_are_replaced_whole() {
        assert_eq!(
            replace_word("dissolve_mask(dissolve) + xdissolve", "dissolve", "d"),
            "dissolve_mask(d) + xdissolve"
        );
    }

    #[test]
    fn unsupported_uniform_types_are_rejected() {
        let source =
            "extern Image noise;\nvec4 effect(vec4 c, Image t, vec2 u, vec2 s) { return c; }";
        assert!(matches!(convert(source), Err(ShaderError::Unsupported(_))));
    }

    /// Converts the user's own installation when present; nothing is copied.
    #[test]
    #[ignore = "reads the local Balatro installation"]
    fn local_installation_shaders_convert() {
        let Some(directory) =
            super::super::import::steam_source().map(|game| game.join("resources").join("shaders"))
        else {
            return;
        };
        let mut report = Vec::new();
        for entry in std::fs::read_dir(&directory)
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "fs") {
                let source = std::fs::read_to_string(&path).unwrap();
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                report.push(match convert(&source) {
                    Ok(_) => format!("ok   {name}"),
                    Err(error) => format!("FAIL {name}: {error}"),
                });
            }
        }
        report.sort();
        println!("{}", report.join("\n"));
    }
}
