#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

const MAX_KERNEL_RADIUS_PIXELS: i32 = 16;

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

struct CameraEffectsSettings {
    wavefolder: vec4<f32>,
    lens_distortion: vec4<f32>,
    gaussian_blur: vec4<f32>,
    bloom: vec4<f32>,
    edge_detection: vec4<f32>,
    edge_color: vec4<f32>,
}

@group(0) @binding(2) var<uniform> settings: CameraEffectsSettings;

fn sample_source(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(screen_texture, screen_sampler, uv);
}

fn safe_sample_source(uv: vec2<f32>) -> vec4<f32> {
    if any(uv < vec2<f32>(0.0, 0.0)) || any(uv > vec2<f32>(1.0, 1.0)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    return sample_source(uv);
}

fn distorted_uv(uv: vec2<f32>) -> vec2<f32> {
    if settings.lens_distortion.x < 0.5 {
        return uv;
    }

    let centered = (uv - vec2<f32>(0.5, 0.5)) * 2.0;
    let zoom = max(settings.lens_distortion.z, 0.1);
    let scaled = centered / zoom;
    let radius_sq = dot(scaled, scaled);
    let strength = clamp(settings.lens_distortion.y, -1.5, 1.5);
    let distorted = scaled * (1.0 + strength * radius_sq);

    return distorted * 0.5 + vec2<f32>(0.5, 0.5);
}

fn distorted_source_color(uv: vec2<f32>) -> vec4<f32> {
    return safe_sample_source(distorted_uv(uv));
}

fn hard_wrap_wavefold_color(color: vec3<f32>) -> vec3<f32> {
    if settings.wavefolder.x < 0.5 {
        return color;
    }

    let gain = max(settings.wavefolder.y, 0.0);
    let modulus = max(settings.wavefolder.z, 0.0001);
    let amplified = color * gain;

    return amplified - modulus * floor(amplified / modulus);
}

fn wavefolded_source_color(uv: vec2<f32>) -> vec4<f32> {
    let source = distorted_source_color(uv);
    return vec4<f32>(hard_wrap_wavefold_color(source.rgb), source.a);
}

fn gaussian_weight(offset: vec2<f32>, sigma_value: f32) -> f32 {
    let sigma = max(sigma_value, 0.0001);
    let sigma_sq = sigma * sigma;
    return exp(-dot(offset, offset) / (2.0 * sigma_sq));
}

fn blurred_color(uv: vec2<f32>) -> vec4<f32> {
    let base = wavefolded_source_color(uv);
    if settings.gaussian_blur.x < 0.5 {
        return base;
    }

    let dimensions = vec2<f32>(textureDimensions(screen_texture));
    let texel = 1.0 / dimensions;
    let sigma = max(settings.gaussian_blur.y, 0.0001);
    let radius = clamp(i32(round(settings.gaussian_blur.z)), 0, MAX_KERNEL_RADIUS_PIXELS);

    var sum_rgb = vec3<f32>(0.0, 0.0, 0.0);
    var sum_alpha = 0.0;
    var total_weight = 0.0;

    for (var y = -MAX_KERNEL_RADIUS_PIXELS; y <= MAX_KERNEL_RADIUS_PIXELS; y = y + 1) {
        for (var x = -MAX_KERNEL_RADIUS_PIXELS; x <= MAX_KERNEL_RADIUS_PIXELS; x = x + 1) {
            if abs(x) > radius || abs(y) > radius {
                continue;
            }

            let pixel_offset = vec2<f32>(f32(x), f32(y));
            let weight = gaussian_weight(pixel_offset, sigma);
            let sample = wavefolded_source_color(uv + pixel_offset * texel);
            sum_rgb += sample.rgb * weight;
            sum_alpha += sample.a * weight;
            total_weight += weight;
        }
    }

    let inv_total_weight = 1.0 / max(total_weight, 0.0001);
    return vec4<f32>(sum_rgb * inv_total_weight, sum_alpha * inv_total_weight);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn bright_pass(color: vec3<f32>, threshold: f32) -> vec3<f32> {
    let brightness = max(luminance(color) - threshold, 0.0);
    return color * brightness;
}

fn bloomed_color(uv: vec2<f32>) -> vec4<f32> {
    let base = blurred_color(uv);
    if settings.bloom.x < 0.5 {
        return base;
    }

    let dimensions = vec2<f32>(textureDimensions(screen_texture));
    let texel = 1.0 / dimensions;
    let threshold = max(settings.bloom.y, 0.0);
    let intensity = max(settings.bloom.z, 0.0);
    let radius = clamp(i32(round(settings.bloom.w)), 0, MAX_KERNEL_RADIUS_PIXELS);
    let sigma = max(f32(radius) * 0.6, 0.75);

    var sum_rgb = vec3<f32>(0.0, 0.0, 0.0);
    var total_weight = 0.0;

    for (var y = -MAX_KERNEL_RADIUS_PIXELS; y <= MAX_KERNEL_RADIUS_PIXELS; y = y + 1) {
        for (var x = -MAX_KERNEL_RADIUS_PIXELS; x <= MAX_KERNEL_RADIUS_PIXELS; x = x + 1) {
            if abs(x) > radius || abs(y) > radius {
                continue;
            }

            let pixel_offset = vec2<f32>(f32(x), f32(y));
            let weight = gaussian_weight(pixel_offset, sigma);
            let sample = wavefolded_source_color(uv + pixel_offset * texel);
            sum_rgb += bright_pass(sample.rgb, threshold) * weight;
            total_weight += weight;
        }
    }

    let bloom_rgb = sum_rgb * (intensity / max(total_weight, 0.0001));
    return vec4<f32>(base.rgb + bloom_rgb, base.a);
}

fn edge_amount(uv: vec2<f32>) -> f32 {
    if settings.edge_detection.x < 0.5 {
        return 0.0;
    }

    let dimensions = vec2<f32>(textureDimensions(screen_texture));
    let texel = 1.0 / dimensions;

    let top_left = luminance(wavefolded_source_color(uv + texel * vec2<f32>(-1.0, -1.0)).rgb);
    let top = luminance(wavefolded_source_color(uv + texel * vec2<f32>(0.0, -1.0)).rgb);
    let top_right = luminance(wavefolded_source_color(uv + texel * vec2<f32>(1.0, -1.0)).rgb);
    let left = luminance(wavefolded_source_color(uv + texel * vec2<f32>(-1.0, 0.0)).rgb);
    let right = luminance(wavefolded_source_color(uv + texel * vec2<f32>(1.0, 0.0)).rgb);
    let bottom_left = luminance(wavefolded_source_color(uv + texel * vec2<f32>(-1.0, 1.0)).rgb);
    let bottom = luminance(wavefolded_source_color(uv + texel * vec2<f32>(0.0, 1.0)).rgb);
    let bottom_right = luminance(wavefolded_source_color(uv + texel * vec2<f32>(1.0, 1.0)).rgb);

    let gradient_x =
        -top_left - 2.0 * left - bottom_left + top_right + 2.0 * right + bottom_right;
    let gradient_y =
        -top_left - 2.0 * top - top_right + bottom_left + 2.0 * bottom + bottom_right;
    let magnitude = length(vec2<f32>(gradient_x, gradient_y));
    let strength = max(settings.edge_detection.y, 0.0);
    let threshold = max(settings.edge_detection.z, 0.0);

    return clamp(magnitude * strength - threshold, 0.0, 1.0);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let base = bloomed_color(in.uv);
    let edge = edge_amount(in.uv);
    let edge_mix = clamp(settings.edge_detection.w, 0.0, 1.0) * edge;
    let final_rgb = mix(base.rgb, settings.edge_color.rgb, edge_mix);

    return vec4<f32>(final_rgb, base.a);
}
