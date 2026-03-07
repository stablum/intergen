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

use crate::config::ColorWavefolderConfig;

const COLOR_WAVEFOLDER_SHADER_PATH: &str = "shaders/color_wavefolder.wgsl";

pub(crate) struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<CameraColorWavefolder>::default(),
            UniformComponentPlugin::<CameraColorWavefolder>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_color_wavefolder_pipeline);
        render_app
            .add_render_graph_node::<ViewNodeRunner<ColorWavefolderNode>>(
                Core3d,
                ColorWavefolderLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    ColorWavefolderLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ColorWavefolderLabel;

#[derive(Default)]
struct ColorWavefolderNode;

impl ViewNode for ColorWavefolderNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static CameraColorWavefolder,
        &'static DynamicUniformIndex<CameraColorWavefolder>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<ColorWavefolderPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<CameraColorWavefolder>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let bind_group = render_context.render_device().create_bind_group(
            "color_wavefolder_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("color_wavefolder_pass"),
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
struct ColorWavefolderPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_color_wavefolder_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = render_device.create_bind_group_layout(
        "color_wavefolder_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<CameraColorWavefolder>(true),
            ),
        ),
    );
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load(COLOR_WAVEFOLDER_SHADER_PATH);
    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("color_wavefolder_pipeline".into()),
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

    commands.insert_resource(ColorWavefolderPipeline {
        layout,
        sampler,
        pipeline_id,
    });
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub(crate) struct CameraColorWavefolder {
    pub(crate) enabled: u32,
    pub(crate) gain: f32,
    pub(crate) modulus: f32,
    pub(crate) _padding: f32,
}

pub(crate) fn camera_color_wavefolder_from_config(
    color_wavefolder_config: &ColorWavefolderConfig,
) -> CameraColorWavefolder {
    CameraColorWavefolder {
        enabled: u32::from(color_wavefolder_config.enabled),
        gain: color_wavefolder_config.gain_clamped(),
        modulus: color_wavefolder_config.modulus_clamped(),
        _padding: 0.0,
    }
}

pub(crate) fn color_wavefolder_status_message(
    color_wavefolder_config: &ColorWavefolderConfig,
) -> String {
    if !color_wavefolder_config.enabled {
        return "Camera-output color wavefolder: disabled".to_string();
    }

    format!(
        "Camera-output color wavefolder: hard wrap enabled, gain {:.2}, modulus {:.2}",
        color_wavefolder_config.gain_clamped(),
        color_wavefolder_config.modulus_clamped()
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
mod tests {
    use super::{
        camera_color_wavefolder_from_config, color_wavefolder_status_message, hard_wrap_wavefold,
    };
    use crate::config::ColorWavefolderConfig;

    #[test]
    fn hard_wrap_wavefold_keeps_only_division_remainder() {
        let wrapped = hard_wrap_wavefold(0.75, 2.0, 1.0);

        assert!((wrapped - 0.5).abs() < 1e-6);
    }

    #[test]
    fn camera_wavefolder_component_uses_clamped_config_values() {
        let settings = camera_color_wavefolder_from_config(&ColorWavefolderConfig {
            enabled: true,
            gain: -3.0,
            modulus: 0.0,
        });

        assert_eq!(settings.enabled, 1);
        assert_eq!(settings.gain, 0.0);
        assert_eq!(settings.modulus, 0.0001);
    }

    #[test]
    fn disabled_status_message_is_explicit() {
        let status = color_wavefolder_status_message(&ColorWavefolderConfig {
            enabled: false,
            gain: 2.4,
            modulus: 1.0,
        });

        assert_eq!(status, "Camera-output color wavefolder: disabled");
    }
}
