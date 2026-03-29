pub(crate) struct MaterialAppearance {
    pub(crate) base_color: [f32; 4],
    pub(crate) metallic: f32,
    pub(crate) perceptual_roughness: f32,
    pub(crate) reflectance: f32,
}

#[derive(Clone, Copy)]
struct SurfaceProperties {
    metallic: f32,
    perceptual_roughness: f32,
    reflectance: f32,
    saturation_bias: f32,
    lightness_bias: f32,
}

pub(crate) fn material_appearance(
    node: &ShapeNode,
    material_config: &MaterialConfig,
    opacity: f32,
) -> MaterialAppearance {
    let family = material_config.surface_family(node.level);
    let surface = resolved_surface(material_config, family);
    let level = node.level as f32;
    let hue = (node.level as f32 * material_config.hue_step_per_level
        + material_config.hue_bias(node.kind))
    .rem_euclid(360.0);
    let saturation = (material_config.saturation
        + surface.saturation_bias
        + material_config.level_saturation_shift * level)
        .clamp(0.0, 1.0);
    let lightness = (material_config.lightness
        + surface.lightness_bias
        + material_config.level_lightness_shift * level)
        .clamp(0.0, 1.0);
    let rgb = hsl_to_rgb(hue, saturation, lightness);
    let opacity = opacity.clamp(0.0, 1.0);

    MaterialAppearance {
        base_color: [rgb[0], rgb[1], rgb[2], opacity],
        metallic: (surface.metallic + material_config.level_metallic_shift * level).clamp(0.0, 1.0),
        perceptual_roughness: (surface.perceptual_roughness
            + material_config.level_roughness_shift * level)
            .clamp(0.02, 1.0),
        reflectance: (surface.reflectance + material_config.level_reflectance_shift * level)
            .clamp(0.0, 1.0),
    }
}

fn resolved_surface(
    material_config: &MaterialConfig,
    family: MaterialSurfaceFamily,
) -> SurfaceProperties {
    match family {
        MaterialSurfaceFamily::Legacy => SurfaceProperties {
            metallic: material_config.metallic,
            perceptual_roughness: material_config.perceptual_roughness,
            reflectance: material_config.reflectance,
            saturation_bias: 0.0,
            lightness_bias: 0.0,
        },
        MaterialSurfaceFamily::Matte => SurfaceProperties {
            metallic: 0.0,
            perceptual_roughness: 0.92,
            reflectance: 0.22,
            saturation_bias: -0.06,
            lightness_bias: 0.02,
        },
        MaterialSurfaceFamily::Satin => SurfaceProperties {
            metallic: 0.08,
            perceptual_roughness: 0.5,
            reflectance: 0.32,
            saturation_bias: 0.0,
            lightness_bias: 0.0,
        },
        MaterialSurfaceFamily::Glossy => SurfaceProperties {
            metallic: 0.02,
            perceptual_roughness: 0.18,
            reflectance: 0.56,
            saturation_bias: 0.02,
            lightness_bias: 0.04,
        },
        MaterialSurfaceFamily::Metal => SurfaceProperties {
            metallic: 0.92,
            perceptual_roughness: 0.28,
            reflectance: 0.82,
            saturation_bias: -0.22,
            lightness_bias: -0.08,
        },
        MaterialSurfaceFamily::Frosted => SurfaceProperties {
            metallic: 0.0,
            perceptual_roughness: 0.38,
            reflectance: 0.7,
            saturation_bias: -0.12,
            lightness_bias: 0.06,
        },
    }
}

fn shape_material(
    node: &ShapeNode,
    material_config: &MaterialConfig,
    opacity: f32,
) -> StandardMaterial {
    let appearance = material_appearance(node, material_config, opacity);

    StandardMaterial {
        base_color: Color::srgba(
            appearance.base_color[0],
            appearance.base_color[1],
            appearance.base_color[2],
            appearance.base_color[3],
        ),
        alpha_mode: alpha_mode_for_opacity(appearance.base_color[3]),
        metallic: appearance.metallic,
        perceptual_roughness: appearance.perceptual_roughness,
        reflectance: appearance.reflectance,
        ..default()
    }
}

pub(crate) fn hsl_to_rgb(hue_degrees: f32, saturation: f32, lightness: f32) -> [f32; 3] {
    if saturation <= f32::EPSILON {
        return [lightness, lightness, lightness];
    }

    let hue = hue_degrees.rem_euclid(360.0) / 360.0;
    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let p = 2.0 * lightness - q;

    [
        hue_to_rgb(p, q, hue + 1.0 / 3.0),
        hue_to_rgb(p, q, hue),
        hue_to_rgb(p, q, hue - 1.0 / 3.0),
    ]
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

pub(crate) fn alpha_mode_for_opacity(opacity: f32) -> AlphaMode {
    if opacity < 0.999 {
        AlphaMode::Blend
    } else {
        AlphaMode::Opaque
    }
}

pub(crate) fn opacity_status_message(opacity: f32) -> String {
    format!(
        "Global object opacity: {:.0}%",
        opacity.clamp(0.0, 1.0) * 100.0
    )
}
