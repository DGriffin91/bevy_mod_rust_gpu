# bevy_mod_rust_gpu

WIP

[rust-gpu](https://github.com/EmbarkStudios/rust-gpu) shaders for Bevy.

- Hot reloading
- DX12 / Vulkan / WebGPU / WebGL

When building, use the `rust-gpu-builder` feature, specifc rust toolchain is required when compiling rust-gpu shaders. See `rust-toolchain.toml`

For distribution after SPIR-V files are built, rename `rust-toolchain.toml` to `rust-toolchain.toml.disabled` or use other method to use preferred toolchain.
