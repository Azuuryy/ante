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
    let source = "extern Image noise;\nvec4 effect(vec4 c, Image t, vec2 u, vec2 s) { return c; }";
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
