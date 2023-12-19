#![cfg_attr(target_arch = "spirv", no_std)]

use glam::*;
use spirv_std::{glam, spirv, Image, Sampler};

#[cfg(target_arch = "spirv")]
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

#[repr(C)]
pub struct PostProcessSettings {
    intensity: f32,
    _padding: Vec3,
}

#[spirv(fragment)]
pub fn fragment(
    #[spirv(frag_coord)] _frag_coord: Vec4,
    uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] settings: &PostProcessSettings,
    output: &mut Vec4,
) {
    // Chromatic aberration strength
    let offset_strength = settings.intensity;

    // Sample each color channel with an arbitrary shift
    *output = vec4(
        image
            .sample(*sampler, uv + vec2(offset_strength, -offset_strength))
            .x,
        image.sample(*sampler, uv + vec2(-offset_strength, 0.0)).y,
        image.sample(*sampler, uv + vec2(0.0, offset_strength)).z,
        1.0,
    );
}
