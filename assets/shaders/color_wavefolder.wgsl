#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

struct CameraColorWavefolder {
    enabled: u32,
    gain: f32,
    modulus: f32,
    _padding: f32,
}

@group(0) @binding(2) var<uniform> settings: CameraColorWavefolder;

fn hard_wrap_wavefold_color(color: vec3<f32>) -> vec3<f32> {
    if settings.enabled == 0u {
        return color;
    }

    let gain = max(settings.gain, 0.0);
    let modulus = max(settings.modulus, 0.0001);
    let amplified = color * gain;

    return amplified - modulus * floor(amplified / modulus);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let source_color = textureSample(screen_texture, screen_sampler, in.uv);
    return vec4(hard_wrap_wavefold_color(source_color.rgb), source_color.a);
}