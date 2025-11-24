#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct SpiralSettings {
    time: f32,
    intensity: f32,
}

@group(0) @binding(2) var<uniform> settings: SpiralSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let center = vec2<f32>(0.5, 0.5);
    let offset = uv - center;
    let distance = length(offset);
    let angle = atan2(offset.y, offset.x);
    
    // Spiral effect
    let spiral_amount = settings.intensity * distance * 2.0;
    let rotation = angle + spiral_amount * settings.time;
    
    let rotated_offset = vec2<f32>(
        cos(rotation) * distance,
        sin(rotation) * distance
    );
    
    let distorted_uv = center + rotated_offset;
    
    // Sample the texture
    var color = textureSample(screen_texture, texture_sampler, distorted_uv);
    
    // Darken edges during transition
    let vignette = 1.0 - smoothstep(0.0, 1.5, distance * settings.time);
    color = vec4<f32>(color.rgb * vignette, color.a);
    
    return color;
}
