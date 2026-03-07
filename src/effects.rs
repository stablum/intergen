use bevy::{
    core_pipeline::{
        FullscreenShader,
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, TextureFormat,
            TextureSampleType,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
};

use crate::config::EffectsConfig;

const CAMERA_EFFECTS_SHADER_PATH: &str = "shaders/color_wavefolder.wgsl";

pub(crate) struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<CameraEffectsSettings>::default(),
            UniformComponentPlugin::<CameraEffectsSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_camera_effects_pipeline);

        render_app
            .add_render_graph_node::<ViewNodeRunner<CameraEffectsNode>>(Core3d, CameraEffectsLabel)
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    CameraEffectsLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CameraEffectsLabel;

#[derive(Default)]
struct CameraEffectsNode;

impl ViewNode for CameraEffectsNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static CameraEffectsSettings,
        &'static DynamicUniformIndex<CameraEffectsSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<CameraEffectsPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<CameraEffectsSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let bind_group = render_context.render_device().create_bind_group(
            "camera_effects_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("camera_effects_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct CameraEffectsPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_camera_effects_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = render_device.create_bind_group_layout(
        "camera_effects_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<CameraEffectsSettings>(true),
            ),
        ),
    );
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load(CAMERA_EFFECTS_SHADER_PATH);
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("camera_effects_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen_shader.to_vertex_state(),
        fragment: Some(FragmentState {
            shader,
            entry_point: Some("fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(CameraEffectsPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub(crate) struct CameraEffectsSettings {
    pub(crate) wavefolder: Vec4,
    pub(crate) lens_radial: Vec4,
    pub(crate) lens_center: Vec4,
    pub(crate) lens_shape: Vec4,
    pub(crate) gaussian_blur: Vec4,
    pub(crate) bloom: Vec4,
    pub(crate) edge_detection: Vec4,
    pub(crate) edge_color: Vec4,
}

pub(crate) fn camera_effects_from_config(effects_config: &EffectsConfig) -> CameraEffectsSettings {
    let lens_center = effects_config.lens_distortion.center_clamped();
    let lens_scale = effects_config.lens_distortion.scale_clamped();
    let lens_tangential = effects_config.lens_distortion.tangential_clamped();

    CameraEffectsSettings {
        wavefolder: Vec4::new(
            enabled_flag(effects_config.color_wavefolder.enabled),
            effects_config.color_wavefolder.gain_clamped(),
            effects_config.color_wavefolder.modulus_clamped(),
            0.0,
        ),
        lens_radial: Vec4::new(
            enabled_flag(effects_config.lens_distortion.enabled),
            effects_config.lens_distortion.strength_clamped(),
            effects_config.lens_distortion.radial_k2_clamped(),
            effects_config.lens_distortion.radial_k3_clamped(),
        ),
        lens_center: Vec4::new(
            lens_center[0],
            lens_center[1],
            effects_config.lens_distortion.zoom_clamped(),
            effects_config
                .lens_distortion
                .chromatic_aberration_clamped(),
        ),
        lens_shape: Vec4::new(
            lens_scale[0],
            lens_scale[1],
            lens_tangential[0],
            lens_tangential[1],
        ),
        gaussian_blur: Vec4::new(
            enabled_flag(effects_config.gaussian_blur.enabled),
            effects_config.gaussian_blur.sigma_clamped(),
            effects_config.gaussian_blur.radius_pixels_clamped() as f32,
            0.0,
        ),
        bloom: Vec4::new(
            enabled_flag(effects_config.bloom.enabled),
            effects_config.bloom.threshold_clamped(),
            effects_config.bloom.intensity_clamped(),
            effects_config.bloom.radius_pixels_clamped() as f32,
        ),
        edge_detection: Vec4::new(
            enabled_flag(effects_config.edge_detection.enabled),
            effects_config.edge_detection.strength_clamped(),
            effects_config.edge_detection.threshold_clamped(),
            effects_config.edge_detection.mix_clamped(),
        ),
        edge_color: Vec4::new(
            effects_config.edge_detection.color[0].clamp(0.0, 1.0),
            effects_config.edge_detection.color[1].clamp(0.0, 1.0),
            effects_config.edge_detection.color[2].clamp(0.0, 1.0),
            1.0,
        ),
    }
}

pub(crate) fn effects_status_messages(effects_config: &EffectsConfig) -> Vec<String> {
    vec![
        color_wavefolder_status_message(effects_config),
        lens_distortion_status_message(effects_config),
        gaussian_blur_status_message(effects_config),
        bloom_status_message(effects_config),
        edge_detection_status_message(effects_config),
    ]
}

fn enabled_flag(enabled: bool) -> f32 {
    if enabled { 1.0 } else { 0.0 }
}

fn color_wavefolder_status_message(effects_config: &EffectsConfig) -> String {
    if !effects_config.color_wavefolder.enabled {
        return "Camera-output color wavefolder: disabled".to_string();
    }

    format!(
        "Camera-output color wavefolder: hard wrap enabled, gain {:.2}, modulus {:.2}",
        effects_config.color_wavefolder.gain_clamped(),
        effects_config.color_wavefolder.modulus_clamped()
    )
}

fn lens_distortion_status_message(effects_config: &EffectsConfig) -> String {
    if !effects_config.lens_distortion.enabled {
        return "Camera-output lens distortion: disabled".to_string();
    }

    let center = effects_config.lens_distortion.center_clamped();

    format!(
        "Camera-output lens distortion: enabled, k1 {:.2}, k2 {:.2}, k3 {:.2}, center ({:.2}, {:.2}), zoom {:.2}, chroma {:.3}",
        effects_config.lens_distortion.strength_clamped(),
        effects_config.lens_distortion.radial_k2_clamped(),
        effects_config.lens_distortion.radial_k3_clamped(),
        center[0],
        center[1],
        effects_config.lens_distortion.zoom_clamped(),
        effects_config
            .lens_distortion
            .chromatic_aberration_clamped()
    )
}

fn gaussian_blur_status_message(effects_config: &EffectsConfig) -> String {
    if !effects_config.gaussian_blur.enabled {
        return "Camera-output gaussian blur: disabled".to_string();
    }

    format!(
        "Camera-output gaussian blur: enabled, sigma {:.2}, radius {} px",
        effects_config.gaussian_blur.sigma_clamped(),
        effects_config.gaussian_blur.radius_pixels_clamped()
    )
}

fn bloom_status_message(effects_config: &EffectsConfig) -> String {
    if !effects_config.bloom.enabled {
        return "Camera-output bloom: disabled".to_string();
    }

    format!(
        "Camera-output bloom: enabled, threshold {:.2}, intensity {:.2}, radius {} px",
        effects_config.bloom.threshold_clamped(),
        effects_config.bloom.intensity_clamped(),
        effects_config.bloom.radius_pixels_clamped()
    )
}

fn edge_detection_status_message(effects_config: &EffectsConfig) -> String {
    if !effects_config.edge_detection.enabled {
        return "Camera-output edge detection: disabled".to_string();
    }

    format!(
        "Camera-output edge detection: enabled, strength {:.2}, threshold {:.2}, mix {:.2}",
        effects_config.edge_detection.strength_clamped(),
        effects_config.edge_detection.threshold_clamped(),
        effects_config.edge_detection.mix_clamped()
    )
}

#[cfg(test)]
fn hard_wrap_wavefold(value: f32, gain: f32, modulus: f32) -> f32 {
    let gain = gain.max(0.0);
    let modulus = modulus.max(0.0001);
    let amplified = value * gain;

    amplified - modulus * (amplified / modulus).floor()
}

#[cfg(test)]
fn distorted_uv(
    uv: Vec2,
    center: Vec2,
    zoom: f32,
    scale: Vec2,
    radial: Vec3,
    tangential: Vec2,
) -> Vec2 {
    let centered = (uv - center) * 2.0;
    let normalized = centered / zoom.max(0.1) / scale.max(Vec2::splat(0.1));
    let radius_sq = normalized.length_squared();
    let radius_quartic = radius_sq * radius_sq;
    let radius_sextic = radius_quartic * radius_sq;
    let radial_gain = 1.0
        + radial.x.clamp(-4.0, 4.0) * radius_sq
        + radial.y.clamp(-4.0, 4.0) * radius_quartic
        + radial.z.clamp(-4.0, 4.0) * radius_sextic;
    let tangential = tangential.clamp(Vec2::splat(-2.0), Vec2::splat(2.0));
    let tangential_offset = Vec2::new(
        2.0 * tangential.x * normalized.x * normalized.y
            + tangential.y * (radius_sq + 2.0 * normalized.x * normalized.x),
        tangential.x * (radius_sq + 2.0 * normalized.y * normalized.y)
            + 2.0 * tangential.y * normalized.x * normalized.y,
    );
    let distorted = (normalized * radial_gain + tangential_offset) * scale.max(Vec2::splat(0.1));

    center + distorted * (zoom.max(0.1) * 0.5)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Vec2, Vec3, Vec4};

    use super::{
        camera_effects_from_config, distorted_uv, effects_status_messages, hard_wrap_wavefold,
    };
    use crate::config::{
        BloomConfig, ColorWavefolderConfig, EdgeDetectionConfig, EffectsConfig, GaussianBlurConfig,
        LensDistortionConfig,
    };

    #[test]
    fn hard_wrap_wavefold_keeps_only_division_remainder() {
        let wrapped = hard_wrap_wavefold(0.75, 2.0, 1.0);

        assert!((wrapped - 0.5).abs() < 1e-6);
    }

    #[test]
    fn radial_lens_distortion_keeps_configured_center_fixed() {
        let center = Vec2::new(0.3, 0.7);
        let uv = distorted_uv(
            center,
            center,
            0.8,
            Vec2::new(1.4, 0.7),
            Vec3::new(0.8, -0.25, 0.1),
            Vec2::new(0.05, -0.03),
        );

        assert!(uv.distance(center) < 1e-6);
    }

    #[test]
    fn camera_effects_settings_use_clamped_config_values() {
        let settings = camera_effects_from_config(&EffectsConfig {
            color_wavefolder: ColorWavefolderConfig {
                enabled: true,
                gain: -3.0,
                modulus: 0.0,
            },
            lens_distortion: LensDistortionConfig {
                enabled: true,
                strength: 5.0,
                radial_k2: -5.0,
                radial_k3: 6.0,
                zoom: 0.0,
                center: [1.2, -0.3],
                scale: [0.0, 9.0],
                tangential: [3.0, -4.0],
                chromatic_aberration: 2.0,
            },
            gaussian_blur: GaussianBlurConfig {
                enabled: true,
                sigma: -2.0,
                radius_pixels: 99,
            },
            bloom: BloomConfig {
                enabled: true,
                threshold: -0.5,
                intensity: -1.0,
                radius_pixels: 42,
            },
            edge_detection: EdgeDetectionConfig {
                enabled: true,
                strength: -1.0,
                threshold: -0.5,
                mix: 4.0,
                color: [1.2, -0.3, 0.4],
            },
        });

        assert_eq!(settings.wavefolder, Vec4::new(1.0, 0.0, 0.0001, 0.0));
        assert_eq!(settings.lens_radial, Vec4::new(1.0, 4.0, -4.0, 4.0));
        assert_eq!(settings.lens_center, Vec4::new(1.0, 0.0, 0.1, 0.5));
        assert_eq!(settings.lens_shape, Vec4::new(0.1, 4.0, 2.0, -2.0));
        assert_eq!(settings.gaussian_blur, Vec4::new(1.0, 0.0001, 16.0, 0.0));
        assert_eq!(settings.bloom, Vec4::new(1.0, 0.0, 0.0, 16.0));
        assert_eq!(settings.edge_detection, Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(settings.edge_color, Vec4::new(1.0, 0.0, 0.4, 1.0));
    }

    #[test]
    fn status_messages_report_all_effects() {
        let messages = effects_status_messages(&EffectsConfig::default());

        assert_eq!(messages.len(), 5);
        assert!(messages[0].contains("wavefolder"));
        assert!(messages[1].contains("lens distortion"));
        assert!(messages[2].contains("gaussian blur"));
        assert!(messages[3].contains("bloom"));
        assert!(messages[4].contains("edge detection"));
    }
}
